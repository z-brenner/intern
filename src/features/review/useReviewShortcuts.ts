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
  /** F2, or Enter where nothing else has focus: open the Details panel for the item and go to its name. */
  onOpen(item: QueueItem): void;
  /** Focus the filter box; false when there is none to focus. */
  onFilter(): boolean;
  /** Whether the queue table's selected cells span more than one row, when Delete is ignored. */
  selectionSpansRows?(): boolean;
  /** Delete was pressed with cells of several rows selected, and did nothing: say why. */
  onDeleteRefused?(): void;
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
 * - J/K move the selected row, from anywhere; Up/Down do the same when
 *   nothing in particular has focus. On a cell of the table the arrow keys
 *   are the table's own (QueueTable), moving between cells;
 * - Enter on a cell (QueueTable), or F2 anywhere, opens the Details panel
 *   for the selected item and goes to its name;
 * - Ctrl+Enter approves and Alt+K keeps the original - from the review
 *   panel's own fields too, since after each decision the caret is already
 *   in the next item's name;
 * - Delete removes, after asking - but not while the table has cells of
 *   several rows selected, which it would seem to remove as well;
 * - "/" goes to the filter.
 *
 * Approve, keep and remove act only while the Details panel is open, and
 * only where it offers the matching button, so a shortcut can never do what
 * a click could not: with the panel closed there is no button to click. The
 * keys that decide an item act on the press, not on its repeats: review
 * moves on after each decision, so a held key would go on to decide the
 * documents after it, unseen.
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
      const { enabled, rows, selected, inspector, onMove, onOpen, onFilter, selectionSpansRows, onDeleteRefused } = latest.current;
      // A key a field or dialog already handled (Enter in the name, Escape
      // in the filter) is not a shortcut as well.
      if (!enabled || event.defaultPrevented || event.isComposing) return;
      const target = event.target instanceof Element ? event.target : null;
      const textField = isTextField(target);
      const inPanel = Boolean(target?.closest('.inspector'));
      // The panel is mounted only while it is open, so its handle is the
      // test of whether its buttons are on screen to be pressed.
      const actions = selected && inspector.current ? itemActions(selected) : undefined;
      const command = event.ctrlKey || event.metaKey;

      if (event.key === 'Enter' && command && !event.altKey && !event.shiftKey) {
        if ((textField && !inPanel) || !actions?.approve) return;
        event.preventDefault();
        if (!event.repeat) inspector.current?.approve();
        return;
      }
      if (event.altKey && !command && (event.code === 'KeyK' || event.key.toLowerCase() === 'k')) {
        if ((textField && !inPanel) || !actions?.keep) return;
        event.preventDefault();
        if (!event.repeat) inspector.current?.keep();
        return;
      }
      if (textField || command || event.altKey) return;

      const row = target?.closest<HTMLElement>('.queue-grid [data-item-id]');
      // Nothing focused, a cell, or the panel's heading: the places a person
      // browsing the queue has focus. Anywhere else the arrow keys keep
      // their ordinary meaning. (A cell's arrows never reach here: the table
      // handles them first.)
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
        case 'Enter':
          // Enter on any other control keeps its ordinary meaning.
          if (event.shiftKey || !browsing || !selected) return;
          event.preventDefault();
          onOpen(selected);
          return;
        case 'F2':
          if (!selected) return;
          event.preventDefault();
          onOpen(selected);
          return;
        case 'Delete':
          if (!actions?.remove) return;
          if (selectionSpansRows?.()) { event.preventDefault(); if (!event.repeat) onDeleteRefused?.(); return; }
          event.preventDefault();
          if (!event.repeat) inspector.current?.requestRemove();
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
