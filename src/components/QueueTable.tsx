import { memo, useEffect, useId, useLayoutEffect, useMemo, useRef, useSyncExternalStore } from 'react';
import type { KeyboardEvent, MouseEvent } from 'react';
import { keptOriginal } from '../features/review/actions';
import { COLUMNS, COPYABLE_COLUMNS, columnIndex, selectAll, selectionBounds } from '../features/queue/grid';
import type { CellRef, ColumnKey } from '../features/queue/grid';
import { selectionOf } from '../features/queue/gridStore';
import type { GridState, GridStore } from '../features/queue/gridStore';
import type { QueueItem } from '../types';
import { checkCell, itemFlaggedFields } from '../features/queue/check';
import { FlaggedName } from './FlaggedName';
import { FileKindIcon } from './FileKindIcon';
import { StatusCell } from './StatusCell';

interface Props {
  items: QueueItem[];
  /** The item the Details panel describes; its row is marked whether or not any cell is selected. */
  selectedId?: string;
  /** The active cell and the selection (see gridStore). */
  store: GridStore;
  /** A cell was clicked or arrowed to: its row becomes the selected item. Never opens the Details panel. */
  onSelectRow(item: QueueItem, cell: HTMLElement): void;
  /** Enter on a cell: open the Details panel for its row. */
  onOpen(item: QueueItem, cell: HTMLElement): void;
}

/** Rows that Page Up and Page Down move by. */
const PAGE_ROWS = 10;

const FIRST_COLUMN = COPYABLE_COLUMNS[0].key;
const LAST_COLUMN = COPYABLE_COLUMNS[COPYABLE_COLUMNS.length - 1].key;

/** A Status cell is never a selection's corner: a corner that would land on one keeps the column it had. */
const copyableColumn = (column: ColumnKey, fallback?: ColumnKey): ColumnKey => COLUMNS[columnIndex(column)].copyable ? column : fallback ?? FIRST_COLUMN;

/**
 * The queue as a grid of cells that behaves like a spreadsheet's: click a
 * cell to select it, Shift+click or drag for a rectangle, Ctrl+A for all of
 * it, and Ctrl+C (or the Copy button above the table) to copy it in a form
 * Excel and Google Sheets paste into separate cells. Status is never part of
 * a selection.
 *
 * It is one Tab stop - the active cell, which keyboard focus always follows -
 * and the arrow keys move between cells (WAI-ARIA grid pattern). Every row
 * used to be a tab stop of its own, so the review panel after the table was
 * a row count of Tab presses away. J/K, which move between rows from
 * anywhere, stay in useReviewShortcuts.
 *
 * Selecting never opens the Details panel; the toolbar's Details button and
 * Enter on a cell do.
 */
