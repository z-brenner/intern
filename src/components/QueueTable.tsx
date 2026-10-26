import { useEffect, useRef } from 'react';
import type { KeyboardEvent, MouseEvent } from 'react';
import { keptOriginal } from '../features/review/actions';
import { COLUMNS, COPYABLE_COLUMNS, columnIndex, inBounds, selectAll, selectionBounds } from '../features/queue/grid';
import type { CellRef, ColumnKey, GridSelection } from '../features/queue/grid';
import type { QueueItem } from '../types';
import { ConfidenceMeter } from './ConfidenceMeter';
import { FileKindIcon } from './FileKindIcon';
import { StatusCell } from './StatusCell';

interface Props {
  items: QueueItem[];
  /** The item the Details panel describes; its row is marked whether or not any cell is selected. */
  selectedId?: string;
  /** The selected cells, if any. Nothing selected is a real state: Copy then copies the whole table. */
  selection?: GridSelection;
  /** The cell that takes Tab when no cell is selected, so the arrows carry on from where they were. */
  restingCell?: CellRef;
  /** `active`: where focus now rests, when that is not the selection's moving end (a click on Status). */
  onSelectionChange(selection: GridSelection | undefined, active?: CellRef): void;
  /** A cell was clicked or arrowed to: its row becomes the selected item. Never opens the Details panel. */
  onSelectRow(item: QueueItem, cell: HTMLElement): void;
  /** Enter on a cell: open the Details panel for its row. */
  onOpen(item: QueueItem, cell: HTMLElement): void;
}

/** Rows that Page Up and Page Down move by. */
const PAGE_ROWS = 10;

/**
 * The queue as a grid of cells that behaves like a spreadsheet's: click a
 * cell to select it, Shift+click or drag for a rectangle, Ctrl+A for all of
 * it, and Ctrl+C (or the Copy button above the table) to copy it in a form
 * Excel and Google Sheets paste into separate cells. Status is never part of
 * a selection.
 *
 * It is one Tab stop - the active cell - and the arrow keys move between
 * cells (WAI-ARIA grid pattern). Every row used to be a tab stop of its own,
 * so the review panel after the table was a row count of Tab presses away.
 * J/K, which move between rows from anywhere, stay in useReviewShortcuts.
 *
 * Selecting never opens the Details panel; the toolbar's Details button and
 * Enter on a cell do.
 */
