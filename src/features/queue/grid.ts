import { keptOriginal } from '../review/actions';
import { confidence } from '../../lib/format';
import type { QueueItem } from '../../types';

/**
 * The queue table as a spreadsheet: which columns it has, which of them a
 * person can select and copy, what each cell holds as plain text, and the
 * two clipboard formats a copied block is written in.
 *
 * Kept free of React and the DOM so the exact text that reaches Excel can be
 * tested on its own, quoting edge cases and all.
 */

export type ColumnKey = 'original' | 'status' | 'proposed' | 'description' | 'confidence';

export interface GridColumn {
  key: ColumnKey;
  label: string;
  /**
   * Status is never selectable and never copied. It is the one column whose
   * text is the app talking about the document ("Reading document…") rather
   * than something about the document worth putting in a spreadsheet, and it
   * changes under the selection while the queue works.
   */
  copyable: boolean;
}

export const COLUMNS: readonly GridColumn[] = [
  { key: 'original', label: 'Original filename', copyable: true },
  { key: 'status', label: 'Status', copyable: false },
  { key: 'proposed', label: 'Proposed filename', copyable: true },
  { key: 'description', label: 'Description', copyable: true },
  { key: 'confidence', label: 'Confidence', copyable: true },
];

export const COPYABLE_COLUMNS = COLUMNS.filter((column) => column.copyable);

export const columnIndex = (key: ColumnKey) => COLUMNS.findIndex((column) => column.key === key);

/** A cell, named by its row's item and its column, so it survives the queue reordering or growing under it. */
export interface CellRef { id: string; column: ColumnKey }

/**
 * A rectangle of cells. `anchor` is where it started (the click, or the cell
 * Shift was first held from); `focus` is the end that moves as it is extended,
 * and is where keyboard focus sits.
 */
export interface GridSelection { anchor: CellRef; focus: CellRef }

export interface Bounds { top: number; bottom: number; left: number; right: number }

/** The selection as row and column indices into the rows shown; undefined when either end has left the view. */
export function selectionBounds(selection: GridSelection | undefined, rows: readonly QueueItem[]): Bounds | undefined {
  if (!selection) return undefined;
  const anchorRow = rows.findIndex((item) => item.id === selection.anchor.id);
  const focusRow = rows.findIndex((item) => item.id === selection.focus.id);
  if (anchorRow < 0 || focusRow < 0) return undefined;
  const anchorColumn = columnIndex(selection.anchor.column);
  const focusColumn = columnIndex(selection.focus.column);
  return {
    top: Math.min(anchorRow, focusRow),
    bottom: Math.max(anchorRow, focusRow),
    left: Math.min(anchorColumn, focusColumn),
    right: Math.max(anchorColumn, focusColumn),
  };
}

/** Whether a cell is part of the selection. A Status cell never is, even inside the rectangle. */
export function inBounds(bounds: Bounds | undefined, row: number, column: number): boolean {
  return Boolean(bounds && COLUMNS[column].copyable && row >= bounds.top && row <= bounds.bottom && column >= bounds.left && column <= bounds.right);
}

/** Every copyable cell of the rows shown, as one selection: Ctrl+A. */
export function selectAll(rows: readonly QueueItem[]): GridSelection | undefined {
  if (!rows.length) return undefined;
  const first = COPYABLE_COLUMNS[0].key;
  const last = COPYABLE_COLUMNS[COPYABLE_COLUMNS.length - 1].key;
  return { anchor: { id: rows[0].id, column: first }, focus: { id: rows[rows.length - 1].id, column: last } };
}

/**
 * What a cell holds as plain text, as the table shows it but without the
 * decoration: no em dash for "nothing yet", no icon. A filed row gives the
 * name it was filed under, which can differ from the proposal by a " (2)"; a
 * row that kept its own name gives that name, which is what the file is now
 * called, rather than the words "Kept original". Confidence is the same
 * rounded percentage the meter prints.
 */
export function cellText(item: QueueItem, column: ColumnKey): string {
  switch (column) {
    case 'original': return item.originalFilename;
    case 'status': return '';
    case 'proposed':
      if (item.status === 'waiting') return '';
      if (item.status !== 'completed') return item.proposedFilename ?? '';
      if (keptOriginal(item)) return item.originalFilename;
      return item.filedName ?? item.proposedFilename ?? '';
    case 'description': return item.status === 'waiting' ? '' : item.description ?? '';
    case 'confidence': return item.status === 'waiting' || item.confidence === undefined ? '' : confidence(item.confidence);
  }
}

/** One copied block: the column headings it covers, and its rows of cell text. */
export interface CopiedBlock {
  columns: GridColumn[];
  rows: string[][];
  /** Whether the block starts with a row of column headings. */
  header: boolean;
}

/**
 * The cells a selection covers, Status left out wherever the rectangle crosses
 * it: selecting Original filename through Proposed filename copies two
 * columns, not three with a blank between them.
 */
export function selectedBlock(selection: GridSelection | undefined, rows: readonly QueueItem[]): CopiedBlock | undefined {
  const bounds = selectionBounds(selection, rows);
  if (!bounds) return undefined;
  const columns = COLUMNS.filter((column, index) => column.copyable && index >= bounds.left && index <= bounds.right);
  if (!columns.length) return undefined;
  return {
    columns,
    rows: rows.slice(bounds.top, bounds.bottom + 1).map((item) => columns.map((column) => cellText(item, column.key))),
    header: false,
  };
}

