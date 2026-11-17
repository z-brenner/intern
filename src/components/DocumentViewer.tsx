import { ExternalLink, LoaderCircle, Minus, Plus, ScanText, X } from 'lucide-react';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import type { KeyboardEvent as ReactKeyboardEvent, ReactNode, RefObject } from 'react';
import { createPortal } from 'react-dom';
import { describeActionError } from '../lib/actionErrors';
import type { DocumentSource, DocumentText } from '../lib/bridge';
import { useEditsOnlyInside } from '../lib/dialogEdits';
import { tabWithin } from '../lib/dialogTab';
import { filenameExtension } from '../lib/filenames';
import type { QueueItem } from '../types';
import { FileKindIcon } from './FileKindIcon';
import { Icon } from './Icon';
import type { PdfDocument, PdfPage } from './pdfDocument';
import './DocumentViewer.css';

/** How the window shows a document: a PDF drawn page by page, a picture, or only the text Intern read. */
export type ViewKind = 'pdf' | 'image' | 'text';

/**
 * What the window can draw of a file, by its extension - the same list the
 * backend reads bytes for (`DRAWN_IN_WINDOW` in viewer.rs). A TIFF scan is
 * text only: no webview Intern ships on draws one.
 */
export function viewKind(filename: string): ViewKind {
  const extension = filenameExtension(filename)?.toLowerCase();
  if (extension === 'pdf') return 'pdf';
  if (extension === 'png' || extension === 'jpg' || extension === 'jpeg') return 'image';
  return 'text';
}

/** Opens a PDF for drawing; pdf.js, loaded only when a PDF is shown. */
export type OpenPdf = (bytes: Uint8Array) => Promise<PdfDocument>;
const openBundledPdf: OpenPdf = async (bytes) => (await import('./pdfDocument')).openPdf(bytes);

/** What is said of a document the viewer could not show, by the backend's code. */
const VIEW_ERRORS: Record<string, string> = {
  PATH_UNAVAILABLE: 'The document is not where Intern last saw it. It may have been moved, renamed or deleted outside Intern.',
  SOURCE_LOCKED: 'Another program is holding the document, so Intern could not read it. Close it there, then try again.',
  VIEW_TOO_LARGE: 'This file is too large to show here. Open it in its own program instead.',
  VIEW_UNSUPPORTED: 'Intern cannot draw this kind of file here.',
  UPLOADER_UNVERIFIED: 'Intern does not open a document from an uploader it could not verify.',
  ITEM_NOT_FOUND: 'That document is no longer in the queue.',
  PASSWORD_PROTECTED: 'This file is password-protected, so Intern could not read its text.',
  UNSUPPORTED_CONTENT: 'This file is not what its extension says it is, so Intern could not read its text.',
  DOCUMENT_TOO_LARGE: 'This document is too large for Intern to read its text.',
  OCR_UNAVAILABLE: 'Intern\'s text-recognition files are missing, so it could not read this scan. Reinstall Intern to restore them.',
  EXTRACTION_FAILED: 'Intern could not read this file\'s text. The file may be damaged.',
  IO_ERROR: 'The document could not be read.',
  PICTURE_UNREADABLE: 'Intern could not draw this picture. The file may be damaged.',
};

