import { useEffect, useLayoutEffect, useRef } from 'react';
import { cellBlock, selectedBlock, tableBlock, toHtml, toTsv } from './grid';
import type { ColumnKey, CopiedBlock } from './grid';
import { selectionOf, useGridValue } from './gridStore';
import type { GridStore } from './gridStore';
import type { QueueItem } from '../../types';

/** Both formats of one copy: tab-separated text, and the same cells as an HTML table. */
export interface ClipboardPayload { text: string; html: string }

export const payloadFor = (block: CopiedBlock): ClipboardPayload => ({ text: toTsv(block), html: toHtml(block) });

/**
 * Put both formats into a copy event's clipboard, in place of whatever the
 * browser would have copied. This is the dependable route in Chromium and in
 * WebView2: the event's own clipboard needs no permission prompt, and it is
 * the only route that writes text/html alongside text/plain everywhere.
 */
export function fillCopyEvent(event: ClipboardEvent, payload: ClipboardPayload): boolean {
  if (!event.clipboardData) return false;
  event.clipboardData.setData('text/plain', payload.text);
  event.clipboardData.setData('text/html', payload.html);
  event.preventDefault();
  return true;
}

/**
 * Write a payload to the clipboard from a click. First by asking the document
 * to copy, and filling the copy event that raises (above); that is how the
 * keyboard's Ctrl+C works too, so the button and the key put exactly the same
 * thing on the clipboard. Where that is refused, the asynchronous clipboard
 * with both formats, and last plain text alone.
 */
export async function writeClipboard(payload: ClipboardPayload): Promise<void> {
  let delivered = false;
  const onCopy = (event: ClipboardEvent) => {
    delivered = fillCopyEvent(event, payload);
    // The table's own Ctrl+C handler must not fill it a second time.
    if (delivered) event.stopImmediatePropagation();
  };
  document.addEventListener('copy', onCopy, true);
  try {
    if (typeof document.execCommand === 'function') document.execCommand('copy');
  } catch {
    // An engine that refuses the command entirely; the routes below remain.
  } finally {
    document.removeEventListener('copy', onCopy, true);
  }
  if (delivered) return;
  const clipboard = typeof navigator !== 'undefined' ? navigator.clipboard : undefined;
  if (clipboard?.write && typeof ClipboardItem !== 'undefined') {
    try {
      await clipboard.write([new ClipboardItem({
        'text/plain': new Blob([payload.text], { type: 'text/plain' }),
        'text/html': new Blob([payload.html], { type: 'text/html' }),
      })]);
      return;
    } catch {
      // Some engines accept only plain text; try that before giving up.
    }
  }
  if (clipboard?.writeText) { await clipboard.writeText(payload.text); return; }
  throw new Error('The clipboard is not available here.');
}

interface Options {
  /** The rows the table shows, in its order. */
  rows: QueueItem[];
  store: GridStore;
  onCopied(block: CopiedBlock): void;
  onFailed(error: unknown): void;
}

/**
 * Copying from the queue table: Ctrl+C (or Cmd+C) while a cell has focus, and
 * the toolbar's Copy button. Both copy the selected cells. With none
 * selected, Ctrl+C copies the cell it is pressed on, as a spreadsheet does,
 * and the button - which then says "Copy table" - copies the whole table as
 * shown, headings first.
 *
 * Ctrl+C is answered from the document's copy event rather than a keydown, so
 * it also follows the platform's own copy command and never needs clipboard
 * permission. It acts only when focus is on a cell of the table: anywhere
 * else, and in any text field, copying is the browser's as ever.
 */
export function useGridClipboard({ rows, store, onCopied, onFailed }: Options) {
  const latest = useRef({ rows, onCopied, onFailed });
  useLayoutEffect(() => { latest.current = { rows, onCopied, onFailed }; });
  const selected = () => selectedBlock(selectionOf(store.get()), latest.current.rows);
  useEffect(() => {
    const onCopy = (event: ClipboardEvent) => {
      if (event.defaultPrevented) return;
      const focused = document.activeElement;
      const cell = focused instanceof HTMLElement ? focused.closest<HTMLElement>('.queue-grid [data-item-id][data-column]') : null;
      if (!cell) return;
      const copied = selected() ?? cellBlock({ id: cell.dataset.itemId!, column: cell.dataset.column as ColumnKey }, latest.current.rows);
      if (!copied?.rows.length) return;
      if (fillCopyEvent(event, payloadFor(copied))) latest.current.onCopied(copied);
    };
    document.addEventListener('copy', onCopy);
    return () => document.removeEventListener('copy', onCopy);
  }, []);
  /** The Copy button. Starts the copy within the click, as the clipboard requires. */
  const copy = () => {
    const copied = selected() ?? tableBlock(latest.current.rows);
    if (!copied.rows.length) return;
    writeClipboard(payloadFor(copied)).then(() => latest.current.onCopied(copied), (error: unknown) => latest.current.onFailed(error));
  };
  return { copy, hasSelection: useGridValue(store, (state) => Boolean(state.anchor)) };
}