/**
 * The whole table as shown - every visible row, every copyable column - with
 * the column headings first. A copied selection carries no headings, as a
 * spreadsheet's does not; a whole table without them would paste as rows of
 * names with nothing to say which column is which.
 */
export function tableBlock(rows: readonly QueueItem[]): CopiedBlock {
  return { columns: [...COPYABLE_COLUMNS], rows: rows.map((item) => COPYABLE_COLUMNS.map((column) => cellText(item, column.key))), header: true };
}

/** The one cell copied with nothing selected: the active cell, as a spreadsheet copies. */
export function cellBlock(cell: CellRef, rows: readonly QueueItem[]): CopiedBlock | undefined {
  if (!COLUMNS[columnIndex(cell.column)]?.copyable) return undefined;
  return selectedBlock({ anchor: cell, focus: cell }, rows);
}

/**
 * Characters a spreadsheet reads, at the start of a cell, as the start of a
 * formula (OWASP's CSV injection list). Filenames come from whoever drops the
 * files, and descriptions and proposed names are model output that a
 * document's own text can steer, so none of them may paste as a formula.
 */
const FORMULA_START = /^[=+\-@\t\r]/;

/**
 * Tab-separated text, as Excel itself puts on the clipboard: a field holding a
 * tab, a line break or a double quote is wrapped in double quotes, with each
 * quote inside doubled, so it lands in one cell instead of spilling into the
 * next column or row. Rows end in CRLF, Excel's own line ending.
 *
 * A text field that would start a formula gets a leading apostrophe, which
 * Excel and Google Sheets both take as "this is text" and do not show. Plain
 * text is what a paste without formatting uses (Keep Text Only,
 * Ctrl+Shift+V), and there the HTML's own protections below do not apply.
 * Confidence is a percentage Intern writes itself and is left as it is.
 */
export function toTsv(block: CopiedBlock): string {
  const quoted = (value: string) => /[\t\r\n"]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
  const field = (value: string, column: GridColumn | undefined) => quoted(column?.key !== 'confidence' && FORMULA_START.test(value) ? `'${value}` : value);
  const lines = block.header ? [block.columns.map((column) => column.label), ...block.rows] : block.rows;
  return lines.map((line) => line.map((value, index) => field(value, block.columns[index])).join('\t')).join('\r\n');
}

const escapeHtml = (value: string) => value
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;');

/**
 * Spaces HTML would collapse: a run of two or more, and any at the start or
 * end of a line. Written as Excel writes them, non-breaking spaces in a
 * `mso-spacerun` span, which Excel pastes back as ordinary spaces - so
 * "Lease  2024.pdf" keeps both of its spaces.
 */
const keepSpaces = (line: string) => line.replace(/^ +| {2,}| +$/g, (run) => `<span style="mso-spacerun:yes">${'&nbsp;'.repeat(run.length)}</span>`);

/**
 * The same block as an HTML table, which Excel and Google Sheets prefer over
 * the plain text when both are on the clipboard.
 *
 * Each spreadsheet is told in its own dialect that a text cell is text, and
 * every other program ignores both hints: Excel reads
 * `mso-number-format:'\@'`, and Google Sheets reads `data-sheets-value` (type
 * 2, a string, holding the exact value). Without them a description starting
 * with "=" would paste as a formula and run, and a filename such as "0012"
 * would become the number 12. Confidence is left alone, so "87%" pastes as a
 * percentage a person can sort. A line break inside a cell is a <br> Excel is
 * told to keep in the same cell; a bare one starts a new row.
 */
export function toHtml(block: CopiedBlock): string {
  const asText = (column: GridColumn, value: string) => column.key === 'confidence'
    ? ''
    : ` style="mso-number-format:'\\@'" data-sheets-value="${escapeHtml(JSON.stringify({ 1: 2, 2: value }))}"`;
  const content = (value: string) => value.split(/\r\n|\r|\n/).map((line) => keepSpaces(escapeHtml(line))).join('<br style="mso-data-placement:same-cell">');
  const head = block.header ? `<thead><tr>${block.columns.map((column) => `<th>${escapeHtml(column.label)}</th>`).join('')}</tr></thead>` : '';
  const body = block.rows.map((row) => `<tr>${row.map((value, index) => `<td${asText(block.columns[index], value)}>${content(value)}</td>`).join('')}</tr>`).join('');
  return `<meta charset="utf-8"><table>${head}<tbody>${body}</tbody></table>`;
}

/** "3 rows" or "3 rows, 2 columns": what a copy put on the clipboard, for the status line. */
export function describeBlock(block: CopiedBlock): string {
  const rows = `${block.rows.length} ${block.rows.length === 1 ? 'row' : 'rows'}`;
  if (block.header) return `the table, ${rows}`;
  if (block.rows.length === 1 && block.columns.length === 1) return '1 cell';
  return `${rows}, ${block.columns.length} ${block.columns.length === 1 ? 'column' : 'columns'}`;
}
