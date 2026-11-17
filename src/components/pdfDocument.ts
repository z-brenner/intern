/*
  PDF pages for the document viewer, drawn by pdf.js inside Intern's own
  window.

  Neither webview Intern ships on can be counted on to draw a PDF itself:
  WebKitGTK has no PDF viewer at all, and WebView2's is not promised for a
  document handed to it as a blob in a frame - which the window's CSP
  (`frame-src 'none'`) refuses anyway. pdf.js draws each page to a canvas
  with nothing but script, so the CSP stays as it is.

  Everything is bundled: Intern makes no network request for a document. The
  worker comes from the app's own files (Vite's `?url`), and pdf.js falls back
  to running it on the window's thread where a worker cannot start. The
  legacy build is the one bundled: the modern one calls methods WebKitGTK
  versions still in use lack (`Uint8Array.prototype.toHex`) while opening
  every document. Scripts in a PDF never run (`isEvalSupported: false`, and
  no scripting sandbox is loaded); nothing in the document can be clicked.

  Of pdf.js's image decoders only OpenJPEG is bundled, for JPEG 2000 images,
  which a scan saved smaller often uses and which have no script fallback
  here. JBIG2 and CCITT, the other scan encodings, decode in script; colour
  profiles are approximated without their decoder; and fonts a PDF does not
  embed are drawn with the system's.

  This module is loaded only when a PDF is shown (DocumentViewer imports it
  lazily), so none of it is in the window's first bundle.
*/
import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist/legacy/build/pdf.mjs';
import type { PDFDocumentProxy, RenderTask } from 'pdfjs-dist/legacy/build/pdf.mjs';
import workerUrl from 'pdfjs-dist/legacy/build/pdf.worker.min.mjs?url';
import openjpegUrl from 'pdfjs-dist/wasm/openjpeg.wasm?url';

GlobalWorkerOptions.workerSrc = workerUrl;

/** The decoders bundled, by the name pdf.js asks for. */
const WASM: Record<string, string> = { 'openjpeg.wasm': openjpegUrl };

/**
 * Hands pdf.js the decoders Vite bundled, which have hashed names it cannot
 * guess: it asks for one by name through this, on the window's thread
 * (`useWorkerFetch: false`). One not bundled is refused, and pdf.js goes on
 * without it.
 */
class BundledWasm {
  constructor(_options: { baseUrl?: string | null }) {}

  async fetch({ filename }: { filename: string }): Promise<Uint8Array> {
    const url = WASM[filename];
    if (!url) throw new Error(`${filename} is not bundled`);
    const response = await fetch(url);
    if (!response.ok) throw new Error(`${filename} could not be loaded`);
    return new Uint8Array(await response.arrayBuffer());
  }
}

/** One page, sized in CSS pixels at 100%, ready to draw. */
export interface PdfPage {
  width: number;
  height: number;
  /** Draws the page into `canvas` at `scale`, sharp on this screen; `cancel` stops a drawing no longer wanted. */
  draw(canvas: HTMLCanvasElement, scale: number): { done: Promise<void>; cancel(): void };
}

export interface PdfDocument {
  pageCount: number;
  page(number: number): Promise<PdfPage>;
  close(): void;
}

/** A PDF point is 1/72 inch and a CSS pixel 1/96: a page at 100% is the size it prints. */
const CSS_PIXELS_PER_POINT = 96 / 72;

export async function openPdf(bytes: Uint8Array): Promise<PdfDocument> {
  // pdf.js takes the buffer it is given to its worker, so it is handed a copy:
  // the viewer keeps the file's bytes for drawing again.
  const task = getDocument({ data: bytes.slice(), isEvalSupported: false, useWorkerFetch: false, WasmFactory: BundledWasm, enableXfa: false });
  const document: PDFDocumentProxy = await task.promise;
  return {
    pageCount: document.numPages,
    async page(number) {
      const page = await document.getPage(number);
      const base = page.getViewport({ scale: CSS_PIXELS_PER_POINT });
      return {
        width: base.width,
        height: base.height,
        draw(canvas, scale) {
          const ratio = window.devicePixelRatio || 1;
          const viewport = page.getViewport({ scale: CSS_PIXELS_PER_POINT * scale * ratio });
          canvas.width = Math.max(1, Math.floor(viewport.width));
          canvas.height = Math.max(1, Math.floor(viewport.height));
          const task: RenderTask = page.render({ canvas, viewport });
          return {
            done: task.promise,
            cancel: () => task.cancel(),
          };
        },
      };
    },
    close: () => { void task.destroy(); },
  };
}
