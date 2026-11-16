import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { SETTLE_MS } from '../../src/lib/aimedPress';
import { openDetails, queueCell, untilSettled } from './grid';

/*
  Two things changed the queue with nothing on screen to say so. A Settings
  save that changes the review level judges the documents that wait again;
  and a document whose name a person typed and did not file can be renamed
  by something other than that person. The window now says each, where it
  says what an action did.
*/

const LEASE = 'Lease Agreement - 123 Main St.pdf';
const SHOTS = process.env.R23_OUT;

/** Settings, at "Naming and filing", where the level is. */
async function openSettings(page: Page) {
  await page.getByRole('button', { name: 'Settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.locator('summary', { hasText: 'Naming and filing' }).click();
  const level = dialog.getByRole('group', { name: 'Which documents are ready' });
  return {
    dialog,
    strict: level.getByRole('radio', { name: /^Strict/ }),
    balanced: level.getByRole('radio', { name: /^Balanced/ }),
    save: dialog.getByRole('button', { name: 'Save settings' }),
  };
}
/** Every row of the queue as it reads, and the toolbar's counts. */
const queueAsShown = (page: Page) => page.evaluate(() => ({
  rows: [...document.querySelectorAll('.queue-grid tbody tr')].map((row) => row.textContent),
  toolbar: document.querySelector('.queue-toolbar')?.textContent,
}));

test('a save that changes the level says what it did to the documents that wait: to Balanced, and back to Strict', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  const result = page.getByRole('group', { name: 'Action result' });

  let settings = await openSettings(page);
  await settings.balanced.check();
  await settings.save.click();
  await expect(settings.dialog).toBeHidden();
  const toBalanced = 'The review level is now Balanced. 1 document that needed review became ready. Rename all ready renames it.';
  await expect(result).toHaveText(toBalanced);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText(toBalanced);
  // Where the outcome of an action is said, with nothing to press but its dismissal.
  await expect(result.getByRole('button')).toHaveCount(1);
  await expect(result.getByRole('button', { name: 'Dismiss' })).toBeVisible();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Ready with a note');
  await expect(page.getByRole('button', { name: 'Completed, 1' })).toBeVisible();
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/level-to-balanced.png` });

  settings = await openSettings(page);
  await settings.strict.check();
  await settings.save.click();
  await expect(settings.dialog).toBeHidden();
  const toStrict = 'The review level is now Strict. 1 ready document with a note needs review again.';
  await expect(result).toHaveText(toStrict);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText(toStrict);
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/level-to-strict.png` });
});

test('at Strict with automatic renaming off, a save that leaves the level as it was says nothing and changes nothing on screen', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/');
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  const before = await queueAsShown(page);

  const settings = await openSettings(page);
  await expect(settings.strict).toBeChecked();
  await expect(settings.dialog.getByRole('checkbox', { name: 'Automatically rename ready documents' })).not.toBeChecked();
  await settings.dialog.getByLabel("This machine's name").fill('Reception');
  await settings.save.click();
  await expect(settings.dialog).toBeHidden();
  await page.waitForTimeout(500);

  await expect(page.locator('.toasts')).toHaveCount(0);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText('');
  expect(await queueAsShown(page)).toEqual(before);
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/strict-unchanged.png` });
});

/*
  The demo does not rename anything by itself: automatic renaming is not
  acted out there. `?queue=changes` is the demo's queue with
  `window.internQueue` to change it as the backend does, and it can file a
  document as if from somewhere else. That is what is played here: the
  lease, with a name typed for it, is filed under the name Intern proposed
  with nobody in this window deciding it. The press on Undo is a real one,
  at a place, made the moment the message comes.
*/
type QueueByItself = { change(id: string, change: Record<string, unknown>): void };
const filedByItself = (page: Page) => page.evaluate(() => {
  const queue = (window as unknown as { internQueue: QueueByItself }).internQueue;
  const name = '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf';
  queue.change('lease', { status: 'completed', filedName: name, filedPath: `C:\\Intern\\Output\\${name}`, keptOriginal: false, undoable: true });
  (window as unknown as { changedAt: number }).changedAt = performance.now();
});
const sinceItChanged = (page: Page) => page.evaluate(() => performance.now() - (window as unknown as { changedAt: number }).changedAt);
const middle = (box: { x: number; y: number; width: number; height: number } | null) => ({ x: box!.x + box!.width / 2, y: box!.y + box!.height / 2 });

test('a document with a name typed for it, renamed with nobody here deciding it, is said, and Undo is taken only once it has been there to be seen', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  const renamed = page.getByRole('group', { name: 'Renamed without you' });
  const undo = renamed.getByRole('button', { name: `Undo the rename of ${LEASE}` });
  const said = `${LEASE} was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details.`;
  for (let attempt = 1; ; attempt += 1) {
    if (attempt > 1) await page.evaluate(() => window.localStorage.clear());
    await page.goto('/?queue=changes');
    const panel = await openDetails(page, LEASE);
    await panel.getByLabel('Filename').fill('2023-09-15 Lease of 123 Main St');
    await untilSettled(page);
    await filedByItself(page);
    await expect(renamed).toHaveText(said);
    // The press made the moment it came: refused, so nothing happens.
    const at = middle(await undo.boundingBox());
    await page.mouse.click(at.x, at.y);
    if (await sinceItChanged(page) < SETTLE_MS - 50) break;
    if (attempt === 3) throw new Error('Three presses in a row came later than the settle time after the change: this machine is too slow for the journey to say anything.');
  }
  await page.waitForTimeout(300);
  await expect(renamed).toHaveText(said);
  await expect(page.getByRole('status', { name: 'Action status' })).toHaveText(said);
  await expect(page.getByRole('group', { name: 'Action result' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Completed, 2' })).toBeVisible();
  // What was typed is where it is kept after such a rename: beside the document, read-only.
  const panel = page.getByRole('complementary', { name: 'Review item' });
  await expect(panel.getByLabel('What you had typed')).toHaveValue('2023-09-15 Lease of 123 Main St\n\nCommercial lease agreement between landlord and tenant for 123 Main St.');
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/renamed-without-you.png` });

  // It does not leave by itself: still there well past the ten seconds an outcome stays.
  await page.waitForTimeout(10_500);
  await expect(renamed).toHaveText(said);

  // Pressed again once it has been there to be seen: the rename is undone, the same way as from a rename's outcome.
  await undo.click();
  await expect(page.getByRole('group', { name: 'Action result' })).toHaveText('Undid 1 rename.');
  await expect(renamed).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Completed, 1' })).toBeVisible();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  const again = await openDetails(page, LEASE);
  await expect(again.getByLabel('What you had typed')).toHaveValue('2023-09-15 Lease of 123 Main St\n\nCommercial lease agreement between landlord and tenant for 123 Main St.');
});