export function QueueTable({ items, selectedId, selection, restingCell, onSelectionChange, onSelectRow, onOpen }: Props) {
  const tableRef = useRef<HTMLTableElement>(null);
  const dragging = useRef(false);
  const bounds = selectionBounds(selection, items);
  // The cell that takes Tab: the moving end of the selection, or with nothing
  // selected, where focus last rested (by default the selected item's row).
  const active: CellRef | undefined = bounds && selection ? selection.focus : restingCell;

  useEffect(() => {
    const stop = () => { dragging.current = false; };
    document.addEventListener('mouseup', stop);
    return () => document.removeEventListener('mouseup', stop);
  }, []);

  const cellElement = (cell: CellRef) => findCell(tableRef.current, cell);
  const focusCell = (cell: CellRef, options?: FocusOptions) => cellElement(cell)?.focus(options);
  // A Status cell is never a selection's corner: a corner that would land on
  // one keeps the column it had, or failing that takes the first copyable one.
  const copyableColumn = (column: ColumnKey, fallback?: ColumnKey): ColumnKey => COLUMNS[columnIndex(column)].copyable ? column : fallback ?? COPYABLE_COLUMNS[0].key;

  const select = (next: GridSelection, row: QueueItem | undefined, element: HTMLElement | undefined) => {
    onSelectionChange(next);
    if (row && element && row.id !== selectedId) onSelectRow(row, element);
  };

  const onCellMouseDown = (event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey) => {
    if (event.button !== 0) return;
    // The table draws its own selection; the browser's text selection would
    // be a second, different one on top of it.
    event.preventDefault();
    const target = { id: item.id, column: copyableColumn(column, active?.column) };
    if (event.shiftKey && active) {
      const anchor = bounds && selection ? selection.anchor : active;
      onSelectionChange({ anchor, focus: target });
    } else if (!COLUMNS[columnIndex(column)].copyable) {
      // Status cannot be selected. A click on it still says "this row", so it
      // selects the row's item and puts focus in the row, selecting nothing.
      onSelectionChange(undefined, target);
      const cell = cellElement(target);
      if (cell && item.id !== selectedId) onSelectRow(item, cell);
    } else {
      dragging.current = true;
      onSelectionChange({ anchor: target, focus: target });
      if (item.id !== selectedId) onSelectRow(item, event.currentTarget);
    }
    focusCell(target, { preventScroll: true });
  };

  const onCellMouseEnter = (event: MouseEvent<HTMLElement>, item: QueueItem, column: ColumnKey) => {
    if (!dragging.current || !selection) return;
    if ((event.buttons & 1) === 0) { dragging.current = false; return; }
    const focus = { id: item.id, column: copyableColumn(column, selection.focus.column) };
    if (focus.id === selection.focus.id && focus.column === selection.focus.column) return;
    onSelectionChange({ anchor: selection.anchor, focus });
    focusCell(focus, { preventScroll: true });
  };

  // A heading selects its column, or with Shift extends the selection across
  // to it. The keyboard has Ctrl+Space for the same.
  const selectColumn = (column: ColumnKey, extend: boolean) => {
    const first = items[0];
    const last = items.at(-1);
    if (!first || !last || !COLUMNS[columnIndex(column)].copyable) return;
    const from = extend && bounds && selection ? selection.anchor.column : column;
    // Focus ends at the top, so selecting a column does not scroll the table to its foot.
    const next = { anchor: { id: last.id, column: from }, focus: { id: first.id, column } };
    onSelectionChange(next);
    focusCell(next.focus, { preventScroll: true });
  };

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
      const all = selectAll(items);
      if (all) onSelectionChange(all);
      return;
    }
    if (event.key === 'Escape') {
      if (!selection) return;
      event.preventDefault();
      onSelectionChange(undefined);
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
      if (event.shiftKey && !command) {
        // Shift+Space: the whole row, every copyable column of it.
        onSelectionChange({ anchor: { id: here.id, column: COPYABLE_COLUMNS[0].key }, focus: { id: here.id, column: COPYABLE_COLUMNS.at(-1)!.key } });
      }
      return;
    }

    // Where an arrow, Home, End or Page key goes. Left and right step over
    // Status; Ctrl goes to the edge, as in a spreadsheet.
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
      onSelectionChange({ anchor: bounds && selection ? selection.anchor : here, focus: target });
      focusCell(target);
      return;
    }
    select({ anchor: target, focus: target }, items[nextRow], cellElement(target));
    focusCell(target);
  };

  // Read out once a selection is bigger than the cell focus is on, which a
  // screen reader's own "selected" on that cell cannot convey.
  const selectedRows = bounds ? bounds.bottom - bounds.top + 1 : 0;
  const selectedColumns = bounds ? COLUMNS.slice(bounds.left, bounds.right + 1).filter((column) => column.copyable).length : 0;
  const summary = selectedRows * selectedColumns > 1
    ? `${selectedRows} ${selectedRows === 1 ? 'row' : 'rows'} by ${selectedColumns} ${selectedColumns === 1 ? 'column' : 'columns'} selected`
    : '';

  return <div className="table-wrap">
    <table ref={tableRef} className="queue-grid" role="grid" aria-label="Queue" aria-multiselectable="true" aria-readonly="true" onKeyDown={onKeyDown}>
      <thead><tr role="row">{COLUMNS.map((column) => <th key={column.key} role="columnheader" scope="col" className={`col-${column.key}`}
        onMouseDown={column.copyable ? (event) => { if (event.button === 0) { event.preventDefault(); selectColumn(column.key, event.shiftKey); } } : undefined}
        title={column.copyable ? `Select the ${column.label} column` : undefined}>{column.label}</th>)}</tr></thead>
      <tbody>{items.map((item, rowIndex) => <tr key={item.id} role="row" className={selectedId === item.id ? 'selected' : undefined} aria-current={selectedId === item.id || undefined}>
        {COLUMNS.map((column, columnIndexValue) => {
          const selected = inBounds(bounds, rowIndex, columnIndexValue);
          const edge = bounds && rowIndex >= bounds.top && rowIndex <= bounds.bottom && columnIndexValue >= bounds.left && columnIndexValue <= bounds.right
            ? [rowIndex === bounds.top && 'edge-top', rowIndex === bounds.bottom && 'edge-bottom', columnIndexValue === bounds.left && 'edge-left', columnIndexValue === bounds.right && 'edge-right'].filter(Boolean).join(' ')
            : '';
          const isActive = active?.id === item.id && active.column === column.key;
          const className = [`col-${column.key}`, column.key === 'confidence' ? `confidence ${item.status}` : '', selected ? 'cell-selected' : '', edge].filter(Boolean).join(' ');
          return <td key={column.key} role="gridcell" className={className} data-item-id={item.id} data-column={column.key}
            tabIndex={column.copyable ? (isActive ? 0 : -1) : undefined}
            aria-selected={column.copyable ? selected : undefined}
            title={column.key === 'description' && item.status !== 'waiting' ? item.description : undefined}
            onMouseDown={(event) => onCellMouseDown(event, item, column.key)}
            onMouseEnter={(event) => onCellMouseEnter(event, item, column.key)}>
            {cellContent(item, column.key)}
          </td>;
        })}
      </tr>)}</tbody>
    </table>
    <p className="sr-only" role="status" aria-label="Cell selection" aria-live="polite">{summary}</p>
  </div>;
}

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
    case 'confidence': return item.status === 'waiting' ? '—' : <ConfidenceMeter value={item.confidence} status={item.status} />;
  }
}

/**
 * A filed row shows the name the file was given, which can differ from the
 * proposal (a " (2)" the destination needed). A kept original shows that it
 * kept its name, not a proposal that was never applied.
 */
function nameCell(item: QueueItem) {
  if (item.status !== 'completed') return item.proposedFilename ?? '—';
  if (keptOriginal(item)) return <span className="kept-original">Kept original</span>;
  return item.filedName ?? item.proposedFilename ?? '—';
}
