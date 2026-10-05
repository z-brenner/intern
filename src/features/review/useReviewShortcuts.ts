import { useEffect, useLayoutEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { QueueItem } from '../../types';
import { itemActions } from './actions';

/**
 * What the review panel does when a shortcut asks. The panel owns the name
 * being edited, so approving from the keyboard goes through it and gets the
 * same checks as the button.
 */
export interface ReviewInspectorHandle {
  approve(): void;
  keep(): void;
  /** Ask before removing: one keystroke is too easy to press by accident. */
  requestRemove(): void;
  focus(target: 'filename' | 'heading'): void;
}

export interface ReviewShortcutOptions {
  /** Off while a dialog is open over the queue, or the queue is not on screen. */
  enabled: boolean;
  /** The rows the table shows, in its order. */
  rows: QueueItem[];
  selected?: QueueItem;
  inspector: RefObject<ReviewInspectorHandle | null>;
  /** Select a row from the keyboard. */
  onMove(item: QueueItem): void;
  /** Enter on a row: select it and go to its name. */
  onOpen(item: QueueItem): void;
  /** Focus the filter box; false when there is none to focus. */
  onFilter(): boolean;
}

const NOT_TEXT = new Set(['button', 'checkbox', 'radio', 'submit', 'reset', 'range', 'color', 'file', 'image']);

/** Somewhere typing goes into a field, where a letter is a letter and not a command. */
function isTextField(target: Element | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  return target instanceof HTMLInputElement && !NOT_TEXT.has(target.type);
}

/**
 * Keyboard review (FRONTEND_UX-10). With 200 items queued, reaching Approve
 * from a selected row took about 197 presses of Tab, and focus went back to
 * the toolbar after every decision. Outside text fields:
 *
 * - J/K, and Up/Down on a row, move the selection;
 * - Enter on a row, or F2 anywhere, goes to the selected item's name;
 * - Ctrl+Enter approves and Alt+K keeps the original - from the review
 *   panel's own fields too, since after each decision the caret is already
 *   in the next item's name;
 * - Delete removes, after asking;
 * - "/" goes to the filter.
 *
 * Each acts only where the panel offers the matching button, so a shortcut
 * can never do what a click could not.
 */
export function useReviewShortcuts(options: ReviewShortcutOptions) {
  // Kept current as each render is committed. A passive effect ran a task
  // later, and the listener is the document's, not React's, so nothing made
  // React run it first: a key pressed in between acted on the rows and
  // selection of the render before - J from the first row "moved" to it.
  const latest = useRef(options);
  useLayoutEffect(() => { latest.current = options; });
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const { enabled, rows, selected, inspector, onMove, onOpen, onFilter } = latest.current;
      // A key a field or dialog already handled (Enter in the name, Escape
      // in the filter) is not a shortcut as well.
      if (!enabled || event.defaultPrevented || event.isComposing) return;
      const target = event.target instanceof Element ? event.target : null;
      const textField = isTextField(target);
      const inPanel = Boolean(target?.closest('.inspector'));
      const actions = selected ? itemActions(selected) : undefined;
      const command = event.ctrlKey || event.metaKey;

      if (event.key === 'Enter' && command && !event.altKey && !event.shiftKey) {
        if ((textField && !inPanel) || !actions?.approve) return;
        event.preventDefault();
        inspector.current?.approve();
        return;
      }
      if (event.altKey && !command && (event.code === 'KeyK' || event.key.toLowerCase() === 'k')) {
        if ((textField && !inPanel) || !actions?.keep) return;
        event.preventDefault();
        inspector.current?.keep();
        return;
      }
      if (textField || command || event.altKey) return;

      const row = target?.closest<HTMLElement>('.row-select');
      // Nothing focused, a row, or the panel's heading: the places a person
      // browsing the queue has focus. Anywhere else the arrow keys keep
      // their ordinary meaning.
      const browsing = Boolean(row) || !target || target === document.body || target.matches('.inspector h2');
      const move = (step: number) => {
        if (!rows.length) return;
        event.preventDefault();
        const index = rows.findIndex((item) => item.id === selected?.id);
        const next = index < 0 ? (step > 0 ? 0 : rows.length - 1) : Math.max(0, Math.min(rows.length - 1, index + step));
        if (rows[next].id !== selected?.id || row?.dataset.itemId !== rows[next].id) onMove(rows[next]);
      };
      switch (event.key) {
        case 'j': case 'J': move(1); return;
        case 'k': case 'K': move(-1); return;
        case 'ArrowDown': if (browsing) move(1); return;
        case 'ArrowUp': if (browsing) move(-1); return;
        case 'Enter': {
          if (event.shiftKey) return;
          const opened = row ? rows.find((item) => item.id === row.dataset.itemId) : undefined;
          if (opened) { event.preventDefault(); onOpen(opened); return; }
          // Enter on any other control keeps its ordinary meaning.
          if (browsing && selected) { event.preventDefault(); inspector.current?.focus(actions?.approve ? 'filename' : 'heading'); }
          return;
        }
        case 'F2':
          if (!selected) return;
          event.preventDefault();
          inspector.current?.focus(actions?.approve ? 'filename' : 'heading');
          return;
        case 'Delete':
          if (!actions?.remove) return;
          event.preventDefault();
          inspector.current?.requestRemove();
          return;
        case '/':
          if (onFilter()) event.preventDefault();
          return;
      }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, []);
}
