import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { detailsToggle, openDetails, openRow, queueCell, selectRow as selectByName } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueEventSource } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';

describe('review actions', () => {
  afterEach(() => vi.unstubAllGlobals());
  const selectRow = openRow;
  /** A folder picker that always picks `path`. */
  const folderPicker = (path: string) => ({ pickFiles: async () => [], pickFolder: async () => ({ path, displayName: path }), pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) });
  it('keeps editing local until a nonblank filename is approved', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    const bridge = { ...baseBridge, approve };
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');
    fireEvent.change(filename, { target: { value: '' } });

    expect(approve).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    expect(screen.getByRole('alert')).toHaveTextContent('Filename is required');
    expect((await bridge.listItems()).find((item) => item.id === 'lease')?.status).toBe('review');
  });

  // FRONTEND_UX-6. A reserved character used to go to the backend and come
  // back as "filename must be one nonblank path component" in a toast.
  it('names a character Windows refuses as it is typed, and never sends the name', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    render(<App bridge={{ ...baseBridge, approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');

    fireEvent.change(filename, { target: { value: '2023-09-15 Lease Agreement: ABC Properties LLC' } });

    expect(screen.getByRole('alert')).toHaveTextContent('\u201c:\u201d cannot be used in a Windows filename.');
    expect(filename).toHaveAttribute('aria-invalid', 'true');
    expect(filename.getAttribute('aria-describedby')).toContain(screen.getByRole('alert').id);
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));
    fireEvent.keyDown(filename, { key: 'Enter' });
    expect(approve).not.toHaveBeenCalled();

    // Fixing the name clears the error without a click.
    fireEvent.change(filename, { target: { value: '2023-09-15 Lease Agreement - ABC Properties LLC' } });
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(filename).toHaveAttribute('aria-invalid', 'false');
  });

  it('keeps the extension out of the field, so it cannot be changed, and adds it back on approve', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    render(<App bridge={{ ...baseBridge, approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');

    expect(filename.tagName).toBe('TEXTAREA');
    expect(filename).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');
    expect(filename).toHaveAccessibleDescription('The name ends in .pdf, which cannot change.');
    fireEvent.change(filename, { target: { value: '2023-09-15 Lease' } });
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    await waitFor(() => expect(approve).toHaveBeenCalledWith('lease', '2023-09-15 Lease.pdf', expect.any(String)));
  });

  it('files the name on Enter, and turns pasted line breaks into spaces', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    render(<App bridge={{ ...baseBridge, approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');

    fireEvent.change(filename, { target: { value: '2023-09-15 Lease\r\nAgreement\n' } });
    expect(filename).toHaveValue('2023-09-15 Lease Agreement ');
    fireEvent.keyDown(filename, { key: 'Enter' });

    await waitFor(() => expect(approve).toHaveBeenCalledWith('lease', '2023-09-15 Lease Agreement.pdf', expect.any(String)));
  });

  it('catches a name pasted whole, extension and all, before it is filed as ".pdf.pdf"', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    render(<App bridge={{ ...baseBridge, approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));

    fireEvent.change(screen.getByLabelText('Filename'), { target: { value: '2023-09-15 Lease.PDF' } });
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    expect(screen.getByRole('alert')).toHaveTextContent('Leave \u201c.pdf\u201d off: ATLAS adds the extension itself.');
    expect(approve).not.toHaveBeenCalled();
  });

  // The backstop for a rule the window does not know: the backend's own
  // refusal, in words, and a way to put it away.
  it('explains a filename the backend refused in words, and can be dismissed', async () => {
    const approve = vi.fn(async () => { throw { code: 'NAME_INVALID', message: 'filename must be one nonblank path component' }; });
    render(<App bridge={{ ...createInMemoryBridge(), approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    const feedback = await screen.findByRole('alert', { name: 'Action error' });
    expect(feedback).toHaveTextContent('That filename cannot be used.');
    expect(feedback).not.toHaveTextContent('nonblank path component');
    fireEvent.click(within(feedback).getByRole('button', { name: 'Dismiss' }));

    expect(screen.queryByRole('alert', { name: 'Action error' })).not.toBeInTheDocument();
  });

  // FRONTEND_UX-8: a filed document's inspector called its old name its
  // current one and never said what it became. And the reviewer could not
  // open the document, or find it once filed.
  // Open, read, Approve: on Windows the viewer still holds the file, and the
  // rename failed with the system's "being used by another process".
  it('says to close a document another program holds, rather than the system error', async () => {
    const approve = vi.fn(async () => { throw { code: 'SOURCE_LOCKED', message: 'atomic no-replace rename failed (The process cannot access the file because it is being used by another process. (os error 32))' }; });
    render(<App bridge={{ ...createInMemoryBridge(), approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    const feedback = await screen.findByRole('alert', { name: 'Action error' });
    expect(feedback).toHaveTextContent('The document is open in another program, or a sync client is still writing it. Close it, then try again.');
    expect(feedback).not.toHaveTextContent('os error 32');
  });

  // Pipeline::apply_if_unchanged refuses an approval of a file that changed
  // since it was read with "Re-analyze it.", and the panel has no control by
  // that name: it says Analyze again, from the panel and from Apply all ready.
  it('points an approval refused because the file changed at Analyze again', async () => {
    const approve = vi.fn(async () => { throw { code: 'FILE_CHANGED', message: 'The file changed after it was analyzed. Re-analyze it.' }; });
    render(<App bridge={{ ...createInMemoryBridge(), approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    const feedback = await screen.findByRole('alert', { name: 'Action error' });
    expect(feedback).toHaveTextContent('Use Analyze again, under More review actions');
    expect(feedback).not.toHaveTextContent('Re-analyze');
    fireEvent.click(within(feedback).getByRole('button', { name: 'Dismiss' }));

    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    const batch = await screen.findByRole('alert', { name: 'Action error' });
    expect(batch).toHaveTextContent('could not be applied. The file changed after it was analyzed, so this name may no longer fit it. Use Analyze again');
    expect(batch).not.toHaveTextContent('Re-analyze');
  });

  // Check again (retry on a parked item) queues nothing: it finishes the
  // stopped rename or rolls it back. It can also fail for the reason the item
  // was parked, and then sending the person back to Check again is a loop.
  it('says what Check again found, and never answers a failed check with Check again', async () => {
    const parked = (id: string, errorCode: string) => ({ id, originalFilename: `${id}.pdf`, status: 'review' as const, proposedFilename: `2024-05-01 ${id}.pdf`, errorCode, parked: true });
    const base = createInMemoryBridge({ items: [parked('stuck', 'RECONCILIATION_REQUIRED'), parked('copied', 'SOURCE_DELETE_FAILED'), parked('ambiguous', 'RECONCILIATION_REQUIRED')] });
    const retry = vi.fn(async (id: string) => {
      if (id === 'ambiguous') throw { code: 'RECONCILIATION_REQUIRED', message: 'an incomplete operation left a file at both of its paths' };
      await base.retry(id);
    });
    render(<App bridge={{ ...base, retry }} />);
    const status = () => screen.getByRole('status', { name: 'Action status' });

    await selectRow(await screen.findByRole('row', { name: /stuck\.pdf/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Check again' }));
    await waitFor(() => expect(status()).toHaveTextContent('Checked stuck.pdf: it was not renamed, and waits for your decision.'));

    await selectRow(screen.getByRole('row', { name: /copied\.pdf/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Check again' }));
    await waitFor(() => expect(status()).toHaveTextContent('Checked copied.pdf: it is filed.'));
    expect(status()).not.toHaveTextContent('queued for retry');

    await selectRow(screen.getByRole('row', { name: /ambiguous\.pdf/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Check again' }));
    const feedback = await screen.findByRole('alert', { name: 'Action error' });
    expect(feedback).not.toHaveTextContent('Check again');
    expect(feedback).toHaveTextContent('Remove from queue and choose \u201cI have resolved the files myself\u201d');
    expect(retry.mock.calls.map(([id]) => id)).toEqual(['stuck', 'copied', 'ambiguous']);
  });

  // Pipeline::approve checks a parked item's files first and approves when
  // the rename never happened, so the panel offers it; a check that fails
  // inside the approval is the same dead end as a failed Check again.
  it('approves a parked item\'s name, checking its files first', async () => {
    const parked = (id: string) => ({ id, originalFilename: `${id}.pdf`, status: 'review' as const, proposedFilename: `2024-05-01 ${id}.pdf`, errorCode: 'FILE_CHANGED', parked: true });
    const base = createInMemoryBridge({ items: [parked('stuck'), parked('ambiguous')] });
    const approve = vi.fn(async (id: string, filename: string, description: string) => {
      if (id === 'ambiguous') throw { code: 'RECONCILIATION_REQUIRED', message: 'an incomplete operation left a file at both of its paths' };
      await base.approve(id, filename, description);
    });
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /ambiguous\.pdf/ }));
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    const feedback = await screen.findByRole('alert', { name: 'Action error' });
    expect(feedback).not.toHaveTextContent('Check again');
    expect(feedback).toHaveTextContent('I have resolved the files myself');

    await selectRow(screen.getByRole('row', { name: /stuck\.pdf/ }));
    fireEvent.change(screen.getByLabelText('Filename'), { target: { value: '2024-05-01 Stuck renamed' } });
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed stuck.pdf.'));
    expect((await base.listItems()).find((item) => item.id === 'stuck')).toMatchObject({ status: 'completed', filedName: '2024-05-01 Stuck renamed.pdf' });
  });

  it('completed_item_labels_and_open_reveal_buttons', async () => {
    const base = createInMemoryBridge({ items: [
      { id: 'filed', originalFilename: 'Completed lease.pdf', status: 'completed', proposedFilename: '2024-01-22 Lease Agreement.pdf', filedName: '2024-01-22 Lease Agreement (2).pdf', undoable: true },
      { id: 'kept', originalFilename: 'Board minutes.docx', status: 'completed', proposedFilename: '2024-05-07 Board Meeting Minutes.docx', keptOriginal: true, undoable: false },
      { id: 'named', originalFilename: '2024-02-01 Invoice.pdf', status: 'completed', proposedFilename: '2024-02-01 Invoice.pdf', filedName: '2024-02-01 Invoice.pdf', reason: 'This document already had this name, so nothing was renamed.' },
    ] });
    const openItem = vi.fn(async () => undefined);
    const revealItem = vi.fn(async () => undefined);
    render(<App bridge={{ ...base, openItem, revealItem }} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));

    // The table names what each file is called now.
    expect(await screen.findByRole('row', { name: /Completed lease\.pdf/ })).toHaveTextContent('2024-01-22 Lease Agreement (2).pdf');
    expect(screen.getByRole('row', { name: /Board minutes\.docx/ })).toHaveTextContent('Kept original');
    expect(screen.getByRole('row', { name: /Board minutes\.docx/ })).not.toHaveTextContent('2024-05-07 Board Meeting Minutes.docx');

    await selectRow(screen.getByRole('row', { name: /Completed lease\.pdf/ }));
    const inspector = screen.getByRole('complementary', { name: 'Review item' });
    expect(within(inspector).getByRole('heading', { level: 2 })).toHaveTextContent('Filed document');
    expect(within(inspector).getByText('Original name')).toBeVisible();
    expect(within(inspector).queryByText('Current name')).not.toBeInTheDocument();
    expect(within(inspector).getByText(/^Renamed to/)).toHaveTextContent('Renamed to 2024-01-22 Lease Agreement (2).pdf');
    fireEvent.click(within(inspector).getByRole('button', { name: 'Open' }));
    fireEvent.click(within(inspector).getByRole('button', { name: 'Show in folder' }));
    await waitFor(() => expect(openItem).toHaveBeenCalledWith('filed'));
    expect(revealItem).toHaveBeenCalledWith('filed');
    expect(within(inspector).getByRole('button', { name: 'Undo' })).toBeEnabled();

    await selectRow(screen.getByRole('row', { name: /Board minutes\.docx/ }));
    await waitFor(() => expect(inspector).toHaveTextContent('Kept its original name'));
    expect(inspector).not.toHaveTextContent('Renamed to');
    expect(within(inspector).queryByRole('button', { name: 'Undo' })).not.toBeInTheDocument();

    // A settled item's reason is a note about what happened, not a reason it waits.
    await selectRow(screen.getByRole('row', { name: /2024-02-01 Invoice\.pdf/ }));
    await waitFor(() => expect(within(inspector).getByRole('heading', { name: 'Note' })).toBeVisible());
    expect(within(inspector).queryByRole('heading', { name: 'Reason for review' })).not.toBeInTheDocument();
  });

  // The backend on this branch sends neither filedName nor keptOriginal, and
  // keeps the unapplied proposal on a kept original. The fallback to that
  // proposal said "Renamed to" a name the file never had.
  it('tells a kept original from a rename before the backend reports filings', async () => {
    render(<App bridge={createInMemoryBridge({ items: [
      { id: 'filed', originalFilename: 'Scan 0401.pdf', status: 'completed', proposedFilename: '2024-01-22 Lease Agreement.pdf', undoable: true },
      { id: 'kept', originalFilename: 'Scan 0402.pdf', status: 'completed', proposedFilename: '2024-05-07 Board Meeting Minutes.pdf', undoable: false },
    ] })} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));

    expect(await screen.findByRole('row', { name: /Scan 0401\.pdf/ })).toHaveTextContent('2024-01-22 Lease Agreement.pdf');
    expect(screen.getByRole('row', { name: /Scan 0402\.pdf/ })).toHaveTextContent('Kept original');
    expect(screen.getByRole('row', { name: /Scan 0402\.pdf/ })).not.toHaveTextContent('2024-05-07 Board Meeting Minutes.pdf');

    await selectRow(screen.getByRole('row', { name: /Scan 0401\.pdf/ }));
    const inspector = screen.getByRole('complementary', { name: 'Review item' });
    await waitFor(() => expect(inspector).toHaveTextContent('Renamed to 2024-01-22 Lease Agreement.pdf'));
    await selectRow(screen.getByRole('row', { name: /Scan 0402\.pdf/ }));
    await waitFor(() => expect(inspector).toHaveTextContent('Kept its original name'));
    expect(inspector).not.toHaveTextContent('Renamed to');
  });

  it('says plainly when the document is no longer where ATLAS saw it', async () => {
    const openItem = vi.fn(async () => { throw { code: 'PATH_UNAVAILABLE', message: 'the document is not where ATLAS last saw it' }; });
    render(<App bridge={{ ...createInMemoryBridge(), openItem }} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('It may have been moved, renamed, or deleted outside ATLAS.');
    // Opening is not a decision: the item is still there to decide.
    expect(screen.getByRole('button', { name: /Approve & rename/i })).toBeEnabled();
  });

  it('sends an ordinary review item back to be analyzed again, and it returns for review', async () => {
    const bridge = createInMemoryBridge({ liveEvents: true, analysisDelayMs: 20 });
    const reanalyze = vi.spyOn(bridge, 'reanalyze');
    const retry = vi.spyOn(bridge, 'retry');
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: 'More review actions' }));
    expect(screen.queryByRole('button', { name: /^Retry/ })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Analyze again' }));

    await waitFor(() => expect(reanalyze).toHaveBeenCalledWith('lease'));
    expect(retry).not.toHaveBeenCalled();
    await waitFor(() => expect(screen.getByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i })).toHaveTextContent('Waiting'));
    await waitFor(() => expect(screen.getByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i })).toHaveTextContent('Needs review'));
  });

  // The description is half of what Intern produces, and it used to be rendered
  // only while an item was still ready or in review. Applying a rename made the
  // item `completed`, which hid the sentence for good - so the fact describing
  // the document disappeared exactly when the document was filed.
  it('still shows the description after a file has been renamed', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));
    await selectRow(await screen.findByRole('row', { name: /Completed lease.pdf/i }));

    const inspector = screen.getByRole('complementary', { name: 'Review item' });
    expect(within(inspector).getByText(/Residential lease agreement for a twelve-month term/i)).toBeVisible();
    // Settled, so it is reported rather than edited - but it must be reachable.
    expect(within(inspector).queryByLabelText('Description')).not.toBeInTheDocument();
    expect(within(inspector).getByRole('button', { name: /Copy description/i })).toBeEnabled();
  });

  it('shows each description in the queue without needing a row selected', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const row = await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i });

    expect(within(row).getByText(/Commercial lease agreement between landlord and tenant/i)).toBeVisible();
  });

  it('preserves a draft through a same-revision queue refresh', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');
    fireEvent.change(filename, { target: { value: 'My local draft' } });
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    await waitFor(() => expect(filename).toHaveValue('My local draft'));
  });

  // Moving on (Alt+N, Next undecided, another row) threw the typing away.
  // It is set aside for the item, and for the reading it was typed against.
  it('keeps what was typed for an item left undecided, for that item and that reading only', () => {
    const calls = { onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(), onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn() };
    const lease: QueueItem = { id: 'lease', originalFilename: 'scan.pdf', status: 'review', proposedFilename: '2024-03-01 Lease.pdf', description: 'A lease.', confidence: 0.8, proposalRevision: '1' };
    const invoice: QueueItem = { id: 'invoice', originalFilename: 'scan 2.pdf', status: 'review', proposedFilename: '2024-04-01 Invoice.pdf', description: 'An invoice.', confidence: 0.8, proposalRevision: '1' };
    const view = render(<ReviewInspector item={lease} drawer={false} {...calls} />);
    const filename = () => screen.getByLabelText('Filename');
    const description = () => screen.getByLabelText('Description');
    fireEvent.change(filename(), { target: { value: '2024-03-01 Lease of Unit 4' } });
    fireEvent.change(description(), { target: { value: 'A lease of Unit 4.' } });

    view.rerender(<ReviewInspector item={invoice} drawer={false} {...calls} />);
    expect(filename()).toHaveValue('2024-04-01 Invoice');
    expect(description()).toHaveValue('An invoice.');
    view.rerender(<ReviewInspector item={lease} drawer={false} {...calls} />);
    expect(filename()).toHaveValue('2024-03-01 Lease of Unit 4');
    expect(description()).toHaveValue('A lease of Unit 4.');
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(calls.onApprove).toHaveBeenLastCalledWith('2024-03-01 Lease of Unit 4.pdf', 'A lease of Unit 4.');

    // Read again, it is a new proposal: shown as it is, not under the edit of the old one.
    view.rerender(<ReviewInspector item={invoice} drawer={false} {...calls} />);
    view.rerender(<ReviewInspector item={{ ...lease, proposedFilename: '2024-03-02 Lease Agreement.pdf', description: 'A lease agreement.', proposalRevision: '2' }} drawer={false} {...calls} />);
    expect(filename()).toHaveValue('2024-03-02 Lease Agreement');
    expect(description()).toHaveValue('A lease agreement.');
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(calls.onApprove).toHaveBeenLastCalledWith('2024-03-02 Lease Agreement.pdf', 'A lease agreement.');
  });

  it('traps keyboard focus in settings and restores it to the invoking control', async () => {
    render(<App bridge={createInMemoryBridge()} selection={folderPicker('C:\\Scans')} />);
    const trigger = (await screen.findAllByRole('button', { name: 'Settings' }))[0];
    fireEvent.click(trigger);

    const change = await screen.findByRole('button', { name: 'Change Queue folder' });
    await waitFor(() => expect(change).toHaveFocus());
    // Tabbing off the LAST control must wrap to the first. Found dynamically
    // rather than named, because this used to hard-code "Save settings" and
    // broke the moment the dialog grew an Updates section - which tested the
    // button order, not the trap.
    const dialog = screen.getByRole('dialog', { name: 'Settings' });
    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled])'));
    expect(focusable.length).toBeGreaterThan(2);
    focusable[focusable.length - 1].focus();
    fireEvent.keyDown(document, { key: 'Tab' });
    expect(screen.getByRole('button', { name: 'Close settings' })).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Escape' });

    expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  // 1100px and below is where the inspector becomes a modal drawer, and 1024
  // is the narrowest window Intern supports. Both tests below drive that width.
  const stubNarrowWindow = () => vi.stubGlobal('matchMedia', vi.fn(() => ({
    matches: true,
    media: '(max-width: 1100px)',
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })));

  // Intern selects the first item needing review at launch. Narrow, that
  // selection used to open the modal drawer: Intern started inside a dialog
  // nobody had asked for, over an inert queue, with the caret already in a
  // proposed filename. Now nothing but the Details button opens the panel,
  // and narrow, not even a panel left open on a wide window last time.
  it('does not launch a narrow window inside a modal nobody opened', async () => {
    window.localStorage.setItem('intern.detailsPanel', 'open');
    stubNarrowWindow();
    render(<App bridge={createInMemoryBridge()} />);

    await waitFor(() => expect(queueCell('Lease Agreement - 123 Main St.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
    expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument();
    expect(screen.getByRole('banner')).not.toHaveAttribute('inert');
    expect(screen.getByRole('navigation', { name: 'Queue navigation' })).not.toHaveAttribute('inert');
    expect(detailsToggle()).toHaveAttribute('aria-expanded', 'false');

    // Selecting is only selecting.
    selectByName('NDA - Acme Corp.docx');
    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();

    // The Details button opens the modal drawer.
    fireEvent.click(detailsToggle());
    expect(await screen.findByRole('dialog', { name: 'Review item' })).toHaveAttribute('aria-modal', 'true');
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
    expect(screen.getByLabelText('Filename')).toHaveValue('2024-03-01 Non-Disclosure Agreement with Acme Corp');
  });

  it('turns the narrow inspector into a contained drawer and restores focus to the current cell', async () => {
    stubNarrowWindow();
    render(<App bridge={createInMemoryBridge()} />);

    await screen.findByRole('grid', { name: 'Queue' });
    selectByName('Lease Agreement - 123 Main St.pdf');
    const toggle = detailsToggle();
    fireEvent.click(toggle);
    const initialDrawer = await screen.findByRole('dialog', { name: 'Review item' });
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(toggle).toHaveAttribute('aria-controls', initialDrawer.id);
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
    expect(screen.getByRole('banner')).toHaveAttribute('inert');
    // The view switch is part of the queue the drawer makes inert.
    expect(screen.getByRole('navigation', { name: 'Queue navigation' }).closest('[inert]')).not.toBeNull();

    const lastAction = screen.getByRole('button', { name: 'More review actions' });
    lastAction.focus();
    fireEvent.keyDown(initialDrawer, { key: 'Tab' });
    expect(screen.getByRole('button', { name: 'Close review' })).toHaveFocus();
    fireEvent.keyDown(initialDrawer, { key: 'Escape' });

    // Closed: back to the row being worked on, which is still selected.
    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
    await waitFor(() => expect(queueCell('Lease Agreement - 123 Main St.pdf')).toHaveFocus());
    expect(detailsToggle()).toHaveAttribute('aria-expanded', 'false');
    expect(queueCell('Lease Agreement - 123 Main St.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');

    // Enter on a cell opens it too. J in the drawer moves to the next row,
    // and closing goes back to that row's cell - the current one, not the
    // cell the drawer was opened from.
    const cell = queueCell('Lease Agreement - 123 Main St.pdf');
    cell.focus();
    fireEvent.keyDown(cell, { key: 'Enter' });
    const reopenedDrawer = await screen.findByRole('dialog', { name: 'Review item' });
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
    const heading = within(reopenedDrawer).getByRole('heading', { name: 'Review item' });
    heading.focus();
    fireEvent.keyDown(heading, { key: 'j' });
    await waitFor(() => expect(reopenedDrawer).toHaveTextContent('NDA - Acme Corp.docx'));
    fireEvent.click(within(reopenedDrawer).getByRole('button', { name: 'Close review' }));

    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
    await waitFor(() => expect(queueCell('NDA - Acme Corp.docx')).toHaveFocus());
  });

  // The panel's More menu closed only by its own button or by taking one of
  // its actions. Escape did nothing beside the queue, and in the narrow
  // drawer closed the whole drawer from under the open menu.
  it('closes the More menu on Escape with focus back on its button, and on a press outside it', async () => {
    const bridge = createInMemoryBridge();
    const reanalyze = vi.spyOn(bridge, 'reanalyze');
    render(<App bridge={bridge} />);
    const panel = await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const more = within(panel).getByRole('button', { name: 'More review actions' });
    const menu = () => within(panel).queryByRole('group', { name: 'More review actions' });

    fireEvent.click(more);
    expect(more).toHaveAttribute('aria-expanded', 'true');
    const again = within(menu()!).getByRole('button', { name: 'Analyze again' });
    again.focus();
    // Handled here: nothing behind the panel hears this Escape as well.
    expect(fireEvent.keyDown(again, { key: 'Escape' })).toBe(false);
    expect(menu()).not.toBeInTheDocument();
    expect(more).toHaveAttribute('aria-expanded', 'false');
    expect(more).toHaveFocus();
    // The panel is beside the queue and stays; a second Escape has nothing to close.
    fireEvent.keyDown(more, { key: 'Escape' });
    expect(panel).toBeInTheDocument();

    // From elsewhere in the panel the menu closes and focus stays where it is.
    fireEvent.click(more);
    const filename = within(panel).getByLabelText('Filename');
    filename.focus();
    fireEvent.keyDown(filename, { key: 'Escape' });
    expect(menu()).not.toBeInTheDocument();
    expect(filename).toHaveFocus();
    expect(filename).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');

    // A press inside the menu, or on its own button, is not a press outside it.
    fireEvent.click(more);
    fireEvent.pointerDown(within(menu()!).getByRole('button', { name: 'Remove from queue' }));
    expect(menu()).toBeInTheDocument();
    fireEvent.pointerDown(more);
    expect(menu()).toBeInTheDocument();
    // One anywhere else is: in the panel, or in the queue.
    fireEvent.pointerDown(within(panel).getByRole('heading', { name: 'Review item' }));
    expect(menu()).not.toBeInTheDocument();
    fireEvent.click(more);
    fireEvent.pointerDown(queueCell('NDA - Acme Corp.docx'));
    expect(menu()).not.toBeInTheDocument();
    expect(more).toHaveAttribute('aria-expanded', 'false');

    // Its actions still work, and still close it.
    fireEvent.click(more);
    fireEvent.click(within(menu()!).getByRole('button', { name: 'Analyze again' }));
    await waitFor(() => expect(reanalyze).toHaveBeenCalledWith('lease'));
  });

  it('closes the More menu before the narrow drawer: Escape once for the menu, again for the drawer', async () => {
    stubNarrowWindow();
    render(<App bridge={createInMemoryBridge()} />);
    await screen.findByRole('grid', { name: 'Queue' });
    selectByName('Lease Agreement - 123 Main St.pdf');
    fireEvent.click(detailsToggle());
    const drawer = await screen.findByRole('dialog', { name: 'Review item' });
    const more = within(drawer).getByRole('button', { name: 'More review actions' });

    fireEvent.click(more);
    const remove = within(drawer).getByRole('button', { name: 'Remove from queue' });
    remove.focus();
    fireEvent.keyDown(remove, { key: 'Escape' });

    // The menu went; the drawer, and the document in it, did not.
    expect(within(drawer).queryByRole('group', { name: 'More review actions' })).not.toBeInTheDocument();
    expect(screen.getByRole('dialog', { name: 'Review item' })).toBe(drawer);
    expect(more).toHaveFocus();
    expect(screen.getByRole('banner')).toHaveAttribute('inert');

    fireEvent.keyDown(more, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
    await waitFor(() => expect(queueCell('Lease Agreement - 123 Main St.pdf')).toHaveFocus());
  });

  // The menu is drawn only while the item can be kept. One that stopped
  // being keepable under an open menu - a refresh showing it read again -
  // took the menu off screen and left it open: the next Escape closed
  // nothing, and it took a second to close the drawer.
  it('closes the drawer at the first Escape once the More menu has gone with the item\'s actions', () => {
    const calls = { onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(), onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn() };
    const lease: QueueItem = { id: 'lease', originalFilename: 'scan.pdf', status: 'review', proposedFilename: '2024-03-01 Lease.pdf', description: 'A lease.', confidence: 0.8, proposalRevision: '1', reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' };
    const view = render(<ReviewInspector item={lease} drawer {...calls} />);
    fireEvent.click(screen.getByRole('button', { name: 'More review actions' }));
    expect(screen.getByRole('group', { name: 'More review actions' })).toBeInTheDocument();

    view.rerender(<ReviewInspector item={{ ...lease, status: 'processing', stage: 'naming', reason: undefined, errorCode: undefined }} drawer {...calls} />);
    expect(screen.queryByRole('group', { name: 'More review actions' })).not.toBeInTheDocument();
    fireEvent.keyDown(screen.getByRole('heading', { name: 'Review item' }), { key: 'Escape' });
    expect(calls.onClose).toHaveBeenCalledOnce();

    // Back in review, the menu stays closed until it is opened again.
    view.rerender(<ReviewInspector item={lease} drawer {...calls} />);
    expect(screen.getByRole('button', { name: 'More review actions' })).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('group', { name: 'More review actions' })).not.toBeInTheDocument();
  });

  // The selected document can leave the queue by itself: its file taken
  // from the Queue folder, or filed by another machine. The focus went to
  // Rename all ready, the toolbar's primary action, and the space bar typed
  // next renamed every ready document. It goes to the button of the view
  // already shown, which changes nothing; not to another row's cell either,
  // where Enter opens that document's details at its name.
  //
  // `from`: where the focus was when the row went. On a button of the panel,
  // the queue takes it back, to the view's button. In the document's
  // description - a person in the middle of a sentence - it does not reach a
  // button at all: what was typed is kept above the queue with the caret in
  // it (see typingKept.test.tsx), and the queue leaves it there.
  for (const from of ['a button of the panel', 'the description being typed'] as const) {
    it(`moves focus to the shown view's own button, never to one that acts, when refresh removes the invoking row: from ${from}`, async () => {
      const base = createInMemoryBridge();
      const bridge = { ...base, approve: vi.fn(base.approve) };
      render(<App bridge={bridge} />);
      await screen.findByRole('grid', { name: 'Queue' });
      await openDetails('Lease Agreement - 123 Main St.pdf');
      const description = screen.getByLabelText('Description');
      fireEvent.change(description, { target: { value: 'Commercial lease, signed' } });
      (from === 'the description being typed' ? description : screen.getByRole('button', { name: /Approve & rename/ })).focus();
      await base.remove('lease');
      fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

      await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
      // A visible queue control, as before.
      const shown = screen.getByRole('button', { name: /^Queue, \d+$/ });
      const holds = from === 'the description being typed' ? await screen.findByLabelText('What you had typed') : shown;
      await waitFor(() => expect(holds).toHaveFocus());
      if (holds !== shown) {
        expect(holds).toHaveAttribute('readonly');
        expect(holds).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc\n\nCommercial lease, signed');
      }
      expect(shown).toHaveAttribute('aria-current', 'page');
      expect(screen.getByRole('button', { name: 'Rename all ready' })).toBeEnabled();
      // The rest of the sentence and a new paragraph, typed where the focus
      // now is, select nothing, open nothing and file nothing.
      for (const key of [' ', 'b', 'y', ' ', 'Enter', 'Enter']) {
        fireEvent.keyDown(document.activeElement!, { key });
        if (key === ' ' || key === 'Enter') fireEvent.click(document.activeElement!);
        fireEvent.keyUp(document.activeElement!, { key });
      }
      await new Promise((resolve) => setTimeout(resolve, 0));
      expect(holds).toHaveFocus();
      expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument();
      expect(screen.queryByRole('row', { current: true })).not.toBeInTheDocument();
      expect(bridge.approve).not.toHaveBeenCalled();
      expect((await base.listItems()).filter((item) => item.status === 'ready')).toHaveLength(3);
    });
  }

  // Whichever view is shown: its own button, not whichever button comes
  // first in the queue's panel.
  it('moves focus to the Completed view\'s button when a filed document leaves it', async () => {
    const base = createInMemoryBridge({ items: [
      { id: 'lease', originalFilename: 'Lease.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement.pdf', confidence: 0.72, description: 'A lease.' },
      { id: 'filed', originalFilename: 'Filed.pdf', status: 'completed', proposedFilename: '2024-01-22 Lease Agreement.pdf', filedName: '2024-01-22 Lease Agreement.pdf', confidence: 0.93 },
      { id: 'filed-2', originalFilename: 'Filed 2.pdf', status: 'completed', proposedFilename: '2024-01-23 Lease Agreement.pdf', filedName: '2024-01-23 Lease Agreement.pdf', confidence: 0.93 },
    ] });
    render(<App bridge={base} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Completed, 2' }));
    await openDetails('Filed.pdf');
    await base.remove('filed');
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
    const shown = screen.getByRole('button', { name: 'Completed, 1' });
    await waitFor(() => expect(shown).toHaveFocus());
    expect(shown).toHaveAttribute('aria-current', 'page');
    // The row still in the view did not take it.
    expect(queueCell('Filed 2.pdf')).not.toHaveFocus();
  });

  // With nothing left in the queue there is no view button, only the drop
  // target and its Choose buttons: the focus goes to none of them. From a
  // button of the panel it goes nowhere; a caret in the description stays
  // in what was typed, kept above the drop target.
  for (const from of ['a button of the panel', 'the description being typed'] as const) {
    it(`moves focus to no button when the only document leaves the queue: from ${from}`, async () => {
      const base = createInMemoryBridge({ items: [{ id: 'lease', originalFilename: 'Lease.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement.pdf', confidence: 0.72, description: 'A lease.' }] });
      render(<App bridge={base} selection={folderPicker('C:\\Scans')} />);
      await screen.findByRole('grid', { name: 'Queue' });
      await openDetails('Lease.pdf');
      (from === 'the description being typed' ? screen.getByLabelText('Description') : screen.getByRole('button', { name: /Approve & rename/ })).focus();
      await base.remove('lease');
      fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

      await waitFor(() => expect(screen.queryByRole('grid', { name: 'Queue' })).not.toBeInTheDocument());
      const queue = screen.getByRole('region', { name: 'Queue items' });
      expect(within(queue).getAllByRole('button').length).toBeGreaterThan(0);
      await new Promise((resolve) => setTimeout(resolve, 50));
      if (from === 'a button of the panel') expect(document.activeElement).toBe(document.body);
      else {
        expect(screen.getByLabelText('What you had typed')).toHaveFocus();
        expect(screen.getByLabelText('What you had typed')).toHaveValue('2023-09-15 Lease Agreement\n\nA lease.');
      }
      for (const button of within(queue).getAllByRole('button')) expect(button).not.toHaveFocus();
    });
  }

  // That rule is for a row that went by itself. A document the person has
  // just removed from the keyboard - Delete, and Enter on the question -
  // leaves them in the queue, and the focus used to land on a cell, where the
  // arrow keys go on; under the rule above it landed on the view's button,
  // where they move nothing. The queue says it changed while the command is
  // still answering, as the desktop backend's does.
  it('moves focus to the table\'s own cell, not the view\'s button, when the person removes the document from the keyboard', async () => {
    const base = createInMemoryBridge({ liveEvents: true }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    let queueChanged = () => {};
    const bridge = {
      ...base,
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
      remove: vi.fn(async (id: string) => { await base.remove(id); queueChanged(); await new Promise((resolve) => setTimeout(resolve, 20)); }),
      approve: vi.fn(base.approve),
    };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await openDetails('Lease Agreement - 123 Main St.pdf');
    screen.getByRole('heading', { name: 'Review item' }).focus();
    fireEvent.keyDown(document.activeElement!, { key: 'Delete' });
    const question = await screen.findByRole('group', { name: 'Confirm removal' });
    fireEvent.click(within(question).getByRole('button', { name: 'Remove from queue' }));
    await waitFor(() => expect(bridge.remove).toHaveBeenCalledWith('lease'));

    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
    // The cell the table keeps for Tab: with nothing selected, the first row's.
    const cell = queueCell('Employment Agreement - John Smith.pdf');
    await waitFor(() => expect(cell).toHaveFocus());
    expect(cell).toHaveAttribute('tabindex', '0');
    expect(screen.getByRole('button', { name: /^Queue, \d+$/ })).not.toHaveFocus();
    expect(screen.getByRole('button', { name: 'Rename all ready' })).not.toHaveFocus();
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(cell).toHaveFocus();
    expect(bridge.approve).not.toHaveBeenCalled();
  });

  // The same after the last decision there is to make: the row has left the
  // view by the person's own approval, and nothing is left to move on to.
  it('moves focus to the table\'s own cell when the last undecided document is approved and leaves the view', async () => {
    const base = createInMemoryBridge({ items: [
      { id: 'waiting', originalFilename: 'Still waiting.pdf', status: 'waiting' },
      { id: 'lease', originalFilename: 'Lease.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement.pdf', confidence: 0.72, description: 'A lease.' },
    ] });
    const bridge = { ...base, approve: vi.fn(base.approve) };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await openDetails('Lease.pdf');
    fireEvent.keyDown(screen.getByLabelText('Filename'), { key: 'Enter', ctrlKey: true });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('lease', '2023-09-15 Lease Agreement.pdf', 'A lease.'));
    await waitFor(() => expect(screen.queryByRole('gridcell', { name: 'Lease.pdf' })).not.toBeInTheDocument());
    const cell = queueCell('Still waiting.pdf');
    await waitFor(() => expect(cell).toHaveFocus());
    expect(screen.getByRole('button', { name: /^Queue, \d+$/ })).not.toHaveFocus();
  });

  it('saves the automatic rename setting', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByLabelText('Automatically rename ready documents'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ automaticRename: true })));
  });

  it('saves the shared intake configuration', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} selection={folderPicker('C:\\Scans')} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByRole('button', { name: 'Change Queue folder' }));
    expect(await screen.findByText('C:\\Scans')).toBeVisible();
    fireEvent.click(screen.getByLabelText('Also process documents uploaded by others'));
    fireEvent.change(screen.getByLabelText("This machine's name"), { target: { value: 'Front desk' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      intakeEnabled: true,
      intakeFolder: 'C:\\Scans',
      processOthersUploads: true,
      machineLabel: 'Front desk',
    })));
  });

  it('saves run in background with start at login, gating start minimized on the tray', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    // Minimized-without-tray would strand the app invisibly, so the checkbox
    // only unlocks once background mode is on.
    expect(await screen.findByLabelText('Start minimized')).toBeDisabled();
    fireEvent.click(screen.getByLabelText('Run in background'));
    fireEvent.click(screen.getByLabelText('Start ATLAS when you sign in'));
    fireEvent.click(screen.getByLabelText('Start minimized'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      runInBackground: true,
      startAtLogin: true,
      startMinimized: true,
    })));
  });

  it('changes the output and Queue folders with the folder picker', async () => {
    const pickFolder = vi.fn()
      .mockResolvedValueOnce({ path: 'C:\\Filed', displayName: 'Filed' })
      .mockResolvedValueOnce({ path: 'C:\\Scans', displayName: 'Scans' });
    render(<App bridge={createInMemoryBridge()} selection={{ pickFiles: async () => [], pickFolder, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByRole('button', { name: 'Change Completed folder' }));
    expect(await screen.findByText('C:\\Filed')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Change Queue folder' }));
    expect(await screen.findByText('C:\\Scans')).toBeVisible();

    expect(pickFolder).toHaveBeenCalledTimes(2);
  });

  it('shows a cloud badge for a OneDrive Queue folder', async () => {
    render(<App bridge={createInMemoryBridge()} selection={folderPicker('C:\\Users\\pat\\OneDrive\\Scans')} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByRole('button', { name: 'Change Queue folder' }));

    expect(await screen.findByText('Synced with OneDrive – Contoso')).toBeVisible();
  });

  it('saves the destination layout and explains where a document will land', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByText('How ATLAS names files', { selector: 'summary' }));
    const layout = await screen.findByLabelText('Arrange filed documents');
    expect(layout).toHaveValue('flat');
    fireEvent.change(layout, { target: { value: 'year_type' } });
    expect(screen.getByText(/2026\\Statement of Work\\/)).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ destinationLayout: 'year_type' })));
  });

  it('offers the synced locations the sync client keeps and fills the folders from them', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByText('Shared folders and troubleshooting', { selector: 'summary' }));
    const locations = await screen.findByRole('group', { name: 'Synced locations on this computer' });
    expect(locations).toHaveTextContent('SharePoint – Contoso');
    expect(locations).toHaveTextContent('C:\\Users\\pat\\Contoso\\Legal - Documents');
    fireEvent.click(screen.getByRole('button', { name: 'Use SharePoint – Contoso as the Completed folder' }));
    const folders = screen.getByRole('group', { name: 'Folders' });
    expect(await within(folders).findByText('C:\\Users\\pat\\Contoso\\Legal - Documents')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Use OneDrive – Contoso as the Queue folder' }));
    expect(await within(folders).findByText('C:\\Users\\pat\\OneDrive - Contoso')).toBeVisible();

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      destination: 'C:\\Users\\pat\\Contoso\\Legal - Documents',
      intakeFolder: 'C:\\Users\\pat\\OneDrive - Contoso',
      intakeMyFolder: true,
      // An Completed folder in a SharePoint library records descriptions for its column.
      recordDescriptions: true,
    })));
  });

  it('labels a network share by its share name', async () => {
    render(<App bridge={createInMemoryBridge()} selection={folderPicker('\\\\fileserver\\legal\\filed')} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByRole('button', { name: 'Change Completed folder' }));

    expect(await screen.findByText('On a network share – \\\\fileserver\\legal')).toBeVisible();
  });

  it('saves the description records setting and reports the records folder', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByText('SharePoint descriptions', { selector: 'summary' }));
    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('Off');
    // Not yet saved on, so the backfill cannot be offered.
    expect(screen.getByRole('button', { name: 'Write records for documents already filed' })).toBeDisabled();
    fireEvent.click(screen.getByLabelText('Write a description record for each filed document'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      recordDescriptions: true,
      destination: 'C:\\Intern\\Output',
    })));
  });

  it('writes records for documents already filed once the setting is saved on', async () => {
    const bridge = createInMemoryBridge();
    await bridge.saveSettings({ destination: 'C:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'C:\\Scans', intakeEnabled: true, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: true, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' });
    render(<App bridge={bridge} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByText('SharePoint descriptions', { selector: 'summary' }));
    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('On');
    expect(status).toHaveTextContent('C:\\Filed\\.intern\\descriptions');
    const backfill = await screen.findByRole('button', { name: 'Write records for documents already filed' });
    await waitFor(() => expect(backfill).toBeEnabled());
    fireEvent.click(backfill);

    // The seeded queue has exactly one completed item that still carries its sentence.
    expect(await screen.findByRole('status', { name: 'Backfill status' })).toHaveTextContent('1 record written.');
    await waitFor(() => expect(status).toHaveTextContent('1 record written since ATLAS started'));
  });

  it('explains a records-without-destination refusal in plain words', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(async () => { throw { code: 'DESCRIPTIONS_NEED_DESTINATION', message: 'description records need a destination folder to live in' }; });
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    fireEvent.click(await screen.findByLabelText('Write a description record for each filed document'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/choose a Completed folder before turning them on.*\(DESCRIPTIONS_NEED_DESTINATION\)/i);
  });

  it('renders the live intake status with the held-for-others count', async () => {
    const bridge = createInMemoryBridge();
    await bridge.saveSettings({ destination: 'C:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'C:\\Scans', intakeEnabled: true, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' });
    render(<App bridge={bridge} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByText('Shared folders and troubleshooting', { selector: 'summary' }));
    const status = await screen.findByRole('status', { name: 'Intake status' });
    await waitFor(() => expect(status).toHaveTextContent('Watching'));
    expect(status).toHaveTextContent('2 machines active');
    expect(status).toHaveTextContent('2 held for others');
    expect(screen.getByRole('button', { name: 'Scan now' })).toBeEnabled();
  });

  it('surfaces intake validation codes from save as plain sentences', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(async () => { throw { code: 'DESTINATION_INSIDE_INTAKE', message: 'destination is inside the intake folder' }; });
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/Completed folder is inside the Queue folder.*\(DESTINATION_INSIDE_INTAKE\)/i);
    // The dialog stays open so the folders can be corrected.
    expect(screen.getByRole('dialog', { name: 'Settings' })).toBeInTheDocument();
  });

  // Both affordances go through the bridge rather than a link, because inside
  // Tauri a <a target="_blank"> has nowhere to open.
  it('opens the published guide from the chrome and from Settings', async () => {
    const openGuide = vi.fn(async () => undefined);
    render(<App bridge={{ ...createInMemoryBridge(), openGuide }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Help & support' }));
    await waitFor(() => expect(openGuide).toHaveBeenCalledTimes(1));

    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    fireEvent.click(await screen.findByRole('button', { name: 'Open the guide' }));

    await waitFor(() => expect(openGuide).toHaveBeenCalledTimes(2));
  });

  it('names the guide address when the browser hand-off is refused', async () => {
    const openGuide = vi.fn(async () => { throw new Error('opener denied'); });
    render(<App bridge={{ ...createInMemoryBridge(), openGuide }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Help & support' }));

    expect(await screen.findByRole('alert', { name: 'Action error' }))
      .toHaveTextContent('https://z-brenner.github.io/intern/guide.html');
  });

  // Keeping the original name renames nothing, so the backend has no rename
  // to undo and refuses one. The in-memory bridge used to offer it anyway.
  it('moves Keep original to Completed without offering an undo the backend would refuse', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: /Keep original/i }));
    fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));

    expect(screen.queryByRole('button', { name: 'Undo' })).not.toBeInTheDocument();
    expect((await bridge.listItems()).find((item) => item.id === 'lease')).toMatchObject({
      status: 'completed',
      keptOriginal: true,
      undoable: false,
      // The proposal stays, unapplied, as the backend leaves it.
      proposedFilename: '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf',
    });
    await expect(bridge.undo('lease')).rejects.toMatchObject({ code: 'STATE_CONFLICT' });
  });
});
