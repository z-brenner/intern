import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useRef } from 'react';
import { ReviewInspector } from './ReviewInspector';
import { markedWords, viewKind } from './DocumentViewer';
import type { OpenPdf } from './DocumentViewer';
import type { PdfDocument } from './pdfDocument';
import type { DocumentSource, DocumentText } from '../lib/bridge';
import { createInMemoryBridge } from '../lib/inMemoryBridge';
import { useReviewShortcuts } from '../features/review/useReviewShortcuts';
import type { ReviewInspectorHandle } from '../features/review/useReviewShortcuts';
import type { QueueItem } from '../types';

/*
  The document a person is naming, read inside Intern's window: the file
  itself where the window can draw it, and the text Intern read from it.
  Nothing pressed or typed in the viewer reaches the review behind it.
*/

const invoice: QueueItem = {
  id: 'invoice', originalFilename: 'Scan 0102.pdf', status: 'review',
  proposedFilename: '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', confidence: 0.81,
  description: 'Invoice INV-3301 from Fabrikam Inc. for February consulting hours.',
  evidence: { date: 'Invoice date: 02/09/2024', type: 'INVOICE INV-3301', parties: 'Fabrikam Inc.' },
  reasonCodes: ['DATE_AMBIGUOUS'], reason: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.',
};
const letter: QueueItem = { ...invoice, id: 'letter', originalFilename: 'Letter to Contoso.docx', proposedFilename: '2024-02-09 Letter to Contoso Ltd.docx', evidence: { date: 'February 9, 2024', parties: 'Contoso Ltd' } };

const pages: DocumentText = {
  pages: [
    { number: 1, text: 'INVOICE INV-3301\nInvoice date: 02/09/2024\nBill to: Fabrikam\nInc.', ocr: false },
    { number: 2, text: 'Thank you for your business.', ocr: true },
  ],
  readNow: false,
  changed: false,
};

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

/** A PDF of `count` pages, drawn by recording each drawing. */
function fakePdf(count = 2) {
  const drawn: number[] = [];
  const closed = vi.fn();
  const openPdf: OpenPdf = vi.fn(async () => ({
    pageCount: count,
    page: async (number) => ({ width: 612, height: 792, draw: () => { drawn.push(number); return { done: Promise.resolve(), cancel: () => {} }; } }),
    close: closed,
  } satisfies PdfDocument));
  return { openPdf, drawn, closed };
}

function source(overrides: Partial<DocumentSource> = {}): DocumentSource & { documentFile: ReturnType<typeof vi.fn>; documentText: ReturnType<typeof vi.fn> } {
  return {
    documentFile: vi.fn(overrides.documentFile ?? (async () => new TextEncoder().encode('%PDF-1.7'))),
    documentText: vi.fn(overrides.documentText ?? (async () => pages)),
  };
}

function openPanel(item: QueueItem, preview: DocumentSource, openPdf: OpenPdf = fakePdf().openPdf, drawer = false) {
  const actions = handlers();
  render(<ReviewInspector item={item} drawer={drawer} preview={preview} openPdf={openPdf} {...actions} />);
  return actions;
}

const viewButton = () => screen.getByRole('button', { name: 'View document' });
const viewer = () => screen.getByRole('dialog', { name: /Scan 0102\.pdf|Letter to Contoso\.docx|photo\.png/ });

afterEach(() => vi.restoreAllMocks());