/** Failures that are the file's for good: trying again cannot change them. */
const SETTLED = new Set(['VIEW_TOO_LARGE', 'VIEW_UNSUPPORTED', 'UPLOADER_UNVERIFIED', 'ITEM_NOT_FOUND', 'PASSWORD_PROTECTED', 'UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'OCR_UNAVAILABLE']);

const errorCode = (error: unknown) => typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;

/** pdf.js says what is wrong with a PDF by the name of what it throws. */
function describeViewError(error: unknown): string {
  const code = errorCode(error);
  if (code && VIEW_ERRORS[code]) return VIEW_ERRORS[code];
  const name = typeof error === 'object' && error && 'name' in error ? String(error.name) : '';
  if (name === 'PasswordException') return 'This PDF is password-protected, so Intern cannot draw it.';
  if (name === 'InvalidPDFException') return 'This file is not a PDF Intern can draw. It may be damaged.';
  return describeActionError(error);
}

type Loaded<T> = { state: 'loading' } | { state: 'ready'; value: T } | { state: 'failed'; error: unknown };

/**
 * Something the viewer loads from the bridge, once asked for, and again on
 * "Try again". What arrives after the viewer has moved on is dropped.
 */
function useLoaded<T>(load: () => Promise<T>, wanted: boolean): [Loaded<T> | undefined, () => void] {
  const [loaded, setLoaded] = useState<Loaded<T>>();
  const [attempt, setAttempt] = useState(0);
  const latest = useRef(load);
  useLayoutEffect(() => { latest.current = load; });
  useEffect(() => {
    if (!wanted) return;
    let current = true;
    setLoaded({ state: 'loading' });
    latest.current().then(
      (value) => { if (current) setLoaded({ state: 'ready', value }); },
      (error: unknown) => { if (current) setLoaded({ state: 'failed', error }); },
    );
    return () => { current = false; };
  }, [wanted, attempt]);
  return [loaded, () => setAttempt((count) => count + 1)];
}

/** How far the document is zoomed, as a share of the width that fits the window. */
const ZOOMS = [0.5, 0.75, 1, 1.25, 1.5, 2, 3];

interface Props {
  item: QueueItem;
  source: DocumentSource;
  onClose(): void;
  /** Open the document in its own program instead, when the window offers that. */
  onOpen?(): void;
  /** How a PDF is opened for drawing; pdf.js unless a test says otherwise. */
  openPdf?: OpenPdf;
}

/**
 * The document a person is naming, inside Intern's window: they used to have
 * to open it in another program to check a name against it, and switch back.
 *
 * A PDF is drawn page by page and a PNG or JPEG picture as it is ("Original");
 * every document can also be read as the text Intern read from it, page by
 * page ("Text Intern read"), which is what the name was made from - for a
 * Word file, a spreadsheet, a message or a TIFF scan it is all the window
 * shows. In that text the lines the evidence quotes and the parties it names
 * are marked, so the reviewer finds what the name rests on without hunting.
 *
 * A dialog over the whole window, like Settings: Escape closes it, Tab stays
 * in it, the focus goes back to where it was when it closes, and no key
 * pressed in it reaches the review behind it - Ctrl+Enter here must not file
 * the document being looked at. It reads only: nothing in it changes the
 * document or the queue.
 */
export function DocumentViewer({ item, source, onClose, onOpen, openPdf = openBundledPdf }: Props) {
  const name = item.filedName ?? item.originalFilename;
  const kind = viewKind(item.originalFilename);
  const [tab, setTab] = useState<'original' | 'text'>(kind === 'text' ? 'text' : 'original');
  const [zoom, setZoom] = useState(1);
  const [drawFailure, setDrawFailure] = useState<unknown>();
  const [file, retryFile] = useLoaded(() => source.documentFile(item.id), kind !== 'text');
  // Read only once asked for: reading again can be minutes of OCR for a scan
  // the person only wanted to look at.
  const [textWanted, setTextWanted] = useState(tab === 'text');
  if (tab === 'text' && !textWanted) setTextWanted(true);
  const [text, retryText] = useLoaded(() => source.documentText(item.id), textWanted);
  const dialog = useRef<HTMLElement>(null);
  const body = useRef<HTMLDivElement>(null);
  const closing = useRef(onClose);
  useLayoutEffect(() => { closing.current = onClose; });
  const titleId = useId();
  const panelId = useId();
  useEditsOnlyInside(dialog);

  const controls = () => [...(dialog.current?.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"]') ?? [])]
    .filter((control) => control.getAttribute('role') !== 'tab' || control.getAttribute('aria-selected') === 'true');

  // Into the document on opening, so the arrow keys and Page Down read it;
  // back to what had the focus on closing - the button that opened it, or the
  // name it was opened from with Alt+V.
  useEffect(() => {
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    body.current?.focus();
    return () => { if (before?.isConnected) before.focus(); };
  }, []);

  // A key aimed behind the viewer - the focus was lost to the page, a press
  // on the backdrop - is not review's: Escape closes, Tab comes back in, and
  // anything else is dropped before it reaches the queue or the panel.
  useEffect(() => {
    const behind = (event: KeyboardEvent) => {
      if (dialog.current?.contains(event.target as Node)) return;
      event.stopPropagation();
      if (event.key === 'Escape') { event.preventDefault(); closing.current(); return; }
      if (event.key === 'Tab') { tabWithin(event, controls()); return; }
      event.preventDefault();
    };
    document.addEventListener('keydown', behind, true);
    return () => document.removeEventListener('keydown', behind, true);
  }, []);

  const onKeyDown = (event: ReactKeyboardEvent<HTMLElement>) => {
    // Nothing behind the viewer hears a key pressed in it.
    event.stopPropagation();
    if (event.key === 'Escape' && !event.nativeEvent.isComposing) { event.preventDefault(); onClose(); return; }
    if (event.key === 'Tab') tabWithin(event.nativeEvent, controls());
  };

  const showTab = (next: 'original' | 'text') => { setTab(next); setDrawFailure(undefined); };
  const onTabKey = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight' && event.key !== 'Home' && event.key !== 'End') return;
    event.preventDefault();
    const next = event.key === 'Home' ? 'original' : event.key === 'End' ? 'text' : tab === 'original' ? 'text' : 'original';
    showTab(next);
    queueMicrotask(() => dialog.current?.querySelector<HTMLElement>(`[role="tab"][data-tab="${next}"]`)?.focus());
  };

  const zoomable = tab === 'original' && file?.state === 'ready' && !drawFailure;
  const zoomIndex = ZOOMS.indexOf(zoom);
  const original = tab === 'original';

  let content: ReactNode;
  if (original) {
    if (!file || file.state === 'loading') content = <Waiting>Opening the document…</Waiting>;
    else if (file.state === 'failed') content = <Failed error={file.error} onRetry={retryFile} onText={() => showTab('text')} />;
    else if (drawFailure) content = <Failed error={drawFailure} onText={() => showTab('text')} />;
    else if (kind === 'pdf') content = <PdfPages bytes={file.value} zoom={zoom} openPdf={openPdf} scroller={body} onFailure={setDrawFailure} />;
    else content = <Picture bytes={file.value} name={name} zoom={zoom} onFailure={setDrawFailure} />;
  } else if (!text || text.state === 'loading') {
    content = <Waiting>Reading the text…</Waiting>;
  } else if (text.state === 'failed') {
    content = <Failed error={text.error} onRetry={retryText} />;
  } else {
    content = <TextPages text={text.value} item={item} />;
  }

  return createPortal(<div className="viewer-backdrop" role="presentation" onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <section ref={dialog} className="document-viewer" role="dialog" aria-modal="true" aria-labelledby={titleId} onKeyDown={onKeyDown} onFocus={(event) => event.stopPropagation()}>
      <header className="viewer-head">
        <h2 id={titleId} className="viewer-title"><FileKindIcon filename={item.originalFilename} /><span>{name}</span></h2>
        {kind !== 'text' && <div className="viewer-tabs" role="tablist" aria-label="Show">
          {(['original', 'text'] as const).map((which) => <button key={which} type="button" role="tab" data-tab={which} id={`${panelId}-${which}`} aria-selected={tab === which} aria-controls={panelId} tabIndex={tab === which ? 0 : -1} onClick={() => showTab(which)} onKeyDown={onTabKey}>
            {which === 'original' ? 'Original' : 'Text Intern read'}
          </button>)}
        </div>}
        <div className="viewer-tools">
          {zoomable && <div className="viewer-zoom" role="group" aria-label="Zoom">
            <button type="button" aria-label="Zoom out" disabled={zoomIndex <= 0} onClick={() => setZoom(ZOOMS[zoomIndex - 1])}><Icon icon={Minus} /></button>
            <button type="button" className="viewer-zoom-level" title="Fit the width of the window" aria-label={zoom === 1 ? 'Fit to width' : `Zoom ${Math.round(zoom * 100)}%, fit to width`} onClick={() => setZoom(1)}>{zoom === 1 ? 'Fit' : `${Math.round(zoom * 100)}%`}</button>
            <button type="button" aria-label="Zoom in" disabled={zoomIndex >= ZOOMS.length - 1} onClick={() => setZoom(ZOOMS[zoomIndex + 1])}><Icon icon={Plus} /></button>
          </div>}
          {onOpen && <button type="button" className="viewer-open" title="Open the document in the program your computer uses for it" onClick={onOpen}><Icon icon={ExternalLink} />Open in its program</button>}
          <button type="button" className="viewer-close" aria-label="Close document" onClick={onClose}><Icon icon={X} /></button>
        </div>
      </header>
      <div ref={body} className={`viewer-body viewer-body--${original ? kind : 'text'}`} id={panelId} role={kind !== 'text' ? 'tabpanel' : undefined}
        aria-labelledby={kind !== 'text' ? `${panelId}-${tab}` : titleId} tabIndex={0}>
        {content}
      </div>
    </section>
  </div>, document.body);
}

function Waiting({ children }: { children: ReactNode }) {
  return <p className="viewer-status" role="status"><Icon icon={LoaderCircle} />{children}</p>;
}

function Failed({ error, onRetry, onText }: { error: unknown; onRetry?(): void; onText?(): void }) {
  const code = errorCode(error);
  return <div className="viewer-status viewer-status--failed" role="alert">
    <p>{describeViewError(error)}</p>
    <div className="viewer-status-actions">
      {onRetry && !(code && SETTLED.has(code)) && <button type="button" onClick={onRetry}>Try again</button>}
      {onText && code !== 'UPLOADER_UNVERIFIED' && code !== 'ITEM_NOT_FOUND' && code !== 'PATH_UNAVAILABLE' && <button type="button" onClick={onText}>Show the text Intern read</button>}
    </div>
  </div>;
}

/** A PNG or JPEG picture, at the width that fits, or zoomed. */
function Picture({ bytes, name, zoom, onFailure }: { bytes: Uint8Array; name: string; zoom: number; onFailure(error: unknown): void }) {
  const [url, setUrl] = useState<string>();
  useEffect(() => {
    const type = filenameExtension(name)?.toLowerCase() === 'png' ? 'image/png' : 'image/jpeg';
    // A blob of a copy of the bytes, which the window's CSP admits as a
    // picture (img-src blob:). Where there are no blob addresses - a test's
    // document - the picture is spelled out in its address instead.
    if (typeof URL.createObjectURL !== 'function') {
      let binary = '';
      bytes.forEach((byte) => { binary += String.fromCharCode(byte); });
      setUrl(`data:${type};base64,${btoa(binary)}`);
      return;
    }
    const made = URL.createObjectURL(new Blob([bytes.slice()], { type }));
    setUrl(made);
    return () => URL.revokeObjectURL(made);
  }, [bytes, name]);
  // At Fit, as wide as the window and no wider than the picture, which is
  // not blown up past its own pixels; zoomed, a share of that width.
  const size = zoom === 1 ? { maxWidth: '100%' } : { width: `${zoom * 100}%`, maxWidth: 'none' };
  return url ? <img className="viewer-picture" src={url} alt={`The document ${name}`} style={size} onError={() => onFailure({ code: 'PICTURE_UNREADABLE' })} /> : null;
}

/** The width pages are fitted to: the viewer's, less its padding, kept current as the window is resized. */
function useFitWidth(scroller: RefObject<HTMLDivElement | null>): number {
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = scroller.current;
    if (!element) return;
    const measure = () => setWidth(Math.max(200, element.clientWidth - 48));
    measure();
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [scroller]);
  return width;
}

/**
 * A PDF's pages, each drawn when it comes into view: a long contract is not
 * drawn whole to show its first page. Until a page is drawn it holds its
 * place at the first page's size, so the scroll bar is right from the start.
 */
function PdfPages({ bytes, zoom, openPdf, scroller, onFailure }: { bytes: Uint8Array; zoom: number; openPdf: OpenPdf; scroller: RefObject<HTMLDivElement | null>; onFailure(error: unknown): void }) {
  const [opened, setOpened] = useState<{ document: PdfDocument; first: PdfPage }>();
  const failed = useRef(onFailure);
  useLayoutEffect(() => { failed.current = onFailure; });
  useEffect(() => {
    let current = true;
    let document: PdfDocument | undefined;
    openPdf(bytes).then(async (made) => {
      document = made;
      if (!current) { made.close(); return; }
      const first = await made.page(1);
      if (current) setOpened({ document: made, first });
    }).catch((error: unknown) => { if (current) failed.current(error); });
    return () => { current = false; document?.close(); };
  }, [bytes, openPdf]);
  const fit = useFitWidth(scroller);
  if (!opened) return <Waiting>Opening the document…</Waiting>;
  const { document, first } = opened;
  return <ol className="viewer-pages" aria-label={`${document.pageCount} ${document.pageCount === 1 ? 'page' : 'pages'}`}>
    {Array.from({ length: document.pageCount }, (_, index) => <PdfPageView key={index} number={index + 1} count={document.pageCount} document={document} first={first} width={fit * zoom} scroller={scroller} onFailure={onFailure} />)}
  </ol>;
}

function PdfPageView({ number, count, document, first, width, scroller, onFailure }: { number: number; count: number; document: PdfDocument; first: PdfPage; width: number; scroller: RefObject<HTMLDivElement | null>; onFailure(error: unknown): void }) {
  const holder = useRef<HTMLLIElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [page, setPage] = useState<PdfPage | undefined>(number === 1 ? first : undefined);
  const [seen, setSeen] = useState(false);
  useEffect(() => {
    const element = holder.current;
    if (!element || typeof IntersectionObserver === 'undefined') { setSeen(true); return; }
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) { setSeen(true); observer.disconnect(); }
    }, { root: scroller.current, rootMargin: '600px 0px' });
    observer.observe(element);
    return () => observer.disconnect();
  }, [scroller]);
  useEffect(() => {
    if (!seen || page) return;
    let current = true;
    document.page(number).then((loaded) => { if (current) setPage(loaded); }, (error: unknown) => { if (current) onFailure(error); });
    return () => { current = false; };
  }, [seen, page, document, number, onFailure]);
  const shown = page ?? first;
  const scale = width > 0 ? width / shown.width : 0;
  useEffect(() => {
    if (!page || !seen || !canvas.current || scale <= 0) return;
    const drawing = page.draw(canvas.current, scale);
    drawing.done.catch((error: unknown) => {
      if (typeof error === 'object' && error && 'name' in error && error.name === 'RenderingCancelledException') return;
      onFailure(error);
    });
    return () => drawing.cancel();
  }, [page, seen, scale, onFailure]);
  return <li ref={holder} className="viewer-page" aria-label={`Page ${number} of ${count}`}>
    <canvas ref={canvas} style={{ width: `${Math.round(shown.width * scale)}px`, height: `${Math.round(shown.height * scale)}px` }} />
  </li>;
}