export function QueueTable({ items, selectedId, store, onSelectRow, onOpen }: Props) {
  const tableRef = useRef<HTMLTableElement>(null);
  const idPrefix = useId();
  const dragging = useRef(false);
  const state = useSyncExternalStore(store.subscribe, store.get);
  const selection = selectionOf(state);
  const bounds = selectionBounds(selection, items);
  // The cell that takes Tab: the active cell while its row is shown, or the
  // selected item's row (the first row before there is one) in its column.
  const focusRow = state.focus ? items.findIndex((item) => item.id === state.focus!.id) : -1;
  const restingRow = items.find((item) => item.id === selectedId) ?? items[0];
  const active: CellRef | undefined = focusRow >= 0 ? state.focus : restingRow && { id: restingRow.id, column: state.focus?.column ?? FIRST_COLUMN };

  // A selection whose corner has left the view (filed, filtered out) is gone,
  // rather than coming back if that row ever returns.
  // Read afresh: the selected item can have moved the active cell since this
  // render, and putting back the one this render saw would undo that.
  useEffect(() => {
    const current = store.get();
    if (current.anchor && !selectionBounds(selectionOf(current), items)) store.set({ focus: current.focus, anchor: undefined });
  });

  useEffect(() => {
    const stop = () => { dragging.current = false; };
    document.addEventListener('mouseup', stop);
    return () => document.removeEventListener('mouseup', stop);
  }, []);

  const cellElement = (cell: CellRef) => findCell(tableRef.current, cell);
  const focusCell = (cell: CellRef, options?: FocusOptions) => cellElement(cell)?.focus(options);
  // Every change of the active cell goes through here, so keyboard focus is
  // always on it and Tab always comes back to it.
  const moveTo = (next: GridState, options?: FocusOptions) => {
    store.set(next);
    if (next.focus) focusCell(next.focus, options);
  };

  // The rows are memoized and get their handlers from here, through a ref,
  // so a row whose cells did not change is not rendered again.
  const onCellMouseDown = (event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey) => {
    if (event.button !== 0) return;
    // The table draws its own selection; the browser's text selection would
    // be a second, different one on top of it.
    event.preventDefault();
    const target = { id: item.id, column: copyableColumn(column, active?.column) };
    if (event.shiftKey && active) {
      moveTo({ anchor: bounds ? state.anchor : active, focus: target }, { preventScroll: true });
    } else if (!COLUMNS[columnIndex(column)].copyable) {
      // Status cannot be selected. A click on it still says "this row", so it
      // selects the row's item and puts focus in the row, selecting nothing.
      moveTo({ focus: target }, { preventScroll: true });
      const cell = cellElement(target);
      if (cell && item.id !== selectedId) onSelectRow(item, cell);
    } else {
      dragging.current = true;
      moveTo({ anchor: target, focus: target }, { preventScroll: true });
      if (item.id !== selectedId) onSelectRow(item, event.currentTarget);
    }
  };

  const onCellMouseEnter = (event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey) => {
    if (!dragging.current || !state.anchor || !state.focus) return;
    if ((event.buttons & 1) === 0) { dragging.current = false; return; }
    const focus = { id: item.id, column: copyableColumn(column, state.focus.column) };
    if (focus.id === state.focus.id && focus.column === state.focus.column) return;
    moveTo({ anchor: state.anchor, focus }, { preventScroll: true });
  };

  // A heading selects its column, or with Shift extends the selection across
  // to it. The keyboard has Ctrl+Space for the same.
  const selectColumn = (column: ColumnKey, extend: boolean) => {
    const first = items[0];
    const last = items.at(-1);
    if (!first || !last || !COLUMNS[columnIndex(column)].copyable) return;
    const from = extend && bounds && state.anchor ? state.anchor.column : column;
    // The active cell ends at the top, so selecting a column does not scroll the table to its foot.
    moveTo({ anchor: { id: last.id, column: from }, focus: { id: first.id, column } }, { preventScroll: true });
  };

  const handlers = useRef({ onCellMouseDown, onCellMouseEnter });
  useLayoutEffect(() => { handlers.current = { onCellMouseDown, onCellMouseEnter }; });
  const stableHandlers = useMemo<RowHandlers>(() => ({
    mouseDown: (event, item, column) => handlers.current.onCellMouseDown(event, item, column),
    mouseEnter: (event, item, column) => handlers.current.onCellMouseEnter(event, item, column),
  }), []);

  const onKeyDown = (event: KeyboardEvent<HTMLTableElement>) => {
    const element = (event.target as HTMLElement).closest<HTMLElement>('[data-item-id][data-column]');
    if (!element || event.altKey || event.nativeEvent.isComposing) return;
    const row = items.findIndex((item) => item.id === element.dataset.itemId);
    const column = columnIndex(element.dataset.column as ColumnKey);
    if (row < 0 || column < 0) return;
    const command = event.ctrlKey || event.metaKey;
    const here: CellRef = { id: items[row].id, column: COLUMNS[column].key };

    if (command && !event.shiftKey && event.key.toLowerCase() === 'a') {
      event.preventDefault();
      // As in a spreadsheet, the active end is the far corner: Shift+Up then
      // gives back the last row, and nothing scrolls.
      const all = selectAll(items);
      if (all) moveTo({ anchor: all.anchor, focus: all.focus }, { preventScroll: true });
      return;
    }
    if (event.key === 'Escape') {
      if (!state.anchor) return;
      event.preventDefault();
      store.set({ focus: here, anchor: undefined });
      return;
    }
    if (event.key === 'Enter' && !command && !event.shiftKey) {
      event.preventDefault();
      onOpen(items[row], element);
      return;
    }
    if (event.key === ' ' && (command || event.shiftKey)) {
      event.preventDefault();
      if (command && !event.shiftKey) { selectColumn(here.column, false); return; }
      // Shift+Space: the whole row, every copyable column of it, active at its end.
      if (event.shiftKey && !command) moveTo({ anchor: { id: here.id, column: FIRST_COLUMN }, focus: { id: here.id, column: LAST_COLUMN } }, { preventScroll: true });
      return;
    }

    // Where an arrow, Home, End or Page key goes, from the active cell. Left
    // and right step over Status; Ctrl goes to the edge, as in a spreadsheet.
    const copyable = COLUMNS.flatMap((entry, index) => entry.copyable ? [index] : []);
    const at = copyable.indexOf(column);
    let nextRow = row;
    let nextColumn = column;
    switch (event.key) {
      case 'ArrowUp': nextRow = command ? 0 : row - 1; break;
      case 'ArrowDown': nextRow = command ? items.length - 1 : row + 1; break;
      case 'ArrowLeft': nextColumn = command ? copyable[0] : copyable[Math.max(0, at - 1)]; break;
      case 'ArrowRight': nextColumn = command ? copyable.at(-1)! : copyable[Math.min(copyable.length - 1, at + 1)]; break;
      case 'Home': nextColumn = copyable[0]; if (command) nextRow = 0; break;
      case 'End': nextColumn = copyable.at(-1)!; if (command) nextRow = items.length - 1; break;
      case 'PageUp': nextRow = row - PAGE_ROWS; break;
      case 'PageDown': nextRow = row + PAGE_ROWS; break;
      default: return;
    }
    event.preventDefault();
    nextRow = Math.max(0, Math.min(items.length - 1, nextRow));
    const target: CellRef = { id: items[nextRow].id, column: COLUMNS[nextColumn].key };
    if (event.shiftKey) {
      moveTo({ anchor: bounds ? state.anchor : here, focus: target });
      return;
    }
    moveTo({ anchor: target, focus: target });
    if (items[nextRow].id !== selectedId) onSelectRow(items[nextRow], cellElement(target)!);
  };

  // Read out once a selection is bigger than the cell focus is on, which a
  // screen reader's own "selected" on that cell cannot convey.
  const selectedRows = bounds ? bounds.bottom - bounds.top + 1 : 0;
  const selectedColumns = bounds ? COLUMNS.slice(bounds.left, bounds.right + 1).filter((column) => column.copyable).length : 0;
  const multiple = selectedRows * selectedColumns > 1;
  const summary = multiple ? `${selectedRows} ${selectedRows === 1 ? 'row' : 'rows'} by ${selectedColumns} ${selectedColumns === 1 ? 'column' : 'columns'} selected` : '';

  return <div className="table-wrap">
    <table ref={tableRef} className="queue-grid" role="grid" aria-label="Queue" aria-multiselectable="true" aria-readonly="true" onKeyDown={onKeyDown}>
      <thead><tr role="row">{COLUMNS.map((column) => <th key={column.key} role="columnheader" scope="col" className={`col-${column.key}`}
        onMouseDown={column.copyable ? (event) => { if (event.button === 0) { event.preventDefault(); selectColumn(column.key, event.shiftKey); } } : undefined}
        title={column.copyable ? `Select the ${column.label} column` : undefined}>{column.label}</th>)}</tr></thead>
      <tbody>{items.map((item, rowIndex) => {
        const inRows = bounds !== undefined && rowIndex >= bounds.top && rowIndex <= bounds.bottom;
        return <QueueRow key={item.id} item={item} statusId={`${idPrefix}-status-${rowIndex}`} current={selectedId === item.id}
          left={inRows ? bounds.left : -1} right={inRows ? bounds.right : -1} top={inRows && rowIndex === bounds.top} bottom={inRows && rowIndex === bounds.bottom}
          tabColumn={active?.id === item.id ? active.column : undefined} marked={multiple && active?.id === item.id ? active.column : undefined}
          handlers={stableHandlers} />;
      })}</tbody>
    </table>
    <p className="sr-only" role="status" aria-label="Cell selection" aria-live="polite">{summary}</p>
  </div>;
}

