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
      // A value starting with a line break would start a formula too.
      '""""\t"\'\r\n"\tplain\t',
    ].join('\r\n'));
  });

  // Filenames come from whoever drops the files, and descriptions are model
  // output a document can steer. A paste without formatting (Keep Text Only,
  // Ctrl+Shift+V) reads the plain text, where "=..." used to run as a formula.
  it('never lets a text cell paste as a formula from the plain text', () => {
    const block: CopiedBlock = {
      columns: tableBlock([]).columns,
      rows: [
        ['=HYPERLINK("http://x","click")', '+1 Lease.pdf', '-2 Notes', '@SUM(A1)'],
        ['\tTabbed.pdf', '\rReturned.pdf', 'Plain = fine.pdf', '87%'],
      ],
      header: false,
    };
    expect(toTsv(block).split('\r\n')).toEqual([
      // Confidence is Intern's own percentage, never a formula, and left alone.
      `"'=HYPERLINK(""http://x"",""click"")"\t'+1 Lease.pdf\t'-2 Notes\t@SUM(A1)`,
      `"'\tTabbed.pdf"\t"'\rReturned.pdf"\tPlain = fine.pdf\t87%`,
    ]);
    // The header row of a whole-table copy is Intern's own words.
    expect(toTsv({ ...block, rows: [], header: true })).toBe('Original filename\tProposed filename\tDescription\tConfidence');
  });

  it('writes an HTML table that pastes into separate cells, as text in both spreadsheets', () => {
    const block: CopiedBlock = {
      columns: tableBlock([]).columns,
      rows: [['<b>&"\'', 'first\nsecond', '=HYPERLINK("x")', '87%']],
      header: true,
    };
    const html = toHtml(block);
    const asText = (value: string) => ` style="mso-number-format:'\\@'" data-sheets-value="${value}"`;
    expect(html).toBe(
      '<meta charset="utf-8"><table>'
      + '<thead><tr><th>Original filename</th><th>Proposed filename</th><th>Description</th><th>Confidence</th></tr></thead>'
      + '<tbody><tr>'
      + `<td${asText('{&quot;1&quot;:2,&quot;2&quot;:&quot;&lt;b&gt;&amp;\\&quot;&#39;&quot;}')}>&lt;b&gt;&amp;&quot;&#39;</td>`
      + `<td${asText('{&quot;1&quot;:2,&quot;2&quot;:&quot;first\\nsecond&quot;}')}>first<br style="mso-data-placement:same-cell">second</td>`
      + `<td${asText('{&quot;1&quot;:2,&quot;2&quot;:&quot;=HYPERLINK(\\&quot;x\\&quot;)&quot;}')}>=HYPERLINK(&quot;x&quot;)</td>`
      + '<td>87%</td>'
      + '</tr></tbody></table>',
    );
    // Parsed back, it is the same cells, and Google Sheets' value is the exact string.
    const parsed = new DOMParser().parseFromString(html, 'text/html');
    const cells = [...parsed.querySelectorAll('tbody td')];
    expect(cells.map((cell) => cell.textContent)).toEqual(['<b>&"\'', 'firstsecond', '=HYPERLINK("x")', '87%']);
    expect(cells.slice(0, 3).map((cell) => JSON.parse(cell.getAttribute('data-sheets-value')!))).toEqual([
      { 1: 2, 2: '<b>&"\'' }, { 1: 2, 2: 'first\nsecond' }, { 1: 2, 2: '=HYPERLINK("x")' },
    ]);
    expect(cells[3].hasAttribute('data-sheets-value')).toBe(false);
    expect(toHtml({ ...block, rows: [['東京 🗂️', '', '', '']], header: false })).toContain('東京 🗂️');
  });

  // HTML collapses a run of spaces, and drops them at either end of a line.
  it('keeps every space of a name in the HTML', () => {
    const html = toHtml({ columns: tableBlock([]).columns.slice(0, 1), rows: [['  Lease  2024.pdf '], ['a\n   b']], header: false });
    const spaces = (count: number) => `<span style="mso-spacerun:yes">${'&nbsp;'.repeat(count)}</span>`;
    expect(html).toContain(`>${spaces(2)}Lease${spaces(2)}2024.pdf${spaces(1)}</td>`);
    expect(html).toContain(`>a<br style="mso-data-placement:same-cell">${spaces(3)}b</td>`);
    // A single space between words is left as it is.
    expect(toHtml({ columns: tableBlock([]).columns.slice(0, 1), rows: [['Lease 2024.pdf']], header: false })).toContain('>Lease 2024.pdf</td>');
  });
});