describe('the document viewer', () => {
  it('opens from the review panel over the window, and Escape closes it and gives the focus back', async () => {
    const preview = source();
    const actions = openPanel(invoice, preview);
    viewButton().focus();
    fireEvent.click(viewButton());

    const dialog = viewer();
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    // The file, drawn: the PDF's pages, each as it comes into view.
    expect(await within(dialog).findByRole('listitem', { name: 'Page 1 of 2' })).toBeInTheDocument();
    expect(within(dialog).getByRole('tab', { name: 'Original' })).toHaveAttribute('aria-selected', 'true');
    expect(preview.documentFile).toHaveBeenCalledWith('invoice');
    // The text is read only once it is asked for.
    expect(preview.documentText).not.toHaveBeenCalled();
    // The focus is in the document, where the arrow keys read it.
    expect(document.activeElement).toBe(within(dialog).getByRole('tabpanel'));

    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(viewButton()).toHaveFocus();
    // Escape closed the viewer, not the review behind it.
    expect(actions.onClose).not.toHaveBeenCalled();
  });

  it('shows the text ATLAS read, page by page, with what the name rests on marked', async () => {
    const preview = source();
    openPanel(invoice, preview);
    fireEvent.click(viewButton());
    fireEvent.click(within(viewer()).getByRole('tab', { name: 'Text ATLAS read' }));

    const first = await within(viewer()).findByRole('region', { name: 'Page 1' });
    expect(preview.documentText).toHaveBeenCalledOnce();
    expect(first).toHaveTextContent('INVOICE INV-3301');
    // The evidence's lines and parties, marked where the text says them, whatever the line breaks.
    const marks = [...viewer().querySelectorAll('.viewer-text-body mark')].map((mark) => [mark.textContent, mark.className.replace('viewer-mark viewer-mark--', '')]);
    expect(marks).toEqual([['INVOICE INV-3301', 'type'], ['Invoice date: 02/09/2024', 'date'], ['Fabrikam\nInc.', 'parties']]);
    expect(within(viewer()).getByText('Marked:')).toBeInTheDocument();
    // A page read from a picture says so.
    expect(within(viewer()).getByRole('region', { name: 'Page 2' })).toHaveTextContent('Scanned');
    // Back to the original, which was not read again.
    fireEvent.click(within(viewer()).getByRole('tab', { name: 'Original' }));
    expect(preview.documentFile).toHaveBeenCalledOnce();
  });

  it('moves between its two views with the arrow keys, as tabs do', async () => {
    openPanel(invoice, source());
    fireEvent.click(viewButton());
    const original = within(viewer()).getByRole('tab', { name: 'Original' });
    original.focus();
    fireEvent.keyDown(original, { key: 'ArrowRight' });
    const text = within(viewer()).getByRole('tab', { name: 'Text ATLAS read' });
    expect(text).toHaveAttribute('aria-selected', 'true');
    await waitFor(() => expect(text).toHaveFocus());
    expect(original).toHaveAttribute('tabindex', '-1');
  });

  it('shows a Word file as the text ATLAS read, with nothing to draw', async () => {
    const preview = source();
    openPanel(letter, preview);
    fireEvent.click(viewButton());

    expect(await within(viewer()).findByRole('region', { name: 'Page 1' })).toBeInTheDocument();
    expect(within(viewer()).queryByRole('tab')).not.toBeInTheDocument();
    expect(preview.documentFile).not.toHaveBeenCalled();
  });

  it('says when the file has changed since ATLAS read it', async () => {
    openPanel(letter, source({ documentText: async () => ({ ...pages, readNow: true, changed: true }) }));
    fireEvent.click(viewButton());
    expect(await within(viewer()).findByRole('note')).toHaveTextContent('This file has changed since ATLAS read it. This is the text as it is now.');
  });

  it('shows a picture as it is', async () => {
    const photo: QueueItem = { ...invoice, id: 'photo', originalFilename: 'photo.png', proposedFilename: '2024-02-09 Receipt.png' };
    openPanel(photo, source({ documentFile: async () => new Uint8Array([137, 80, 78, 71]) }));
    fireEvent.click(viewButton());
    const picture = await within(viewer()).findByRole('img', { name: 'The document photo.png' });
    expect(picture.getAttribute('src')).toMatch(/^(blob:|data:image\/png;base64,)/);
  });

  it('says why a document cannot be shown, and tries again when that can help', async () => {
    let held = true;
    const preview = source({ documentFile: async () => { if (held) throw { code: 'SOURCE_LOCKED', message: 'held' }; return new Uint8Array([37]); } });
    openPanel(invoice, preview);
    fireEvent.click(viewButton());

    const alert = await within(viewer()).findByRole('alert');
    expect(alert).toHaveTextContent('Another program is holding the document, so ATLAS could not read it. Close it there, then try again.');
    held = false;
    fireEvent.click(within(alert).getByRole('button', { name: 'Try again' }));
    expect(await within(viewer()).findByRole('listitem', { name: 'Page 1 of 2' })).toBeInTheDocument();
    expect(preview.documentFile).toHaveBeenCalledTimes(2);
  });

  it('says a document that has gone is not where ATLAS last saw it, and a file too large is to be opened instead', async () => {
    openPanel(invoice, source({ documentFile: async () => { throw { code: 'PATH_UNAVAILABLE', message: 'gone' }; }, documentText: async () => { throw { code: 'PATH_UNAVAILABLE', message: 'gone' }; } }));
    fireEvent.click(viewButton());
    expect(await within(viewer()).findByRole('alert')).toHaveTextContent('The document is not where ATLAS last saw it. It may have been moved, renamed or deleted outside ATLAS.');
    fireEvent.click(within(viewer()).getByRole('tab', { name: 'Text ATLAS read' }));
    expect(await within(viewer()).findByRole('alert')).toHaveTextContent('not where ATLAS last saw it');
    fireEvent.keyDown(viewer(), { key: 'Escape' });

    const actions = handlers();
    render(<ReviewInspector item={{ ...invoice, id: 'big' }} drawer={false} preview={source({ documentFile: async () => { throw { code: 'VIEW_TOO_LARGE', message: 'big' }; } })} openPdf={fakePdf().openPdf} {...actions} />);
    fireEvent.click(screen.getAllByRole('button', { name: 'View document' }).at(-1)!);
    const alert = await within(viewer()).findByRole('alert');
    expect(alert).toHaveTextContent('This file is too large to show here. Open it in its own program instead.');
    // Trying again would say the same; reading its text is offered instead.
    expect(within(alert).queryByRole('button', { name: 'Try again' })).not.toBeInTheDocument();
    fireEvent.click(within(alert).getByRole('button', { name: 'Show the text ATLAS read' }));
    expect(await within(viewer()).findByRole('region', { name: 'Page 1' })).toBeInTheDocument();
    // And its own program is a press away.
    fireEvent.click(within(viewer()).getByRole('button', { name: 'Open in its program' }));
    expect(actions.onOpen).toHaveBeenCalledOnce();
  });

  it('says what pdf.js could not draw, and offers the text instead', async () => {
    const openPdf: OpenPdf = async () => { throw Object.assign(new Error('Invalid PDF structure.'), { name: 'InvalidPDFException' }); };
    openPanel(invoice, source(), openPdf);
    fireEvent.click(viewButton());
    const alert = await within(viewer()).findByRole('alert');
    expect(alert).toHaveTextContent('This file is not a PDF ATLAS can draw. It may be damaged.');
    fireEvent.click(within(alert).getByRole('button', { name: 'Show the text ATLAS read' }));
    expect(await within(viewer()).findByRole('region', { name: 'Page 1' })).toBeInTheDocument();
  });

  it('keeps Tab inside, and no key pressed in it reaches the review behind', async () => {
    const actions = openPanel(invoice, source(), fakePdf().openPdf, true);
    const heard = vi.fn();
    document.addEventListener('keydown', heard);
    try {
      fireEvent.click(viewButton());
      const dialog = viewer();
      await within(dialog).findByRole('listitem', { name: 'Page 1 of 2' });
      const close = within(dialog).getByRole('button', { name: 'Close document' });
      close.focus();
      fireEvent.keyDown(close, { key: 'Tab' });
      expect(dialog.contains(document.activeElement)).toBe(true);
      expect(document.activeElement).not.toBe(close);
      // Ctrl+Enter, Alt+K and Delete in the viewer: review never hears them.
      for (const key of [{ key: 'Enter', ctrlKey: true }, { key: 'k', code: 'KeyK', altKey: true }, { key: 'Delete' }]) fireEvent.keyDown(close, key);
      expect(heard).not.toHaveBeenCalled();
      expect(actions.onApprove).not.toHaveBeenCalled();
      expect(actions.onKeep).not.toHaveBeenCalled();
      // A key aimed behind it, with the focus lost to the page, is dropped; Escape still closes it.
      (document.activeElement as HTMLElement).blur();
      expect(fireEvent.keyDown(document.body, { key: 'Enter', ctrlKey: true })).toBe(false);
      fireEvent.keyDown(document.body, { key: 'Escape' });
      expect(screen.queryByRole('dialog', { name: /Scan 0102/ })).not.toBeInTheDocument();
      // Escape closed the viewer, and the drawer it was opened from stays open.
      expect(actions.onClose).not.toHaveBeenCalled();
    } finally {
      document.removeEventListener('keydown', heard);
    }
  });

  it('closes with a press on the backdrop, and when review moves on to another document', async () => {
    const preview = source();
    const actions = handlers();
    const { rerender } = render(<ReviewInspector item={invoice} drawer={false} preview={preview} openPdf={fakePdf().openPdf} {...actions} />);
    fireEvent.click(viewButton());
    fireEvent.click(viewer().parentElement!);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

    fireEvent.click(viewButton());
    expect(viewer()).toBeInTheDocument();
    rerender(<ReviewInspector item={letter} drawer={false} preview={preview} openPdf={fakePdf().openPdf} {...actions} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('is offered only where Open is, and only with somewhere to read the document from', () => {
    const { unmount } = render(<ReviewInspector item={invoice} drawer={false} {...handlers()} />);
    expect(screen.queryByRole('button', { name: 'View document' })).not.toBeInTheDocument();
    unmount();
    // Held because nobody could vouch for who uploaded it: not opened from Intern, so not viewed either.
    render(<ReviewInspector item={{ ...invoice, errorCode: 'UPLOADER_UNVERIFIED', reasonCodes: undefined }} drawer={false} preview={source()} {...handlers()} />);
    expect(screen.queryByRole('button', { name: 'View document' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Show in folder' })).toBeInTheDocument();
  });

  it('closes the PDF it opened', async () => {
    const pdf = fakePdf();
    openPanel(invoice, source(), pdf.openPdf);
    fireEvent.click(viewButton());
    await within(viewer()).findByRole('listitem', { name: 'Page 1 of 2' });
    await waitFor(() => expect(pdf.drawn).toContain(1));
    fireEvent.keyDown(viewer(), { key: 'Escape' });
    expect(pdf.closed).toHaveBeenCalled();
  });
});

describe('what the viewer shows of a file, and marks in its text', () => {
  it('draws PDFs and the pictures a webview shows, and shows everything else as text', () => {
    expect(viewKind('Scan.PDF')).toBe('pdf');
    expect(viewKind('photo.jpeg')).toBe('image');
    expect(viewKind('receipt.png')).toBe('image');
    for (const name of ['scan.tiff', 'letter.docx', 'sheet.xlsx', 'mail.msg', 'notes.txt']) expect(viewKind(name), name).toBe('text');
  });

  it('marks the evidence\'s lines and parties, once each, and nothing too short to mean anything', () => {
    expect(markedWords({ ...invoice, evidence: { date: 'March 8, 2024', type: 'Letter', parties: 'Fabrikam Inc.; Northwind; AB; fabrikam inc.' } })).toEqual([
      { field: 'date', words: 'March 8, 2024' },
      { field: 'type', words: 'Letter' },
      { field: 'parties', words: 'Fabrikam Inc.' },
      { field: 'parties', words: 'Northwind' },
    ]);
    expect(markedWords({ ...invoice, evidence: undefined })).toEqual([]);
  });
});

describe('the viewer from the keyboard', () => {
  /** The review shortcuts, with an open panel whose handle records what it was asked. */
  function Shortcuts({ selected, handle }: { selected: QueueItem; handle: ReviewInspectorHandle }) {
    const inspector = useRef<ReviewInspectorHandle>(handle);
    useReviewShortcuts({ enabled: true, rows: [selected], selected, inspector, onMove: vi.fn(), onOpen: vi.fn(), onFilter: () => false });
    return <div><input aria-label="Filter" /><aside className="inspector"><textarea aria-label="Filename" /></aside></div>;
  }
  const handle = () => ({ approve: vi.fn(), keep: vi.fn(), requestRemove: vi.fn(), focus: vi.fn(), view: vi.fn() });

  it('opens with Alt+V from the name, as Alt+K keeps from there, and with V where no field has the focus', () => {
    const asked = handle();
    render(<Shortcuts selected={invoice} handle={asked} />);
    fireEvent.keyDown(screen.getByLabelText('Filename'), { key: 'v', code: 'KeyV', altKey: true });
    expect(asked.view).toHaveBeenCalledOnce();
    fireEvent.keyDown(document.body, { key: 'v', code: 'KeyV' });
    expect(asked.view).toHaveBeenCalledTimes(2);
    // V typed into a field is a letter, and a held key opens nothing twice.
    fireEvent.keyDown(screen.getByLabelText('Filename'), { key: 'v', code: 'KeyV' });
    fireEvent.keyDown(screen.getByLabelText('Filter'), { key: 'v', code: 'KeyV', altKey: true });
    fireEvent.keyDown(document.body, { key: 'v', code: 'KeyV', repeat: true });
    // Ctrl+V pastes.
    fireEvent.keyDown(document.body, { key: 'v', code: 'KeyV', ctrlKey: true });
    expect(asked.view).toHaveBeenCalledTimes(2);
  });

  it('opens nothing for a document Open is not offered for', () => {
    const asked = handle();
    render(<Shortcuts selected={{ id: 'w', originalFilename: 'scan.pdf', status: 'waiting' }} handle={asked} />);
    fireEvent.keyDown(document.body, { key: 'v', code: 'KeyV' });
    fireEvent.keyDown(screen.getByLabelText('Filename'), { key: 'v', code: 'KeyV', altKey: true });
    expect(asked.view).not.toHaveBeenCalled();
  });

  it('opens from the panel\'s handle only where View document is offered', () => {
    const ref = { current: null as ReviewInspectorHandle | null };
    render(<ReviewInspector ref={ref} item={invoice} drawer={false} preview={source()} openPdf={fakePdf().openPdf} {...handlers()} />);
    act(() => ref.current!.view());
    expect(viewer()).toBeInTheDocument();
  });
});

describe('the browser preview\'s stand-in document', () => {
  it('gives a PDF for a PDF, refuses to draw anything else, and has text for every document', async () => {
    const bridge = createInMemoryBridge({ items: [invoice, letter] });
    const pdf = new TextDecoder().decode(await bridge.documentFile('invoice'));
    expect(pdf.startsWith('%PDF-1.4')).toBe(true);
    expect(pdf).toContain('(Date: Invoice date: 02/09/2024) Tj');
    // Each object where the cross-reference table says it is.
    const offsets = [...pdf.matchAll(/^(\d{10}) 00000 n $/gm)].map((match) => Number(match[1]));
    offsets.forEach((offset, index) => expect(pdf.slice(offset).startsWith(`${index + 1} 0 obj`), `object ${index + 1}`).toBe(true));
    await expect(bridge.documentFile('letter')).rejects.toMatchObject({ code: 'VIEW_UNSUPPORTED' });
    const text = await bridge.documentText('letter');
    expect(text.pages[0].text).toContain('From: Contoso Ltd');
    await expect(bridge.documentText('gone')).rejects.toMatchObject({ code: 'ITEM_NOT_FOUND' });
  });
});