interface RowHandlers {
  mouseDown(event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey): void;
  mouseEnter(event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey): void;
}

interface RowProps {
  item: QueueItem;
  statusId: string;
  /** The row of the item the Details panel describes. */
  current: boolean;
  /** The selected columns of this row, by index; -1 when none are. */
  left: number;
  right: number;
  /** Whether the selection's top or bottom edge runs along this row. */
  top: boolean;
  bottom: boolean;
  /** The column of this row's cell that takes Tab, if it has it. */
  tabColumn?: ColumnKey;
  /** The active cell inside a selection of several cells, marked as a spreadsheet marks it. */
  marked?: ColumnKey;
  handlers: RowHandlers;
}

/**
 * One row, given only its own part of the selection, so dragging across a
 * long queue renders the rows whose cells change rather than all of them.
 */
const QueueRow = memo(function QueueRow({ item, statusId, current, left, right, top, bottom, tabColumn, marked, handlers }: RowProps) {
  return <tr role="row" className={current ? 'selected' : undefined} aria-current={current || undefined}>
    {COLUMNS.map((column, index) => {
      const inRange = left >= 0 && index >= left && index <= right;
      const selected = inRange && column.copyable;
      const className = [
        `col-${column.key}`,
        selected ? 'cell-selected' : '',
        marked === column.key ? 'cell-active' : '',
        inRange && top ? 'edge-top' : '',
        inRange && bottom ? 'edge-bottom' : '',
        inRange && index === left ? 'edge-left' : '',
        inRange && index === right ? 'edge-right' : '',
      ].filter(Boolean).join(' ');
      // The arrow keys step over Status, so from the keyboard a screen
      // reader would never reach it. The filename cell is described by it
      // instead: moving down the first column reads each status.
      return <td key={column.key} role="gridcell" className={className} data-item-id={item.id} data-column={column.key}
        id={column.key === 'status' ? statusId : undefined}
        aria-describedby={column.key === 'original' ? statusId : undefined}
        tabIndex={column.copyable ? (tabColumn === column.key ? 0 : -1) : undefined}
        aria-selected={column.copyable ? selected : undefined}
        title={column.key === 'description' && item.status !== 'waiting' ? item.description : column.key === 'check' ? checkCell(item)?.title : undefined}
        onMouseDown={(event) => handlers.mouseDown(event, item, column.key)}
        onMouseEnter={(event) => handlers.mouseEnter(event, item, column.key)}>
        {cellContent(item, column.key)}
      </td>;
    })}
  </tr>;
});

