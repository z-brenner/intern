import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { openDetails, queueCell } from './grid';

/*
  The queue table copies like a spreadsheet, through the real clipboard of a
  real Chromium (the engine of the desktop app's WebView2): what Ctrl+C and
  the Copy button leave there is read back as both formats a spreadsheet
  pastes from.
*/

test.use({ viewport: { width: 1440, height: 900 } });

async function readClipboard(page: Page) {
  return page.evaluate(async () => {
    const [item] = await navigator.clipboard.read();
    const html = item.types.includes('text/html') ? await (await item.getType('text/html')).text() : '';
    const table = new DOMParser().parseFromString(html, 'text/html');
    return {
      text: await (await item.getType('text/plain')).text(),
      cells: [...table.querySelectorAll('tr')].map((row) => [...row.querySelectorAll('td, th')].map((cell) => cell.textContent)),
    };
  });
}

test.beforeEach(async ({ context, page }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.goto('/?reviewBatch=1');
  await expect(page.getByRole('row', { name: /Scan 0101\.pdf/ })).toHaveAttribute('aria-current', 'true');
});

test('a dragged range copies with Ctrl+C into separate cells, without Status', async ({ page }) => {
  const from = await queueCell(page, 'Scan 0101.pdf').boundingBox();
  const to = await queueCell(page, 'Scan 0103.pdf', 'proposed').boundingBox();
  await page.mouse.move(from!.x + 20, from!.y + from!.height / 2);
  await page.mouse.down();
  await page.mouse.move(to!.x + 20, to!.y + to!.height / 2, { steps: 8 });
  await page.mouse.up();

  await expect(page.getByRole('gridcell', { selected: true })).toHaveCount(6);
  await expect(page.getByRole('button', { name: 'Copy selection' })).toBeVisible();
  // Selecting is not opening.
  await expect(page.getByRole('complementary', { name: 'Review item' })).toHaveCount(0);

  await page.keyboard.press('ControlOrMeta+C');
  const copied = await readClipboard(page);
  expect(copied.text).toBe([
    'Scan 0101.pdf\t2024-02-01 Engagement Letter with Northwind Traders.pdf',
    'Scan 0102.pdf\t2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf',
    'Scan 0103.pdf\t2024-03-14 Notice of Termination to Contoso Ltd.pdf',
  ].join('\r\n'));
  expect(copied.cells).toEqual([
    ['Scan 0101.pdf', '2024-02-01 Engagement Letter with Northwind Traders.pdf'],
    ['Scan 0102.pdf', '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf'],
    ['Scan 0103.pdf', '2024-03-14 Notice of Termination to Contoso Ltd.pdf'],
  ]);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText('Copied 3 rows, 2 columns.');
});

test('Shift+click, Ctrl+A and the keyboard select cells; Escape clears', async ({ page }) => {
  await queueCell(page, 'Scan 0102.pdf', 'description').click();
  await queueCell(page, 'Scan 0103.pdf', 'check').click({ modifiers: ['Shift'] });
  await page.keyboard.press('ControlOrMeta+C');
  expect((await readClipboard(page)).text).toBe([
    'Invoice INV-3301 from Fabrikam Inc. for February consulting hours.\tDate could be day- or month-first',
    'Notice terminating the services agreement with Contoso Ltd.\tAll checks passed',
  ].join('\r\n'));

  await page.keyboard.press('Shift+ArrowDown');
  await expect(page.getByRole('gridcell', { selected: true })).toHaveCount(6);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('gridcell', { selected: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Copy table' })).toBeVisible();

  await page.keyboard.press('ControlOrMeta+A');
  await expect(page.getByRole('gridcell', { selected: true })).toHaveCount(28);
  await page.keyboard.press('ControlOrMeta+C');
  const all = await readClipboard(page);
  expect(all.text.split('\r\n')).toHaveLength(7);
  expect(all.text.split('\r\n')[6]).toBe('Scan 0107.pdf\t\t\t');
});

test('the Copy button copies the whole table with headings when nothing is selected', async ({ page }) => {
  await page.getByRole('button', { name: 'Copy table' }).click();
  const copied = await readClipboard(page);
  const lines = copied.text.split('\r\n');
  expect(lines[0]).toBe('Original filename\tProposed filename\tDescription\tCheck');
  expect(lines[1]).toBe('Scan 0101.pdf\t2024-02-01 Engagement Letter with Northwind Traders.pdf\tEngagement letter between Northwind Traders and the firm for 2024 advisory work.\tModel unsure of its reading');
  expect(lines).toHaveLength(8);
  expect(copied.cells[0]).toEqual(['Original filename', 'Proposed filename', 'Description', 'Check']);
  expect(copied.cells).toHaveLength(8);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText('Copied the table, 7 rows.');
});

test('the Copy button copies a selection, and leaves the Details panel as it was', async ({ page }) => {
  const panel = await openDetails(page);
  await queueCell(page, 'Scan 0104.pdf', 'check').click();
  await expect(panel).toContainText('Scan 0104.pdf');
  await page.getByRole('button', { name: 'Copy selection' }).click();
  expect((await readClipboard(page)).text).toBe('Model unsure of its reading');
  await expect(panel).toBeVisible();
});
