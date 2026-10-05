import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SUPPORTED_FORMATS_LABEL } from '../../lib/formats';
import { createFixtureBatchBridge, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { DesktopBridge, LaunchReportSource } from '../../lib/bridge';
import type { AddReport } from '../../types';
import type { QueueBridgeEvent } from '../../lib/tauriBridge';

describe('queue interactions', () => {
  const selectRow = async (row: HTMLElement) => {
    const select = within(row).getByRole('button', { name: /select/i });
    const filename = select.getAttribute('aria-label')?.replace(/^Select /, '');
    fireEvent.click(select);
    const inspector = await screen.findByRole('complementary', { name: 'Review item' });
    if (filename) await waitFor(() => expect(inspector).toHaveTextContent(filename));
  };
  it('keeps identical bytes from different paths separate and deduplicates only the same path', async () => {
    const bridge = createFixtureBatchBridge();

    const report = await bridge.addFiles([
      { path: 'browser://duplicate-invoice-a.pdf', displayName: 'duplicate-invoice-a.pdf' },
      { path: 'browser://duplicate-invoice-b.pdf', displayName: 'duplicate-invoice-b.pdf' },
      { path: 'browser://unsupported.zip', displayName: 'unsupported.zip' },
      { path: 'browser://~$nda.docx', displayName: '~$nda.docx' },
    ]);

    const items = await bridge.listItems();
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-a.pdf')).toMatchObject({ status: 'review', proposedFilename: '2025-04-30 Invoice from Nimbus Orchard Supply Co.pdf' });
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-b.pdf')).toMatchObject({ status: 'review', reason: expect.stringMatching(/different path.*separate/i) });
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-b.pdf')?.id).not.toBe(items.find((item) => item.originalFilename === 'duplicate-invoice-a.pdf')?.id);
    // What the desktop leaves out is reported, not queued as a failed row.
    expect(report).toEqual({ added: 2, alreadyQueued: 0, skipped: [{ name: 'unsupported.zip', code: 'UNSUPPORTED_FORMAT' }, { name: '~$nda.docx', code: 'TEMPORARY_FILE' }] });
    expect(items.map((item) => item.originalFilename)).toEqual(['duplicate-invoice-a.pdf', 'duplicate-invoice-b.pdf']);

    expect(await bridge.addFiles([{ path: 'browser://duplicate-invoice-a.pdf', displayName: 'duplicate-invoice-a.pdf' }])).toEqual({ added: 0, alreadyQueued: 1, skipped: [] });
    expect(await bridge.listItems()).toHaveLength(2);
  });

  // add_report_message_lists_skipped_files (TAURI_SHELL-5): one .zip among
  // twenty-five attachments used to refuse all twenty-five. The rest are
  // queued, and the one is named with its reason - read out, and on screen,
  // because twenty-four rows for twenty-five files otherwise just look like a
  // file went missing.
  it('names the files an add left out, and why, without refusing the rest', async () => {
    const bridge = createInMemoryBridge({ items: [] });
    const attachments = Array.from({ length: 24 }, (_, index) => ({ path: `C:/Inbox/Invoice ${index + 1}.pdf`, displayName: `Invoice ${index + 1}.pdf` }));
    const pickFiles = vi.fn(async () => [...attachments, { path: 'C:/Inbox/notes.zip', displayName: 'notes.zip' }]);
    render(<App bridge={bridge} selection={{ pickFiles, pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add files' }));

    const message = 'Added 24 documents. Skipped 1: notes.zip (not a supported format).';
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent(message));
    expect(screen.getByRole('note', { name: 'Files not added' })).toHaveTextContent(message);
    expect(await screen.findByRole('row', { name: /Invoice 24\.pdf/ })).toBeVisible();
    expect(screen.queryByRole('row', { name: /notes\.zip/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('alert', { name: 'Action error' })).not.toBeInTheDocument();

    fireEvent.click(within(screen.getByRole('note', { name: 'Files not added' })).getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByRole('note', { name: 'Files not added' })).not.toBeInTheDocument();
  });

  it('reports a backend add in the same words, and keeps a clean add off the screen', async () => {
    const base = createInMemoryBridge({ items: [] });
    const reports = [
      { added: 198, alreadyQueued: 0, skipped: [{ name: 'scan-57.pdf', code: 'SOURCE_LOCKED' }, { name: 'empty.pdf', code: 'EMPTY_FILE' }] },
      { added: 1, alreadyQueued: 2, skipped: [] },
    ];
    const addFolder = vi.fn(async () => reports.shift()!);
    const folder = { path: 'C:/Inbox/Scans', displayName: 'Scans' };
    render(<App bridge={{ ...base, addFolder }} selection={{ pickFiles: async () => [], pickFolder: async () => folder, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add folder' }));
    const skipped = 'Added 198 documents. Skipped 2: scan-57.pdf (another program has it open), empty.pdf (the file is empty).';
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent(skipped));
    expect(screen.getByRole('note', { name: 'Files not added' })).toHaveTextContent(skipped);

    fireEvent.click(await screen.findByRole('button', { name: 'Add folder' }));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Added 1 document. 2 were already in the queue.'));
    // Nothing was left out this time, so the earlier notice goes with it.
    expect(screen.queryByRole('note', { name: 'Files not added' })).not.toBeInTheDocument();
  });

  // "Send to > Intern" adds outside the window. What it left out used to be
  // said nowhere at all; it is the same note as an add made here.
  it('names what documents sent to Intern left out', async () => {
    const base = createInMemoryBridge({ items: [] });
    let deliver: ((report: AddReport) => void) | undefined;
    const stop = vi.fn();
    const subscribeLaunchReports = vi.fn((handler: (report: AddReport) => void) => { deliver = handler; return stop; });
    const bridge: DesktopBridge & LaunchReportSource = { ...base, subscribeLaunchReports };
    const view = render(<App bridge={bridge} />);
    await screen.findByRole('main', { name: 'Intern' });
    await waitFor(() => expect(deliver).toBeDefined());

    act(() => deliver!({ added: 1, alreadyQueued: 0, skipped: [{ name: 'Contract.doc', code: 'UNSUPPORTED_FORMAT' }, { name: 'blank.pdf', code: 'EMPTY_FILE' }] }));

    const message = 'Added 1 document. Skipped 2: Contract.doc (not a supported format), blank.pdf (the file is empty).';
    expect(await screen.findByRole('note', { name: 'Files not added' })).toHaveTextContent(message);
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent(message);
    view.unmount();
    expect(stop).toHaveBeenCalled();
  });

  it('focuses the existing result when the same unchanged path is dropped again', async () => {
    const bridge = createFixtureBatchBridge();
    const file = { path: 'browser://duplicate-invoice-a.pdf', displayName: 'duplicate-invoice-a.pdf' };
    const resolveDrop = vi.fn(async () => ({ files: [file] }));
    render(<App bridge={bridge} selection={{ pickFiles: async () => [], pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop }} />);
    const zone = await screen.findByRole('region', { name: /drag files/i });

    fireEvent.drop(zone);
    expect(await screen.findByRole('complementary', { name: 'Review item' })).toHaveTextContent('duplicate-invoice-a.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Close review' }));
    fireEvent.drop(zone);

    expect(await screen.findByRole('complementary', { name: 'Review item' })).toHaveTextContent('duplicate-invoice-a.pdf');
    expect(screen.getAllByRole('button', { name: 'Select duplicate-invoice-a.pdf' })).toHaveLength(1);
  });

  it('describes supported formats without adding a nonfunctional keyboard stop', async () => {
    render(<App bridge={createInMemoryBridge()} />);

    const zone = await screen.findByRole('region', { name: /drag files/i });
    expect(zone).toHaveTextContent(`Supports ${SUPPORTED_FORMATS_LABEL}`);
    expect(zone).toHaveTextContent(/PowerPoint/);
    expect(zone).toHaveTextContent(/Outlook \.msg/);
    expect(zone).not.toHaveAttribute('tabindex');
  });

  // Pointing the queue at a large folder used to be unrecoverable from inside
  // the app: pause leaves the backlog, Clear history only touches finished
  // items, and dropping a waiting item was one click each.
  it('discards the waiting backlog without touching work in flight or already done', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);

    const discard = await screen.findByRole('button', { name: 'Discard waiting items' });
    expect(discard).toHaveTextContent('3');

    fireEvent.click(discard);

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Discarded 3 waiting items.'));
    const remaining = await bridge.listItems();
    expect(remaining.some((item) => item.status === 'waiting')).toBe(false);
    // The states that must survive: mid-flight, awaiting a decision, and renamed.
    expect(remaining.some((item) => item.status === 'processing')).toBe(true);
    expect(remaining.some((item) => item.status === 'review')).toBe(true);
    expect(remaining.some((item) => item.status === 'ready')).toBe(true);
    expect(remaining.some((item) => item.status === 'completed')).toBe(true);
    expect(screen.queryByRole('button', { name: 'Discard waiting items' })).not.toBeInTheDocument();
  });

  it('offers no discard action when nothing is waiting', async () => {
    const items = (await createInMemoryBridge().listItems()).filter((item) => item.status !== 'waiting');
    render(<App bridge={createInMemoryBridge({ items })} />);

    await screen.findByRole('button', { name: 'Apply all ready' });
    expect(screen.queryByRole('button', { name: 'Discard waiting items' })).not.toBeInTheDocument();
  });

  it('uses restrained extension-aware document icons', async () => {
    render(<App bridge={createInMemoryBridge()} />);

    expect((await screen.findByRole('row', { name: /Employment Agreement/i })).querySelector('.file-kind--pdf')).toBeInTheDocument();
    expect(screen.getByRole('row', { name: /NDA - Acme Corp/i }).querySelector('.file-kind--document')).toBeInTheDocument();
    expect(screen.getByRole('row', { name: /Q1 Financials/i }).querySelector('.file-kind--pdf')).toBeInTheDocument();
    expect(screen.getByRole('row', { name: /Notes from Call/i }).querySelector('.file-kind--text')).toBeInTheDocument();
    // Every demo row is a format the queue accepts. The spreadsheet icon still
    // exists for a file a user drags in by mistake, but the demo no longer shows
    // an unsupported type sailing through the pipeline.
    expect(screen.queryByRole('row', { name: /\.xlsx/i })).not.toBeInTheDocument();
  });

  it('announces useful queue state changes', async () => {
    render(<App bridge={createInMemoryBridge()} />);

    const status = await screen.findByRole('status', { name: 'Queue status' });
    expect(status).toHaveTextContent('Queue active. 1 processing, 3 ready, 1 needs review, 3 waiting, 1 completed.');
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    await waitFor(() => expect(status).toHaveTextContent('Queue paused. 3 ready, 1 needs review, 4 waiting, 1 completed.'));
  });

  it('applies one ready rename from its contextual inspector action', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(base.approve);
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Apply rename' }));

    await waitFor(() => expect(approve).toHaveBeenCalledWith(
      'employment',
      '2024-04-12 Employment Agreement with John Smith.pdf',
      '',
    ));
  });

  it('applies all ready proposals in one deliberate batch action', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(base.approve);
    render(<App bridge={{ ...base, approve }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));

    await waitFor(() => expect(approve).toHaveBeenCalledTimes(3));
    expect(approve.mock.calls.map(([id]) => id)).toEqual(['employment', 'nda', 'service']);
  });

  it('reports partial Apply all failures while preserving the failed ready item', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(async (id: string, filename: string, description: string) => {
      if (id === 'nda') throw new Error('Destination is locked.');
      await base.approve(id, filename, description);
    });
    render(<App bridge={{ ...base, approve }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('2 renames applied. 1 could not be applied. Destination is locked.');
    expect(screen.getByRole('row', { name: /NDA - Acme Corp/i })).toHaveTextContent('Ready');
    expect(approve).toHaveBeenCalledTimes(3);
  });

  it('shows a command failure and keeps the selected item available to retry', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(async () => { throw new Error('Destination is unavailable.'); });
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Apply rename' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('Destination is unavailable.');
    expect(screen.getByRole('button', { name: 'Apply rename' })).toBeEnabled();
    expect(screen.getByRole('complementary', { name: 'Review item' })).toBeVisible();
  });

  it('disables contextual actions while a bridge command is pending', async () => {
    let finish: (() => void) | undefined;
    const base = createInMemoryBridge();
    const approve = vi.fn(() => new Promise<void>((resolve) => { finish = resolve; }));
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    const action = screen.getByRole('button', { name: 'Apply rename' });
    fireEvent.click(action);

    expect(action).toBeDisabled();
    finish?.();
    // Done, review moves on to the next ready item, whose action is enabled.
    await waitFor(() => expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('NDA - Acme Corp.docx'));
    expect(screen.getByRole('button', { name: 'Apply rename' })).toBeEnabled();
  });

  it('does not close a newer selection when an earlier item action completes', async () => {
    let finish: (() => void) | undefined;
    const base = createInMemoryBridge();
    const approve = vi.fn(() => new Promise<void>((resolve) => { finish = resolve; }));
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Apply rename' }));
    await selectRow(screen.getByRole('row', { name: /NDA - Acme Corp/i }));
    finish?.();

    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveValue('2024-03-01 Non-Disclosure Agreement with Acme Corp'));
    expect(screen.getByRole('complementary', { name: 'Review item' })).toBeVisible();
  });

  it('does not close a newer selection when Apply all completes', async () => {
    let finish: (() => void) | undefined;
    const base = createInMemoryBridge();
    const approve = vi.fn(async (id: string, filename: string, description: string) => {
      if (id === 'employment') await new Promise<void>((resolve) => { finish = resolve; });
      await base.approve(id, filename, description);
    });
    render(<App bridge={{ ...base, approve }} />);

    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Apply all ready' }));
    await selectRow(screen.getByRole('row', { name: /Lease Agreement - 123 Main St/i }));
    finish?.();

    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc'));
    expect(screen.getByRole('complementary', { name: 'Review item' })).toBeVisible();
  });

  it.each([
    ['Cancel processing', 'cancel', { id: 'active', originalFilename: 'active.pdf', status: 'processing' as const }],
    ['Retry item', 'retry', { id: 'failed', originalFilename: 'failed.pdf', status: 'failed' as const }],
    ['Remove item', 'remove', { id: 'failed', originalFilename: 'failed.pdf', status: 'failed' as const }],
  ])('reports a polite visible error when %s fails', async (actionName, method, item) => {
    const command = vi.fn(async () => { throw new Error(`${actionName} failed.`); });
    const base = createInMemoryBridge({ items: [item] });
    const bridge = { ...base, [method]: command };
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: new RegExp(item.originalFilename, 'i') }));
    fireEvent.click(screen.getByRole('button', { name: actionName }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent(`${actionName} failed.`);
    expect(screen.getByRole('button', { name: actionName })).toBeEnabled();
  });

  it('reports Clear history failures without emptying the completed view', async () => {
    const clearHistory = vi.fn(async () => { throw new Error('History is locked.'); });
    render(<App bridge={{ ...createInMemoryBridge(), clearHistory }} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear history' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('History is locked.');
    expect(screen.getByRole('row', { name: /Completed lease.pdf/i })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Clear history' })).toBeEnabled();
  });

  it('offers retry and remove only when a failed item is selected', async () => {
    const retry = vi.fn(async () => undefined);
    const remove = vi.fn(async () => undefined);
    const bridge = { ...createInMemoryBridge({ items: [{ id: 'failed', originalFilename: 'broken.pdf', status: 'failed', reason: 'Extraction failed.' }] }), retry, remove };
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: /broken.pdf/i }));
    expect(screen.queryByLabelText('Filename')).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Failure details' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Retry item' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Remove item' }));

    await waitFor(() => expect(remove).toHaveBeenCalledWith('failed'));
    expect(retry).not.toHaveBeenCalled();
  });

  it('wires cancellation for the selected active item', async () => {
    const cancel = vi.fn(async () => undefined);
    const bridge = { ...createInMemoryBridge({ items: [{ id: 'active', originalFilename: 'active.pdf', status: 'processing', progress: 25 }] }), cancel };
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: /active.pdf/i }));
    expect(screen.queryByLabelText('Filename')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel processing' }));

    await waitFor(() => expect(cancel).toHaveBeenCalledWith('active'));
  });

  // No figure ever comes from the backend, and "Processing (0%)" on every
  // row read as stalled for the whole of a long OCR run.
  it('processing without progress shows stage', async () => {
    const bridge = createInMemoryBridge({ items: [
      { id: 'reading', originalFilename: 'reading.pdf', status: 'processing', stage: 'reading' },
      { id: 'naming', originalFilename: 'naming.pdf', status: 'processing', stage: 'naming' },
      { id: 'filing', originalFilename: 'filing.pdf', status: 'processing', stage: 'filing', cancelable: false },
      { id: 'counted', originalFilename: 'counted.pdf', status: 'processing', stage: 'reading', progress: 33.3333 },
    ] });
    render(<App bridge={bridge} />);

    expect(await screen.findByRole('row', { name: /reading\.pdf/ })).toHaveTextContent('Reading document…');
    expect(screen.getByRole('row', { name: /naming\.pdf/ })).toHaveTextContent('Proposing a name…');
    expect(screen.getByRole('row', { name: /filing\.pdf/ })).toHaveTextContent('Renaming…');
    expect(screen.getByRole('row', { name: /counted\.pdf/ })).toHaveTextContent('Reading document… (33%)');
    expect(screen.queryByText(/\(0%\)/)).not.toBeInTheDocument();

    await selectRow(screen.getByRole('row', { name: /naming\.pdf/ }));
    expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Proposing a name…');
  });

  it('shows the demo queue working without a made-up percentage', async () => {
    render(<App bridge={createInMemoryBridge()} />);

    const row = await screen.findByRole('row', { name: /Q1 Financials/ });
    expect(row).toHaveTextContent('Proposing a name…');
    expect(row).not.toHaveTextContent('%');
  });

  it('does not offer cancellation during the atomic apply stage', async () => {
    const bridge = createInMemoryBridge({ items: [{ id: 'applying', originalFilename: 'applying.pdf', status: 'processing', progress: 90, cancelable: false }] });
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: /applying.pdf/i }));

    expect(screen.queryByRole('button', { name: 'Cancel processing' })).not.toBeInTheDocument();
  });

  it('offers Clear history only within a nonempty Completed view', async () => {
    const clearHistory = vi.fn(async () => undefined);
    const bridge = { ...createInMemoryBridge(), clearHistory };
    render(<App bridge={bridge} />);

    expect(screen.queryByRole('button', { name: 'Clear history' })).not.toBeInTheDocument();
    fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear history' }));

    await waitFor(() => expect(clearHistory).toHaveBeenCalledOnce());
  });

  it('shows em dashes for a waiting row proposal and confidence', async () => {
    const bridge = createInMemoryBridge({
      items: [{ id: 'waiting', originalFilename: 'Invoice INV-1001.pdf', status: 'waiting' }],
    });

    render(<App bridge={bridge} />);

    const row = await screen.findByRole('row', { name: /Invoice INV-1001.pdf/i });
    expect(within(row).getAllByText('—')).toHaveLength(2);
  });

  it('opens the review inspector after selecting a review item', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));

    expect(screen.getByRole('complementary', { name: 'Review item' })).toBeVisible();
    expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');
  });

  it('filters the table from Queue to Completed navigation', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));

    expect(screen.getByRole('row', { name: /Completed lease.pdf/i })).toBeVisible();
    expect(screen.queryByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i })).not.toBeInTheDocument();
  });

  it('passes exact serializable path selections from the injected picker to the bridge', async () => {
    const baseBridge = createInMemoryBridge({ items: [] });
    const addFiles = vi.fn(baseBridge.addFiles);
    const bridge = { ...baseBridge, addFiles };
    const first = { path: 'C:/Inbox/Alpha.pdf', displayName: 'Alpha.pdf' };
    const second = { path: 'C:/Inbox/Beta.txt', displayName: 'Beta.txt' };
    const pickFiles = vi.fn(async () => [first, second]);
    render(<App bridge={bridge} selection={{ pickFiles, pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add files' }));

    await waitFor(() => expect(pickFiles).toHaveBeenCalledOnce());
    await waitFor(() => expect(addFiles).toHaveBeenCalledWith([first, second]));
    expect(await screen.findByRole('row', { name: /Alpha.pdf/i })).toBeVisible();
    expect(screen.getByRole('row', { name: /Beta.txt/i })).toBeVisible();
  });

  it('preserves folder identity for a dropped directory instead of fabricating a PDF row', async () => {
    const baseBridge = createInMemoryBridge({ items: [] });
    const addFolder = vi.fn(baseBridge.addFolder);
    const bridge = { ...baseBridge, addFolder };
    const folder = { path: 'C:/Inbox/Contracts', displayName: 'Contracts', files: [] };
    const resolveDrop = vi.fn(async () => ({ folder }));
    render(<App bridge={bridge} selection={{ pickFiles: async () => [], pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop }} />);

    fireEvent.drop(await screen.findByLabelText('Drag files or folders here to add to the queue'), {
      dataTransfer: { files: [], items: [{ kind: 'file', getAsFileSystemHandle: async () => ({ kind: 'directory', name: 'Contracts' }) }] },
    });

    await waitFor(() => expect(resolveDrop).toHaveBeenCalledOnce());
    await waitFor(() => expect(addFolder).toHaveBeenCalledWith(folder));
    expect(await screen.findByRole('row', { name: /Contracts\//i })).toBeVisible();
    expect(screen.queryByRole('row', { name: /Contracts folder.pdf/i })).not.toBeInTheDocument();
  });

  it('uses the refreshed selected item rather than stale inspector data', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    await bridge.approve('lease', '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf', '');
    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));

    expect(await screen.findByRole('button', { name: 'Undo' })).toBeVisible();
    expect(screen.queryByRole('button', { name: /Approve & rename/i })).not.toBeInTheDocument();
  });

  // A batch of forty used to run behind a disabled button with nothing said
  // until the end, and nothing offered to take it back.
  it('apply_all_shows_progress_and_undo_toast', async () => {
    const base = createInMemoryBridge();
    const release: Array<() => void> = [];
    const approve = vi.fn(async (id: string, filename: string, description: string) => {
      await new Promise<void>((resolve) => { release.push(resolve); });
      await base.approve(id, filename, description);
    });
    render(<App bridge={{ ...base, approve }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));

    for (const step of [1, 2, 3]) {
      await waitFor(() => expect(screen.getByRole('group', { name: 'Action progress' })).toHaveTextContent(`Applying ${step} of 3…`));
      // Work still running cannot be dismissed, or undone half-way.
      expect(within(screen.getByRole('group', { name: 'Action progress' })).queryByRole('button')).not.toBeInTheDocument();
      await waitFor(() => expect(release).toHaveLength(step));
      release[step - 1]();
    }

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 3 documents.');
    expect(within(result).getByRole('button', { name: 'Undo these 3 renames' })).toHaveTextContent('Undo');
    expect(screen.queryByRole('group', { name: 'Action progress' })).not.toBeInTheDocument();
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 3 documents.');
  });

  it('undo_from_toast_reverts_batch', async () => {
    const base = createInMemoryBridge();
    let releaseFirst: (() => void) | undefined;
    const undo = vi.fn(async (id: string) => {
      if (id === 'employment') await new Promise<void>((resolve) => { releaseFirst = resolve; });
      await base.undo(id);
    });
    render(<App bridge={{ ...base, undo }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));
    fireEvent.click(within(await screen.findByRole('group', { name: 'Action result' })).getByRole('button', { name: 'Undo these 3 renames' }));

    // One at a time, as the backend undoes them: the next waits on the first.
    await waitFor(() => expect(undo).toHaveBeenCalledWith('employment'));
    expect(screen.getByRole('group', { name: 'Action progress' })).toHaveTextContent('Undoing 1 of 3…');
    expect(undo).toHaveBeenCalledTimes(1);
    releaseFirst?.();

    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 3 renames.'));
    expect(undo.mock.calls.map(([id]) => id)).toEqual(['employment', 'nda', 'service']);
    for (const name of [/Employment Agreement/, /NDA - Acme Corp/, /Service Agreement/]) {
      expect(screen.getByRole('row', { name })).toHaveTextContent('Needs review');
    }
    expect(screen.queryByRole('button', { name: /^Undo these/ })).not.toBeInTheDocument();
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Undid 3 renames.');
    // Undo left with its toast; focus goes to the first document put back.
    await waitFor(() => expect(screen.getByRole('button', { name: 'Select Employment Agreement - John Smith.pdf' })).toHaveFocus());
  });

  it('reports an undo the backend refuses and still undoes the rest', async () => {
    const base = createInMemoryBridge();
    const undo = vi.fn(async (id: string) => {
      if (id === 'nda') throw { code: 'STATE_CONFLICT', message: 'The filed copy changed after it was renamed.' };
      await base.undo(id);
    });
    render(<App bridge={{ ...base, undo }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));
    fireEvent.click(within(await screen.findByRole('group', { name: 'Action result' })).getByRole('button', { name: 'Undo these 3 renames' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('Undid 2 renames. 1 could not be undone. The filed copy changed after it was renamed.');
    expect(undo).toHaveBeenCalledTimes(3);
    expect(screen.getByRole('row', { name: /Service Agreement/ })).toHaveTextContent('Needs review');
    expect(screen.queryByRole('row', { name: /NDA - Acme Corp/ })).not.toBeInTheDocument();
  });

  it('offers Undo after a single rename, and Undo puts the document back', async () => {
    const base = createInMemoryBridge();
    const undo = vi.fn(base.undo);
    render(<App bridge={{ ...base, undo }} />);
    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: /Apply rename/ }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 1 document.');
    fireEvent.click(within(result).getByRole('button', { name: 'Undo this rename' }));

    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 1 rename.'));
    expect(undo).toHaveBeenCalledWith('employment');
    expect(screen.getByRole('row', { name: /Employment Agreement/i })).toHaveTextContent('Needs review');
  });

  it('takes Undo away once another action follows, so it cannot read as undoing that one', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));
    fireEvent.click(screen.getByRole('button', { name: /Apply rename/ }));
    await screen.findByRole('group', { name: 'Action result' });

    // Review has moved on to the next ready item; keeping it is not a rename.
    await waitFor(() => expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('NDA - Acme Corp.docx'));
    fireEvent.click(screen.getByRole('button', { name: /Keep original/ }));

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Kept NDA - Acme Corp.docx under its own name.'));
    expect(screen.queryByRole('group', { name: 'Action result' })).not.toBeInTheDocument();
  });

  // Accepted is not renamed: while the queue is busy the backend files an
  // approved name between documents, and the item stays ready until then.
  it('does not call an approval renamed until the queue shows it filed', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(async (id: string, filename: string, description: string) => {
      if (id === 'employment') await base.approve(id, filename, description);
    });
    render(<App bridge={{ ...base, approve }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 1 document. 2 will be renamed when the queue is free.');
    expect(within(result).getByRole('button', { name: 'Undo this rename' })).toBeEnabled();
  });

  /**
   * The desktop raises queue://changed as a command files a document, and
   * the event can arrive after the command's reply - while the action's own
   * reread is under way. The background read it starts supersedes that
   * reread, which is then never shown; here the newer read is held until the
   * test lets it go, so what is on screen is still the queue from before.
   */
  const eventAfterReply = () => {
    const base = createInMemoryBridge();
    let listener: ((event: QueueBridgeEvent) => void) | undefined;
    let phase: 'idle' | 'decided' | 'event' = 'idle';
    let release!: () => void;
    const held = new Promise<void>((resolve) => { release = resolve; });
    const bridge = {
      ...base,
      approve: async (id: string, filename: string, description: string) => { await base.approve(id, filename, description); phase = 'decided'; },
      listItems: async () => {
        if (phase === 'event') { phase = 'idle'; await held; }
        const read = await base.listItems();
        if (phase === 'decided') { phase = 'event'; queueMicrotask(() => listener?.({ type: 'changed' })); }
        return read;
      },
      subscribeQueue: async (next: (event: QueueBridgeEvent) => void) => { listener = next; return () => { listener = undefined; }; },
    };
    return { bridge, release };
  };

  it('says what a rename did from its own read, not one a queue event overtook', async () => {
    const { bridge, release } = eventAfterReply();
    render(<App bridge={bridge} />);
    await selectRow(await screen.findByRole('row', { name: /Employment Agreement/i }));

    fireEvent.click(screen.getByRole('button', { name: /Apply rename/ }));

    // Filed and undoable, which only the read made after the rename can say.
    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 1 document.');
    expect(within(result).getByRole('button', { name: 'Undo this rename' })).toBeEnabled();
    release();
  });

  it('says what Apply all ready did from its own read, not one a queue event overtook', async () => {
    const { bridge, release } = eventAfterReply();
    render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Apply all ready' }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 3 documents.');
    expect(result).not.toHaveTextContent('when the queue is free');
    expect(within(result).getByRole('button', { name: 'Undo these 3 renames' })).toBeEnabled();
    release();
  });
});

