import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { SETTLE_MS } from '../../src/lib/aimedPress';
import { openDetails, untilSettled } from './grid';

/*
  A press of the pointer acts only on a control it can have been aimed at
  (src/lib/aimedPress.ts). The window changes by itself - a document leaves
  the queue, or is read again - and a press already on its way lands a
  moment later on whatever has come under the pointer. A quarter of a second
  after the open document left the queue, a press on its way to the panel's
  Close review landed on Rename all ready and renamed three documents; after
  a re-reading, one on its way to the name landed on "Discard what I had",
  and what had been typed was gone. Such a press now does nothing, and the
  same press made once the control has been there for the settle time is an
  ordinary press.

  `?queue=changes` is the demo's queue with `window.internQueue` to change it
  as the backend does. The presses here are real ones, at a place, not at a
  control: a press on its way does not look where it lands.
*/

const LEASE = 'Lease Agreement - 123 Main St.pdf';
const WIDE = { width: 1400, height: 900 };
const NARROW = { width: 800, height: 600 };

type QueueByItself = { change(id: string, change: Record<string, unknown>): void; leave(id: string): void };
/** Changes the queue with nobody asking, and notes when by the page's own clock. */
const byItself = (page: Page, how: 'left the queue' | 'read again') => page.evaluate((how) => {
  const queue = (window as unknown as { internQueue: QueueByItself }).internQueue;
  if (how === 'left the queue') queue.leave('lease');
  else queue.change('lease', { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined });
  (window as unknown as { changedAt: number }).changedAt = performance.now();
}, how);
/** Milliseconds since the queue changed by itself, by the page's own clock. */
const sinceItChanged = (page: Page) => page.evaluate(() => performance.now() - (window as unknown as { changedAt: number }).changedAt);
/** What is on top at a place, by its accessible name. */
const buttonAt = (page: Page, at: { x: number; y: number }) => page.evaluate(({ x, y }) => {
  const button = document.elementFromPoint(x, y)?.closest('button');
  return button ? button.getAttribute('aria-label') ?? button.textContent : null;
}, at);
/** How the border of the button at a place is drawn: the browser draws one with no border of its own pressed in ("inset") while a mouse button is held on it. */
const borderAt = (page: Page, at: { x: number; y: number }) => page.evaluate(({ x, y }) => {
  const button = document.elementFromPoint(x, y)?.closest('button');
  return button ? getComputedStyle(button).borderTopStyle : null;
}, at);
const middle = (box: { x: number; y: number; width: number; height: number } | null) => ({ x: box!.x + box!.width / 2, y: box!.y + box!.height / 2 });
/** Every row of the queue as it reads: its name, its status and the name proposed for it. */
const rows = (page: Page) => page.locator('.queue-grid tbody tr').evaluateAll((all) => all.map((row) => `${row.querySelector('[data-column="original"]')?.textContent} | ${row.querySelector('[data-column="status"]')?.textContent} | ${row.querySelector('[data-column="proposed"]')?.textContent}`));

/**
 * A press that must land within the settle time of a change says nothing
 * when the machine running the journey stalls for longer than that between
 * the change and the press. Such a run is made again, three times at most.
 */
async function withinTheSettleTime(page: Page, journey: () => Promise<void>) {
  for (let attempt = 1; ; attempt += 1) {
    await journey();
    if (await sinceItChanged(page) < SETTLE_MS - 50) return;
    if (attempt === 3) throw new Error('Three presses in a row came later than the settle time after the change: this machine is too slow for the journey to say anything.');
  }
}