/** A run of the text the evidence is about: a line it quotes, a party it names. */
interface Marked { field: 'date' | 'type' | 'parties'; words: string }

const FIELD_NAMES: Record<Marked['field'], string> = { date: 'Date', type: 'Type', parties: 'Parties' };

/**
 * What to mark in the text: the lines the evidence quotes for the date and
 * the type, and the parties it names - the words the proposed name rests on.
 * A run of fewer than three characters would mark half the document.
 */
export function markedWords(item: QueueItem): Marked[] {
  const of = (field: Marked['field'], value?: string) => (value ?? '').split(';').map((words) => words.trim()).filter((words) => words.length >= 3).map((words) => ({ field, words }));
  const all = [...of('date', item.evidence?.date), ...of('type', item.evidence?.type), ...of('parties', item.evidence?.parties)];
  return all.filter((marked, index) => all.findIndex((other) => other.words.toLowerCase() === marked.words.toLowerCase()) === index);
}

const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** `text` with every run of `marks` in it marked, ignoring case and how the lines break. */
function markedText(text: string, marks: readonly Marked[], found: Set<Marked['field']>): ReactNode[] {
  if (!marks.length) return [text];
  const longestFirst = [...marks].sort((left, right) => right.words.length - left.words.length);
  const pattern = new RegExp(longestFirst.map((marked) => escapeRegExp(marked.words).replace(/\s+/g, '\\s+')).join('|'), 'gi');
  const squeeze = (words: string) => words.replace(/\s+/g, ' ').toLowerCase();
  const parts: ReactNode[] = [];
  let at = 0;
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0;
    const marked = longestFirst.find((candidate) => squeeze(candidate.words) === squeeze(match[0]));
    if (start > at) parts.push(text.slice(at, start));
    if (marked) found.add(marked.field);
    parts.push(<mark key={start} className={`viewer-mark viewer-mark--${marked?.field ?? 'date'}`} title={marked ? `${FIELD_NAMES[marked.field]} evidence` : undefined}>{match[0]}</mark>);
    at = start + match[0].length;
  }
  if (at < text.length) parts.push(text.slice(at));
  return parts;
}