/** A cell of the grid by item and column, matched on its data attributes rather than a selector an id could break. */
export function findCell(root: ParentNode | null | undefined, cell: CellRef): HTMLElement | undefined {
  if (!root) return undefined;
  return [...root.querySelectorAll<HTMLElement>(`.queue-grid [data-column="${cell.column}"]`)].find((element) => element.dataset.itemId === cell.id);
}

function cellContent(item: QueueItem, column: ColumnKey) {
  switch (column) {
    case 'original': return <span className="cell-filename"><FileKindIcon filename={item.originalFilename} /><span>{item.originalFilename}</span></span>;
    case 'status': return <StatusCell item={item} />;
    case 'proposed': return item.status === 'waiting' ? '—' : nameCell(item);
    // The description is half of what Intern produces. It has a column of its
    // own so it copies into a column of its own; a long one is cut to a few
    // lines here, with the whole sentence on hover and in the Details panel.
    case 'description': return item.status === 'waiting' || !item.description ? '—' : <span className="row-description">{item.description}</span>;
    case 'check': return checkContent(item);
  }
}

/**
 * A filed row shows the name the file was given, which can differ from the
 * proposal (a " (2)" the destination needed). A kept original shows that it
 * kept its name, not a proposal that was never applied.
 */
function nameCell(item: QueueItem) {
  if (item.status !== 'completed') return item.proposedFilename ? <FlaggedName name={item.proposedFilename} flagged={itemFlaggedFields(item)} /> : '—';
  if (keptOriginal(item)) return <span className="kept-original">Kept original</span>;
  return item.filedName ?? item.proposedFilename ?? '—';
}

/** What to check, in a few words, toned as the status is; an em dash before there is anything to say. */
function checkContent(item: QueueItem) {
  const cell = checkCell(item);
  if (!cell) return '—';
  return <span className={`check-label ${cell.tone}`}>{cell.text}{cell.more > 0 && <small> +{cell.more} more</small>}</span>;
}