for (const size of [WIDE, NARROW]) {
  test(`a press on its way to the panel does nothing on Rename all ready, which came there as the document left the queue, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    let aimedAt = { x: 0, y: 0 };
    let before: string[] = [];
    await withinTheSettleTime(page, async () => {
      await page.goto('/?queue=changes');
      const panel = await openDetails(page, LEASE);
      // The focus on the heading, not in a field: nothing typed is kept above the queue to move the toolbar.
      await panel.getByRole('heading', { name: 'Review item' }).click();
      aimedAt = middle(await panel.getByRole('button', { name: size === WIDE ? 'Close review' : 'Next undecided' }).boundingBox());
      before = (await rows(page)).filter((row) => !row.startsWith(LEASE));
      await untilSettled(page);
      await byItself(page, 'left the queue');
      await expect(page.locator('#review-panel')).toHaveCount(0);
      await page.mouse.click(aimedAt.x, aimedAt.y);
    });
    // It is Rename all ready that the press landed on, or the journey shows nothing.
    expect(await buttonAt(page, aimedAt)).toBe('Rename all ready');
    await page.waitForTimeout(300);
    await expect(page.locator('.toasts')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Rename all ready' })).toHaveText('Rename all ready 3');
    await expect(page.getByRole('button', { name: 'Rename all ready' })).not.toBeFocused();
    expect(await rows(page)).toEqual(before);

    // Once it has been there for the settle time, the same press is taken.
    await untilSettled(page);
    await page.mouse.click(aimedAt.x, aimedAt.y);
    await expect(page.locator('.toasts')).toContainText('Renamed 3 documents.');
  });
}

test('a press on its way to the name does nothing on "Discard what I had", which came there as the document was read again', async ({ page }) => {
  await page.setViewportSize(WIDE);
  let aimedAt = { x: 0, y: 0 };
  let heldDown: string | null = '';
  const kept = page.getByRole('textbox', { name: 'What you had typed' });
  await withinTheSettleTime(page, async () => {
    await page.goto('/?queue=changes');
    const panel = await openDetails(page, LEASE);
    const name = panel.getByRole('textbox', { name: 'Filename' });
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2 signed');
    aimedAt = middle(await name.boundingBox());
    await untilSettled(page);
    await byItself(page, 'read again');
    await expect(kept).toBeFocused();
    await page.mouse.move(aimedAt.x, aimedAt.y);
    await page.mouse.down();
    heldDown = await borderAt(page, aimedAt);
    await page.mouse.up();
  });
  expect(await buttonAt(page, aimedAt)).toBe('Discard what I had');
  await page.waitForTimeout(300);
  await expect(kept).toHaveValue(/ v2 signed/);
  // Not the focus either: the caret is still in what was kept.
  await expect(kept).toBeFocused();
  // Nor was anything shown: with the mouse button held down the control looked as it does now.
  expect(heldDown).toBe(await borderAt(page, aimedAt));

  await untilSettled(page);
  await page.mouse.click(aimedAt.x, aimedAt.y);
  await expect(kept).toHaveCount(0);
});

/*
  What a person does at speed is not slowed. An item of the More menu is
  taken as soon as the pointer has gone to it: the person asked for the menu.
  And Approve & rename is the same button for every document, where the
  person left it: one document after another is approved with the pointer at
  rest on it, each press made as soon as the panel names the next.
*/
test('More and then one of its items, at once, is taken', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/?queue=changes');
  const panel = await openDetails(page, LEASE);
  await panel.getByRole('button', { name: 'More review actions' }).click();
  await panel.getByRole('button', { name: 'Analyze again' }).click();
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText('Sent back to be analyzed again.');
});

test('one document after another is approved with the pointer at rest on Approve & rename', async ({ page }) => {
  await page.setViewportSize(NARROW);
  await page.goto('/?queue=changes');
  const panel = await openDetails(page, LEASE);
  const shown = panel.locator('.selected-file');
  const at = middle(await panel.getByRole('button', { name: 'Approve & rename' }).boundingBox());
  await page.mouse.move(at.x, at.y);
  for (const next of ['Employment Agreement - John Smith.pdf', 'NDA - Acme Corp.docx', 'Service Agreement - BlueSky LLC.pdf']) {
    await page.mouse.down();
    await page.mouse.up();
    await expect(shown).toHaveText(next);
  }
  await expect(page.getByRole('button', { name: /^Completed, / })).toHaveText(/4$/);
});