/** The text Intern read, page by page, with what the name rests on marked. */
function TextPages({ text, item }: { text: DocumentText; item: QueueItem }) {
  const marks = markedWords(item);
  const found = new Set<Marked['field']>();
  const pages = text.pages.map((page) => ({ ...page, parts: markedText(page.text, marks, found) }));
  const legend = (['date', 'type', 'parties'] as const).filter((field) => found.has(field));
  return <div className="viewer-text">
    {text.changed && <p className="viewer-note" role="note">This file has changed since Intern read it. This is the text as it is now.</p>}
    {legend.length > 0 && <p className="viewer-legend">Marked: {legend.map((field) => <mark key={field} className={`viewer-mark viewer-mark--${field}`}>{FIELD_NAMES[field]}</mark>)}</p>}
    {pages.length === 0 && <p className="viewer-status">Intern found no text in this document.</p>}
    {pages.map((page) => <section key={page.number} className="viewer-text-page" aria-label={`Page ${page.number}`}>
      <h3>Page {page.number}{page.ocr && <span className="viewer-ocr" title="Read from a picture of the page, so a letter can be misread"><Icon icon={ScanText} />Scanned</span>}</h3>
      {page.text.trim() ? <p className="viewer-text-body">{page.parts}</p> : <p className="viewer-text-empty">No text on this page.</p>}
    </section>)}
  </div>;
}
