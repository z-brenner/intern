import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

describe('review actions', () => {
  afterEach(() => vi.unstubAllGlobals());
  const selectRow = (row: HTMLElement) => fireEvent.click(within(row).getByRole('button', { name: /select/i }));
  it('keeps editing local until a nonblank filename is approved', async () => {
    const baseBridge = createInMemoryBridge();
    const approve = vi.fn(baseBridge.approve);
    const bridge = { ...baseBridge, approve };
    render(<App bridge={bridge} />);
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));

    fireEvent.change(screen.getByLabelText('Filename'), { target: { value: '2023-09-15 Lease.PDF' } });
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/i }));

    expect(screen.getByRole('alert')).toHaveTextContent('Leave \u201c.pdf\u201d off: Intern adds the extension itself.');
    expect(approve).not.toHaveBeenCalled();
  });

  // The backstop for a rule the window does not know: the backend's own
  // refusal, in words, and a way to put it away.
  it('explains a filename the backend refused in words, and can be dismissed', async () => {
    const approve = vi.fn(async () => { throw { code: 'NAME_INVALID', message: 'filename must be one nonblank path component' }; });
    render(<App bridge={{ ...createInMemoryBridge(), approve }} />);
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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

    selectRow(screen.getByRole('row', { name: /Completed lease\.pdf/ }));
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

    selectRow(screen.getByRole('row', { name: /Board minutes\.docx/ }));
    await waitFor(() => expect(inspector).toHaveTextContent('Kept its original name'));
    expect(inspector).not.toHaveTextContent('Renamed to');
    expect(within(inspector).queryByRole('button', { name: 'Undo' })).not.toBeInTheDocument();

    // A settled item's reason is a note about what happened, not a reason it waits.
    selectRow(screen.getByRole('row', { name: /2024-02-01 Invoice\.pdf/ }));
    await waitFor(() => expect(within(inspector).getByRole('heading', { name: 'Note' })).toBeVisible());
    expect(within(inspector).queryByRole('heading', { name: 'Reason for review' })).not.toBeInTheDocument();
  });

  it('says plainly when the document is no longer where Intern saw it', async () => {
    const openItem = vi.fn(async () => { throw { code: 'PATH_UNAVAILABLE', message: 'the document is not where Intern last saw it' }; });
    render(<App bridge={{ ...createInMemoryBridge(), openItem }} />);
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Open' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('It may have been moved, renamed, or deleted outside Intern.');
    // Opening is not a decision: the item is still there to decide.
    expect(screen.getByRole('button', { name: /Approve & rename/i })).toBeEnabled();
  });

  it('sends an ordinary review item back to be analyzed again, and it returns for review', async () => {
    const bridge = createInMemoryBridge({ liveEvents: true, analysisDelayMs: 20 });
    const reanalyze = vi.spyOn(bridge, 'reanalyze');
    const retry = vi.spyOn(bridge, 'retry');
    render(<App bridge={bridge} />);
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
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
    selectRow(await screen.findByRole('row', { name: /Completed lease.pdf/i }));

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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const filename = screen.getByLabelText('Filename');
    fireEvent.change(filename, { target: { value: 'My local draft' } });
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    await waitFor(() => expect(filename).toHaveValue('My local draft'));
  });

  it('traps keyboard focus in settings and restores it to the invoking control', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const trigger = (await screen.findAllByRole('button', { name: 'Settings' }))[0];
    fireEvent.click(trigger);

    const destination = await screen.findByLabelText('Destination folder');
    await waitFor(() => expect(destination).toHaveFocus());
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

  // The panel is seeded with the first item needing review so it is not empty
  // on launch. Narrow, that seeding used to open the modal drawer: Intern
  // started inside a dialog nobody had asked for, over an inert queue, with
  // the caret already in a proposed filename.
  it('does not launch a narrow window inside a modal nobody opened', async () => {
    stubNarrowWindow();
    render(<App bridge={createInMemoryBridge()} />);

    const seeded = await screen.findByRole('complementary', { name: 'Review item' });
    expect(seeded).not.toHaveAttribute('aria-modal');
    expect(screen.getByRole('banner')).not.toHaveAttribute('inert');
    expect(screen.getByRole('navigation', { name: 'Queue navigation' })).not.toHaveAttribute('inert');
    expect(screen.getByLabelText('Filename')).not.toHaveFocus();

    // A selection a person makes is still the modal drawer it was.
    fireEvent.click(screen.getByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' }));

    expect(await screen.findByRole('dialog', { name: 'Review item' })).toHaveAttribute('aria-modal', 'true');
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
  });

  it('turns the narrow inspector into a contained drawer and restores row focus', async () => {
    stubNarrowWindow();
    render(<App bridge={createInMemoryBridge()} />);

    const trigger = await screen.findByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' });
    fireEvent.click(trigger);
    const initialDrawer = await screen.findByRole('dialog', { name: 'Review item' });
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
    expect(screen.getByRole('banner')).toHaveAttribute('inert');
    expect(screen.getByRole('navigation', { name: 'Queue navigation' })).toHaveAttribute('inert');

    const lastAction = screen.getByRole('button', { name: 'More review actions' });
    lastAction.focus();
    fireEvent.keyDown(initialDrawer, { key: 'Tab' });
    expect(screen.getByRole('button', { name: 'Close review' })).toHaveFocus();
    fireEvent.keyDown(initialDrawer, { key: 'Escape' });

    fireEvent.click(trigger);
    const reopenedDrawer = await screen.findByRole('dialog', { name: 'Review item' });
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveFocus());
    fireEvent.keyDown(reopenedDrawer, { key: 'Escape' });

    expect(screen.queryByRole('dialog', { name: 'Review item' })).not.toBeInTheDocument();
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it('moves focus to a visible queue control when refresh removes the invoking row', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);
    const trigger = await screen.findByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' });
    fireEvent.click(trigger);
    await bridge.remove('lease');
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());
    await waitFor(() => expect(screen.getByRole('button', { name: 'Apply all ready' })).toHaveFocus());
  });

  it('saves the automatic high-confidence rename setting', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByLabelText('Automatically rename high-confidence files'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ automaticRename: true })));
  });

  it('saves the shared intake configuration', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByLabelText('Watch a folder for new documents'));
    fireEvent.change(screen.getByLabelText('Intake folder'), { target: { value: 'C:\\Scans' } });
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
    fireEvent.click(screen.getByLabelText('Start Intern when you sign in'));
    fireEvent.click(screen.getByLabelText('Start minimized'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      runInBackground: true,
      startAtLogin: true,
      startMinimized: true,
    })));
  });

  it('browses for the destination and intake folders at the selection boundary', async () => {
    const pickFolder = vi.fn()
      .mockResolvedValueOnce({ path: 'C:\\Filed', displayName: 'Filed' })
      .mockResolvedValueOnce({ path: 'C:\\Scans', displayName: 'Scans' });
    render(<App bridge={createInMemoryBridge()} selection={{ pickFiles: async () => [], pickFolder, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.click(await screen.findByRole('button', { name: 'Browse for destination folder' }));
    await waitFor(() => expect(screen.getByLabelText('Destination folder')).toHaveValue('C:\\Filed'));
    fireEvent.click(screen.getByRole('button', { name: 'Browse for intake folder' }));
    await waitFor(() => expect(screen.getByLabelText('Intake folder')).toHaveValue('C:\\Scans'));

    expect(pickFolder).toHaveBeenCalledTimes(2);
  });

  it('shows a cloud badge for a OneDrive intake folder', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.change(await screen.findByLabelText('Intake folder'), { target: { value: 'C:\\Users\\pat\\OneDrive\\Scans' } });

    expect(await screen.findByText('Synced with OneDrive – Contoso')).toBeVisible();
  });

  it('saves the destination layout and explains where a document will land', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

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

    const locations = await screen.findByRole('group', { name: 'Synced locations on this computer' });
    expect(locations).toHaveTextContent('SharePoint – Contoso');
    expect(locations).toHaveTextContent('C:\\Users\\pat\\Contoso\\Legal - Documents');
    fireEvent.click(screen.getByRole('button', { name: 'Use SharePoint – Contoso as the destination folder' }));
    fireEvent.click(screen.getByRole('button', { name: 'Use OneDrive – Contoso as the intake folder' }));

    expect(screen.getByLabelText('Destination folder')).toHaveValue('C:\\Users\\pat\\Contoso\\Legal - Documents');
    expect(screen.getByLabelText('Intake folder')).toHaveValue('C:\\Users\\pat\\OneDrive - Contoso');
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      destination: 'C:\\Users\\pat\\Contoso\\Legal - Documents',
      intakeFolder: 'C:\\Users\\pat\\OneDrive - Contoso',
    })));
  });

  it('labels a network share by its share name', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    fireEvent.change(await screen.findByLabelText('Destination folder'), { target: { value: '\\\\fileserver\\legal\\filed' } });

    expect(await screen.findByText('On a network share – \\\\fileserver\\legal')).toBeVisible();
  });

  it('saves the description records setting and reports the records folder', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('Off');
    // Not yet saved on, so the backfill cannot be offered.
    expect(screen.getByRole('button', { name: 'Write records for documents already filed' })).toBeDisabled();
    fireEvent.change(screen.getByLabelText('Destination folder'), { target: { value: 'C:\\Filed' } });
    fireEvent.click(screen.getByLabelText('Write a description record for each filed document'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({
      recordDescriptions: true,
      destination: 'C:\\Filed',
    })));
  });

  it('writes records for documents already filed once the setting is saved on', async () => {
    const bridge = createInMemoryBridge();
    await bridge.saveSettings({ destination: 'C:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: '', intakeEnabled: false, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: true, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' });
    render(<App bridge={bridge} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('On');
    expect(status).toHaveTextContent('C:\\Filed\\.intern\\descriptions');
    const backfill = await screen.findByRole('button', { name: 'Write records for documents already filed' });
    await waitFor(() => expect(backfill).toBeEnabled());
    fireEvent.click(backfill);

    // The seeded queue has exactly one completed item that still carries its sentence.
    expect(await screen.findByRole('status', { name: 'Backfill status' })).toHaveTextContent('1 record written.');
    await waitFor(() => expect(status).toHaveTextContent('1 record written since Intern started'));
  });

  it('explains a records-without-destination refusal in plain words', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(async () => { throw { code: 'DESCRIPTIONS_NEED_DESTINATION', message: 'description records need a destination folder to live in' }; });
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    fireEvent.click(await screen.findByLabelText('Write a description record for each filed document'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/choose a destination before turning them on.*\(DESCRIPTIONS_NEED_DESTINATION\)/i);
  });

  it('renders the live intake status with the held-for-others count', async () => {
    const bridge = createInMemoryBridge();
    await bridge.saveSettings({ destination: 'C:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'C:\\Scans', intakeEnabled: true, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' });
    render(<App bridge={bridge} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);

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

    expect(await screen.findByRole('alert')).toHaveTextContent(/destination folder is inside the intake folder.*\(DESTINATION_INSIDE_INTAKE\)/i);
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
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    fireEvent.click(screen.getByRole('button', { name: /Keep original/i }));
    fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));
    selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));

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
