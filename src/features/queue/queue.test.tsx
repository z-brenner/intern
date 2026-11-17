import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SUPPORTED_FORMATS_LABEL } from '../../lib/formats';
import { createFixtureBatchBridge, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { DesktopBridge, LaunchReportSource } from '../../lib/bridge';
import type { AddReport, QueueItem } from '../../types';
import type { QueueBridgeEvent } from '../../lib/tauriBridge';
import { clickCell, moreAction, openDetails, queueCell } from './gridTesting';

/**
 * An install from an earlier release that files in place, with no output
 * folder: approving the name a document already has then has nowhere to move
 * it. The window is shown a pair of folders so it opens past setup; the
 * bridge underneath still files in place.
 */
function filingInPlace(items: QueueItem[]): DesktopBridge {
  const base = createInMemoryBridge({ items, settings: { destination: '' } });
  return { ...base, getSettings: async () => ({ ...(await base.getSettings()), destination: 'C:\\Intern\\Output' }) };
}

describe('queue interactions', () => {
  // Selects the row and opens the Details panel for it, as a reviewer does.
  const selectRow = async (row: HTMLElement) => {
    const cell = row.querySelector<HTMLElement>('[data-column="original"]')!;
    clickCell(cell);
    await openDetails();
    const inspector = await screen.findByRole('complementary', { name: 'Review item' });
    await waitFor(() => expect(inspector).toHaveTextContent(cell.textContent ?? ''));
  };
  it('keeps identical bytes from different paths separate and deduplicates only the same path', async () => {
    const bridge = createFixtureBatchBridge();

    const report = await bridge.copyToInput({ files: [
      { path: 'browser://duplicate-invoice-a.pdf', displayName: 'duplicate-invoice-a.pdf' },
      { path: 'browser://duplicate-invoice-b.pdf', displayName: 'duplicate-invoice-b.pdf' },
      { path: 'browser://unsupported.zip', displayName: 'unsupported.zip' },
      { path: 'browser://~$nda.docx', displayName: '~$nda.docx' },
    ] });

    const items = await bridge.listItems();
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-a.pdf')).toMatchObject({ status: 'review', proposedFilename: '2025-04-30 Invoice from Nimbus Orchard Supply Co.pdf' });
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-b.pdf')).toMatchObject({ status: 'review', reason: expect.stringMatching(/different path.*separate/i) });
    expect(items.find((item) => item.originalFilename === 'duplicate-invoice-b.pdf')?.id).not.toBe(items.find((item) => item.originalFilename === 'duplicate-invoice-a.pdf')?.id);
    // What the desktop leaves out is reported, not queued as a failed row.
    expect(report).toEqual({ added: 2, alreadyQueued: 0, copied: 2, inputFolder: 'C:\\Intern\\Input', skipped: [{ name: 'unsupported.zip', code: 'UNSUPPORTED_FORMAT' }, { name: '~$nda.docx', code: 'TEMPORARY_FILE' }] });
    expect(items.map((item) => item.originalFilename)).toEqual(['duplicate-invoice-a.pdf', 'duplicate-invoice-b.pdf']);

    // Its copy still waits in the queue, so the same file is not copied twice.
    expect(await bridge.copyToInput({ files: [{ path: 'browser://duplicate-invoice-a.pdf', displayName: 'duplicate-invoice-a.pdf' }] })).toEqual({ added: 0, alreadyQueued: 1, copied: 0, inputFolder: 'C:\\Intern\\Input', skipped: [] });
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

    const message = 'Copied 24 files into Input. Skipped 1: notes.zip (not a supported format).';
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
    const reports: AddReport[] = [
      { added: 198, alreadyQueued: 0, copied: 198, inputFolder: 'C:\\Intern\\Input', skipped: [{ name: 'scan-57.pdf', code: 'SOURCE_LOCKED' }, { name: 'empty.pdf', code: 'EMPTY_FILE' }] },
      { added: 1, alreadyQueued: 2, copied: 1, inputFolder: 'C:\\Intern\\Input', skipped: [] },
    ];
    const copyToInput = vi.fn(async () => reports.shift()!);
    const folder = { path: 'C:/Inbox/Scans', displayName: 'Scans' };
    render(<App bridge={{ ...base, copyToInput }} selection={{ pickFiles: async () => [], pickFolder: async () => folder, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add folder' }));
    await waitFor(() => expect(copyToInput).toHaveBeenCalledWith({ folder }));
    const skipped = 'Copied 198 files into Input. Skipped 2: scan-57.pdf (another program has it open), empty.pdf (the file is empty).';
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent(skipped));
    expect(screen.getByRole('note', { name: 'Files not added' })).toHaveTextContent(skipped);

    fireEvent.click(await screen.findByRole('button', { name: 'Add folder' }));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied 1 file into Input. 2 were already in the queue.'));
    // Nothing was left out this time, so the earlier notice goes with it.
    expect(screen.queryByRole('note', { name: 'Files not added' })).not.toBeInTheDocument();
  });

  // "Send to > Intern" adds outside the window. What it left out used to be
  // said nowhere at all; it is the same note as an add made here.
  it('names what documents sent to ATLAS left out', async () => {
    const base = createInMemoryBridge({ items: [] });
    let deliver: ((report: AddReport) => void) | undefined;
    const stop = vi.fn();
    const subscribeLaunchReports = vi.fn((handler: (report: AddReport) => void) => { deliver = handler; return stop; });
    const bridge: DesktopBridge & LaunchReportSource = { ...base, subscribeLaunchReports };
    const view = render(<App bridge={bridge} />);
    await screen.findByRole('main', { name: 'ATLAS' });
    await waitFor(() => expect(deliver).toBeDefined());

    act(() => deliver!({ added: 1, alreadyQueued: 0, copied: 1, inputFolder: 'C:\\Intern\\Input', skipped: [{ name: 'Contract.doc', code: 'UNSUPPORTED_FORMAT' }, { name: 'blank.pdf', code: 'EMPTY_FILE' }] }));

    const message = 'Copied 1 file into Input. Skipped 2: Contract.doc (not a supported format), blank.pdf (the file is empty).';
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

    // The added row is selected; selecting never opens the Details panel.
    fireEvent.drop(zone);
    await waitFor(() => expect(queueCell('duplicate-invoice-a.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument();
    expect(await openDetails()).toHaveTextContent('duplicate-invoice-a.pdf');
    fireEvent.click(screen.getByRole('button', { name: 'Close review' }));
    fireEvent.drop(zone);

    await waitFor(() => expect(queueCell('duplicate-invoice-a.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
    expect(screen.getAllByRole('gridcell', { name: 'duplicate-invoice-a.pdf' }).filter((cell) => cell.dataset.column === 'original')).toHaveLength(1);
  });

  it('describes supported formats without adding a nonfunctional keyboard stop', async () => {
    render(<App bridge={createInMemoryBridge()} />);

    const zone = await screen.findByRole('region', { name: /drag files/i });
    expect(zone).toHaveTextContent(`Supports ${SUPPORTED_FORMATS_LABEL}`);
    expect(zone).toHaveTextContent(/PowerPoint/);
    expect(zone).toHaveTextContent(/Outlook \.msg/);
    expect(zone).not.toHaveAttribute('tabindex');
  });

  // "A button to clear everything from the queue, in case you want to start
  // from scratch." Pointing the queue at a large folder by mistake used to be
  // unrecoverable from inside the app, and Discard waiting only took what had
  // not started. Clear queue takes everything not filed, after a question
  // that says how many and what becomes of the files.
  it('clears every document that is not filed after asking, and says the files stay where they are', async () => {
    const bridge = createInMemoryBridge();
    const clearQueue = vi.fn(bridge.clearQueue);
    render(<App bridge={{ ...bridge, clearQueue }} />);
    await screen.findByRole('grid', { name: 'Queue' });

    // It cannot be undone, so it is behind More, away from Rename all ready.
    fireEvent.click(moreAction('Clear queue…'));
    const dialog = screen.getByRole('alertdialog', { name: 'Clear the queue?' });
    expect(dialog).toHaveTextContent("This takes all 8 documents out of ATLAS's queue, including any being read or waiting for your review. Completed documents stay as they are.");
    expect(dialog).toHaveTextContent("Your files stay where they are, in Input. ATLAS won't pick them up again unless they change, or you add them again with Add files.");
    expect(dialog).toHaveTextContent("This can't be undone.");
    // Cancel has the focus: a key pressed on the way in clears nothing.
    expect(within(dialog).getByRole('button', { name: 'Cancel' })).toHaveFocus();
    expect(screen.getByRole('region', { name: 'Queue items' }).closest('.workspace')).toHaveAttribute('inert');

    fireEvent.click(within(dialog).getByRole('button', { name: 'Clear queue' }));

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Cleared 8 documents from the queue. The files are where they were.'));
    expect(clearQueue).toHaveBeenCalledOnce();
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Cleared 8 documents from the queue.');
    const remaining = await bridge.listItems();
    expect(remaining.map((item) => item.status)).toEqual(['completed']);
    // The view's own button has the focus; the More button went with the rows.
    await waitFor(() => expect(screen.getByRole('button', { name: /^Queue, 0/ })).toHaveFocus());
    expect(screen.queryByRole('button', { name: 'More actions' })).not.toBeInTheDocument();
  });

  it('clears nothing when the question is answered No, and gives the focus back to More', async () => {
    const bridge = createInMemoryBridge();
    const clearQueue = vi.fn(bridge.clearQueue);
    render(<App bridge={{ ...bridge, clearQueue }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    fireEvent.click(moreAction('Clear queue…'));
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'More actions' })).toHaveFocus();

    fireEvent.click(moreAction('Clear queue…'));
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(clearQueue).not.toHaveBeenCalled();
    expect((await bridge.listItems()).length).toBe(9);
  });

  // Some documents are kept: Remove asks to be confirmed for one whose files
  // need checking, and Clear queue does not decide that for anyone.
  it('says how many it kept and why', async () => {
    const clearQueue = vi.fn(async () => ({ cleared: 5, unsettled: 1, busy: 1, stillMarked: 2 }));
    render(<App bridge={{ ...createInMemoryBridge(), clearQueue }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    fireEvent.click(moreAction('Clear queue…'));
    fireEvent.click(screen.getByRole('button', { name: 'Clear queue' }));
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent(
      'Cleared 5 documents from the queue. The files are where they were. 4 stay in the queue: 1 needs its files checked first; 1 was being renamed; 2 are open in another program, so their own name could not be given back.',
    ));
  });

  it('says what went wrong when the queue could not be cleared, and leaves it as it was', async () => {
    const clearQueue = vi.fn(async () => { throw new Error('The queue is locked.'); });
    render(<App bridge={{ ...createInMemoryBridge(), clearQueue }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    fireEvent.click(moreAction('Clear queue…'));
    fireEvent.click(screen.getByRole('button', { name: 'Clear queue' }));
    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('The queue is locked.');
    expect(screen.getAllByRole('row')).toHaveLength(9);
  });

  // Cleared, the documents wait in the Queue folder; adding them again on
  // purpose brings them back.
  it('brings cleared documents back when the Queue folder is added again', async () => {
    const bridge = createInMemoryBridge();
    await bridge.clearQueue();
    expect((await bridge.listItems()).filter((item) => item.status !== 'completed')).toHaveLength(0);
    const report = await bridge.queueInputFolder();
    expect(report.added).toBe(8);
    expect((await bridge.listItems()).filter((item) => item.status === 'waiting').map((item) => item.originalFilename)).toContain('NDA - Acme Corp.docx');
  });

  it('closes the More menu from outside it, and with Escape back to its button', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const more = await screen.findByRole('button', { name: 'More actions' });
    fireEvent.click(more);
    expect(screen.getByRole('menu', { name: 'More actions' })).toBeInTheDocument();
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole('menu', { name: 'More actions' })).not.toBeInTheDocument();

    fireEvent.click(more);
    fireEvent.keyDown(screen.getByRole('menuitem', { name: 'Clear queue…' }), { key: 'Escape' });
    expect(screen.queryByRole('menu', { name: 'More actions' })).not.toBeInTheDocument();
    expect(more).toHaveFocus();
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

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

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

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

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
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
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

  it('reports Clear completed failures without emptying the completed view', async () => {
    const clearHistory = vi.fn(async () => { throw new Error('History is locked.'); });
    render(<App bridge={{ ...createInMemoryBridge(), clearHistory }} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));
    fireEvent.click(moreAction('Clear completed…'));
    fireEvent.click(screen.getByRole('button', { name: 'Clear completed' }));

    expect(await screen.findByRole('alert', { name: 'Action error' })).toHaveTextContent('History is locked.');
    expect(screen.getByRole('row', { name: /Completed lease.pdf/i })).toBeVisible();
    expect(moreAction('Clear completed…')).toBeEnabled();
  });

  it('offers retry and remove only when a failed item is selected', async () => {
    const retry = vi.fn(async () => undefined);
    const remove = vi.fn(async () => undefined);
    const bridge = { ...createInMemoryBridge({ items: [{ id: 'failed', originalFilename: 'broken.pdf', status: 'failed', reason: 'Extraction failed.' }] }), retry, remove };
    render(<App bridge={bridge} />);

    await selectRow(await screen.findByRole('row', { name: /broken.pdf/i }));
    expect(screen.queryByLabelText('Filename')).not.toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Why it failed' })).toBeVisible();
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

  // The same question, from the same place, in Completed.
  it('offers Clear completed only within the Completed view, after asking', async () => {
    const clearHistory = vi.fn(async () => undefined);
    const bridge = { ...createInMemoryBridge(), clearHistory };
    render(<App bridge={bridge} />);

    await screen.findByRole('grid', { name: 'Queue' });
    fireEvent.click(screen.getByRole('button', { name: 'More actions' }));
    expect(screen.queryByRole('menuitem', { name: 'Clear completed…' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Completed/ }));
    fireEvent.click(moreAction('Clear completed…'));
    const dialog = screen.getByRole('alertdialog', { name: 'Clear completed?' });
    expect(dialog).toHaveTextContent('This takes the 1 document off the Completed list, with its rename history.');
    expect(dialog).toHaveTextContent("Your files stay where they are: filed documents keep their new names. You won't be able to undo these renames from ATLAS.");
    expect(dialog).toHaveTextContent("This can't be undone.");
    expect(clearHistory).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Clear completed' }));

    await waitFor(() => expect(clearHistory).toHaveBeenCalledOnce());
    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Cleared 1 completed document. The files are where they were.'));
  });

  it('shows em dashes for a waiting row proposal and confidence', async () => {
    const bridge = createInMemoryBridge({
      items: [{ id: 'waiting', originalFilename: 'Invoice INV-1001.pdf', status: 'waiting' }],
    });

    render(<App bridge={bridge} />);

    // Proposed filename, description and confidence; Status says "Waiting".
    const row = await screen.findByRole('row', { name: /Invoice INV-1001.pdf/i });
    expect(within(row).getAllByText('—')).toHaveLength(3);
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
    const copyToInput = vi.fn(baseBridge.copyToInput);
    const bridge = { ...baseBridge, copyToInput };
    const first = { path: 'C:/Inbox/Alpha.pdf', displayName: 'Alpha.pdf' };
    const second = { path: 'C:/Inbox/Beta.txt', displayName: 'Beta.txt' };
    const pickFiles = vi.fn(async () => [first, second]);
    render(<App bridge={bridge} selection={{ pickFiles, pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add files' }));

    await waitFor(() => expect(pickFiles).toHaveBeenCalledOnce());
    await waitFor(() => expect(copyToInput).toHaveBeenCalledWith({ files: [first, second] }));
    expect(await screen.findByRole('row', { name: /Alpha.pdf/i })).toBeVisible();
    expect(screen.getByRole('row', { name: /Beta.txt/i })).toBeVisible();
  });

  // A dropped folder is flattened into the input folder: its documents, not
  // the folder, become rows, a name taken twice is numbered, and what is not
  // a document is passed over as Add folder always has.
  it('copies a dropped directory flat into the input folder', async () => {
    const baseBridge = createInMemoryBridge({ items: [] });
    const copyToInput = vi.fn(baseBridge.copyToInput);
    const bridge = { ...baseBridge, copyToInput };
    const folder = { path: 'C:/Inbox/Contracts', displayName: 'Contracts', files: [
      { path: 'C:/Inbox/Contracts/Lease.pdf', displayName: 'Contracts/Lease.pdf' },
      { path: 'C:/Inbox/Contracts/Old/Lease.pdf', displayName: 'Contracts/Old/Lease.pdf' },
      { path: 'C:/Inbox/Contracts/notes.zip', displayName: 'Contracts/notes.zip' },
    ] };
    const resolveDrop = vi.fn(async () => ({ folder }));
    render(<App bridge={bridge} selection={{ pickFiles: async () => [], pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop }} />);

    fireEvent.drop(await screen.findByLabelText('Drag files or folders here to copy them into Input'), {
      dataTransfer: { files: [], items: [{ kind: 'file', getAsFileSystemHandle: async () => ({ kind: 'directory', name: 'Contracts' }) }] },
    });

    await waitFor(() => expect(resolveDrop).toHaveBeenCalledOnce());
    await waitFor(() => expect(copyToInput).toHaveBeenCalledWith({ folder }));
    expect(await screen.findByRole('gridcell', { name: 'Lease.pdf' })).toBeVisible();
    expect(screen.getByRole('gridcell', { name: 'Lease (2).pdf' })).toBeVisible();
    expect(screen.queryByRole('row', { name: /Contracts\//i })).not.toBeInTheDocument();
    expect(screen.queryByRole('row', { name: /notes\.zip/ })).not.toBeInTheDocument();
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Copied 2 files into Input.');
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

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

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
    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));
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
    await waitFor(() => expect(queueCell('Employment Agreement - John Smith.pdf')).toHaveFocus());
  });

  it('reports an undo the backend refuses and still undoes the rest', async () => {
    const base = createInMemoryBridge();
    const undo = vi.fn(async (id: string) => {
      if (id === 'nda') throw { code: 'STATE_CONFLICT', message: 'The filed copy changed after it was renamed.' };
      await base.undo(id);
    });
    render(<App bridge={{ ...base, undo }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));
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

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 1 document. 2 will be renamed when the queue is free.');
    expect(within(result).getByRole('button', { name: 'Undo this rename' })).toBeEnabled();
  });

  // Pipeline::complete_already_named: approving the name a document already
  // has completes it as kept, with nothing renamed and nothing to undo. It
  // was announced as "Renamed", and counted among the renames Undo offered.
  it('does not call an approval of the name a document already has a rename', async () => {
    const items: QueueItem[] = [
      { id: 'named', originalFilename: '2024-05-01 Lease.pdf', status: 'ready', proposedFilename: '2024-05-01 Lease.pdf' },
      { id: 'memo', originalFilename: 'memo.pdf', status: 'ready', proposedFilename: '2024-05-02 Memo.pdf' },
    ];
    render(<App bridge={filingInPlace(items)} />);
    await selectRow(await screen.findByRole('row', { name: /2024-05-01 Lease\.pdf/ }));

    fireEvent.click(screen.getByRole('button', { name: /Apply rename/ }));

    await waitFor(() => expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('2024-05-01 Lease.pdf already had this name, so nothing was renamed. Next: memo.pdf.'));
    expect(screen.getByRole('status', { name: 'Action status' })).not.toHaveTextContent('Renamed');
    expect(screen.queryByRole('group', { name: 'Action result' })).not.toBeInTheDocument();
  });

  it('counts what Apply all ready found already named apart from what it renamed', async () => {
    const items: QueueItem[] = [
      { id: 'named', originalFilename: '2024-05-01 Lease.pdf', status: 'ready', proposedFilename: '2024-05-01 Lease.pdf' },
      { id: 'memo', originalFilename: 'memo.pdf', status: 'ready', proposedFilename: '2024-05-02 Memo.pdf' },
      { id: 'notice', originalFilename: 'notice.pdf', status: 'ready', proposedFilename: '2024-05-03 Notice.pdf' },
    ];
    const base = filingInPlace(items);
    const undo = vi.fn(base.undo);
    render(<App bridge={{ ...base, undo }} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 2 documents. 1 already had its name.');
    fireEvent.click(within(result).getByRole('button', { name: 'Undo these 2 renames' }));
    await waitFor(() => expect(screen.getByRole('group', { name: 'Action result' })).toHaveTextContent('Undid 2 renames.'));
    expect(undo.mock.calls.map(([id]) => id)).toEqual(['memo', 'notice']);
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

    fireEvent.click(await screen.findByRole('button', { name: 'Rename all ready' }));

    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result).toHaveTextContent('Renamed 3 documents.');
    expect(result).not.toHaveTextContent('when the queue is free');
    expect(within(result).getByRole('button', { name: 'Undo these 3 renames' })).toBeEnabled();
    release();
  });
});

