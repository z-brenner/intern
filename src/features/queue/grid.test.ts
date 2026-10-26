import { describe, expect, it } from 'vitest';
import { cellText, describeBlock, inBounds, selectAll, selectedBlock, selectionBounds, tableBlock, toHtml, toTsv } from './grid';
import type { CopiedBlock, GridSelection } from './grid';
import type { QueueItem } from '../../types';

const rows: QueueItem[] = [
  { id: 'a', originalFilename: 'Scan 0001.pdf', status: 'review', proposedFilename: '2024-01-02 Lease.pdf', description: 'A lease.', confidence: 0.874 },
  { id: 'b', originalFilename: 'Scan 0002.pdf', status: 'ready', proposedFilename: '2024-02-03 Invoice.pdf', description: 'An invoice.', confidence: 0.95 },
  { id: 'c', originalFilename: 'Scan 0003.pdf', status: 'waiting' },
];

const range = (anchor: [string, GridSelection['anchor']['column']], focus: [string, GridSelection['focus']['column']]): GridSelection => ({
  anchor: { id: anchor[0], column: anchor[1] },
  focus: { id: focus[0], column: focus[1] },
});

describe('queue grid', () => {
  it('copies a cell as the table shows it, without its decoration', () => {
    expect(cellText(rows[0], 'original')).toBe('Scan 0001.pdf');
    expect(cellText(rows[0], 'proposed')).toBe('2024-01-02 Lease.pdf');
    expect(cellText(rows[0], 'description')).toBe('A lease.');
    // The same rounded percentage the confidence meter prints.
    expect(cellText(rows[0], 'confidence')).toBe('87%');
    // Nothing yet is empty, not an em dash.
    expect(['proposed', 'description', 'confidence'].map((column) => cellText(rows[2], column as 'proposed'))).toEqual(['', '', '']);
    expect(cellText({ ...rows[0], confidence: undefined }, 'confidence')).toBe('');
    // Status is never copied.
    expect(cellText(rows[0], 'status')).toBe('');
  });

  it('copies the name a filed document has now', () => {
    const filed: QueueItem = { id: 'f', originalFilename: 'Scan 9.pdf', status: 'completed', proposedFilename: '2024-01-02 Lease.pdf', filedName: '2024-01-02 Lease (2).pdf', undoable: true };
    expect(cellText(filed, 'proposed')).toBe('2024-01-02 Lease (2).pdf');
    // A kept original is called what it was called: not "Kept original", and
    // not the proposal that was never applied.
    const kept: QueueItem = { id: 'k', originalFilename: 'Board minutes.docx', status: 'completed', proposedFilename: '2024-03-01 Minutes.docx', keptOriginal: true };
    expect(cellText(kept, 'proposed')).toBe('Board minutes.docx');
  });

  it('skips Status wherever a selection crosses it', () => {
    const block = selectedBlock(range(['a', 'original'], ['b', 'proposed']), rows)!;
    expect(block.columns.map((column) => column.key)).toEqual(['original', 'proposed']);
    expect(block.rows).toEqual([['Scan 0001.pdf', '2024-01-02 Lease.pdf'], ['Scan 0002.pdf', '2024-02-03 Invoice.pdf']]);
    expect(block.header).toBe(false);

    // Dragged backwards, bottom-right to top-left, it is the same rectangle.
    expect(selectedBlock(range(['b', 'proposed'], ['a', 'original']), rows)).toEqual(block);

    const bounds = selectionBounds(range(['a', 'original'], ['b', 'proposed']), rows);
    expect(inBounds(bounds, 0, 1)).toBe(false);
    expect(inBounds(bounds, 0, 0)).toBe(true);
    expect(inBounds(bounds, 2, 0)).toBe(false);
  });

  it('forgets a selection whose corner has left the view', () => {
    expect(selectionBounds(range(['a', 'original'], ['gone', 'proposed']), rows)).toBeUndefined();
    expect(selectedBlock(range(['a', 'original'], ['gone', 'proposed']), rows)).toBeUndefined();
  });

  it('selects every copyable cell with Ctrl+A', () => {
    const block = selectedBlock(selectAll(rows), rows)!;
    expect(block.columns.map((column) => column.label)).toEqual(['Original filename', 'Proposed filename', 'Description', 'Confidence']);
    expect(block.rows).toHaveLength(3);
    expect(selectAll([])).toBeUndefined();
  });

  it('copies the whole table with its headings, and a selection without', () => {
    const table = tableBlock(rows);
    expect(toTsv(table)).toBe([
      'Original filename\tProposed filename\tDescription\tConfidence',
      'Scan 0001.pdf\t2024-01-02 Lease.pdf\tA lease.\t87%',
      'Scan 0002.pdf\t2024-02-03 Invoice.pdf\tAn invoice.\t95%',
      'Scan 0003.pdf\t\t\t',
    ].join('\r\n'));
    expect(describeBlock(table)).toBe('the table, 3 rows');
    const one = selectedBlock(range(['b', 'description'], ['b', 'description']), rows)!;
    expect(toTsv(one)).toBe('An invoice.');
    expect(describeBlock(one)).toBe('1 cell');
    expect(describeBlock(selectedBlock(range(['a', 'original'], ['b', 'proposed']), rows)!)).toBe('2 rows, 2 columns');
  });

  it('quotes a field as Excel does, so it stays in one cell', () => {
    const block: CopiedBlock = {
      columns: tableBlock([]).columns,
      rows: [[
        'tab\there',
        'line one\nline two',
        'She said "hello"',
        'Crème brûlée — 東京 🗂️',
      ], ['"', '\r\n', 'plain', '']],
      header: false,
    };
    expect(toTsv(block)).toBe([
      '"tab\there"\t"line one\nline two"\t"She said ""hello"""\tCrème brûlée — 東京 🗂️',
      '""""\t"\r\n"\tplain\t',
    ].join('\r\n'));
  });

  it('writes an HTML table that pastes into separate cells', () => {
    const block: CopiedBlock = {
      columns: tableBlock([]).columns,
      rows: [['<b>&"\'', 'first\nsecond', '=HYPERLINK("x")', '87%']],
      header: true,
    };
    const html = toHtml(block);
    expect(html).toBe(
      '<meta charset="utf-8"><table>'
      + '<thead><tr><th>Original filename</th><th>Proposed filename</th><th>Description</th><th>Confidence</th></tr></thead>'
      + '<tbody><tr>'
      + '<td style="mso-number-format:\'\\@\'">&lt;b&gt;&amp;&quot;&#39;</td>'
      + '<td style="mso-number-format:\'\\@\'">first<br style="mso-data-placement:same-cell">second</td>'
      + '<td style="mso-number-format:\'\\@\'">=HYPERLINK(&quot;x&quot;)</td>'
      + '<td>87%</td>'
      + '</tr></tbody></table>',
    );
    // Parsed back, it is the same cells.
    const parsed = new DOMParser().parseFromString(html, 'text/html');
    expect([...parsed.querySelectorAll('tbody td')].map((cell) => cell.textContent)).toEqual(['<b>&"\'', 'firstsecond', '=HYPERLINK("x")', '87%']);
    expect(toHtml({ ...block, rows: [['東京 🗂️', '', '', '']], header: false })).toContain('東京 🗂️');
  });
});
