import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { openDetails, queueCell, untilSettled } from './grid';

/*
  A name or a description typed in review and not filed was kept by the
  window and nothing else: closing Intern, a crash, or the restart an update
  makes threw it away with no notice. It is stored as it is typed
  (src/features/review/typedStore.ts) and put back when Intern starts: in
  the fields when the document has the reading it was typed over, set aside
  read-only beside another, dropped with a document no longer in the queue.

  The demo starts from the same queue on every load, so a reload here is
  Intern closed and opened: the window and everything it kept are gone, its
  storage stays, and the queue is as the backend would list it at the start.
  Each journey has a browser context, and so a storage, of its own.

  A reload stands in for a close and says nothing about what reaches the
  disk: the browser goes on running, and its storage with it. What a real
  close, a kill or a power cut leave on disk is not tried by these journeys.
*/

const LEASE = 'Lease Agreement - 123 Main St.pdf';
const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
const PROPOSED = '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc';
const PROPOSED_DESCRIPTION = 'Commercial lease agreement between landlord and tenant for 123 Main St.';
const TYPED = `${PROPOSED} v2 signed`;
const DESCRIBED = `${PROPOSED_DESCRIPTION} Countersigned copy received 3 May`;
const REREAD = '2023-09-15 Lease between ABC Properties and TenantCo';
const WIDE = { width: 1400, height: 900 };
const NARROW = { width: 800, height: 600 };
const STORED = 'intern.typedInReview.v1';
const CLOSED = 'intern.typedInReview.closed';

/** Types on at the end of the name and of the description, as a person correcting them does. */
async function typeInTheLease(page: Page) {
  const panel = await openDetails(page, LEASE);
  await panel.getByRole('textbox', { name: 'Filename' }).click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' v2 signed');
  await panel.getByRole('textbox', { name: 'Description' }).click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' Countersigned copy received 3 May');
  return panel;
}
const stored = (page: Page) => page.evaluate((key) => window.localStorage.getItem(key), STORED);
/** What has the focus: a field's name, or the tag of anything else. */
const focused = (page: Page) => page.evaluate(() => {
  const element = document.activeElement;
  return element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement ? element.getAttribute('aria-label') : element?.tagName;
});
/** Every row of the queue as it reads: its name, its status and the name proposed for it. */
const rows = (page: Page) => page.locator('.queue-grid tbody tr').evaluateAll((all) => all.map((row) => `${row.querySelector('[data-column="original"]')?.textContent} | ${row.querySelector('[data-column="status"]')?.textContent} | ${row.querySelector('[data-column="proposed"]')?.textContent}`));
type QueueByItself = { change(id: string, change: Record<string, unknown>): void; leave(id: string): void };
const queueByItself = (page: Page, how: 'read again' | 'new reading') => page.evaluate((how) => {
  const queue = (window as unknown as { internQueue: QueueByItself }).internQueue;
  if (how === 'read again') queue.change('lease', { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined });
  else queue.change('lease', { status: 'review', stage: undefined, proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', proposalRevision: '1' });
}, how);

for (const size of [WIDE, NARROW]) {
  test(`a name and a description typed and not filed are in their fields after Intern is closed and opened, and nothing has the focus for them, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/');
    await typeInTheLease(page);
    const before = await rows(page);
    // Closed straight after the last letter: nothing waits to be written.
    await page.reload();

    await expect(queueCell(page, LEASE).locator('xpath=ancestor::tr')).toHaveAttribute('aria-current', 'true');
    // No field has the focus, and what is typed now goes into none and files nothing.
    expect(await focused(page)).toBe('BODY');
    await page.keyboard.type(' more');
    await page.waitForTimeout(200);
    expect(await focused(page)).toBe('BODY');
    await expect(page.locator('.toasts')).toHaveCount(0);
    expect(await rows(page)).toEqual(before);
    // Wide, the panel was left open and is open on the Lease as it starts; narrow, a drawer opens only when asked.
    if (size === NARROW) await expect(page.locator('#review-panel')).toHaveCount(0);
    const panel = size === WIDE ? page.getByRole('complementary', { name: 'Review item' }) : await openDetails(page, LEASE);
    await expect(panel).toContainText(LEASE);
    await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(TYPED);
    await expect(panel.getByRole('textbox', { name: 'Description' })).toHaveValue(DESCRIBED);
    await expect(page.getByRole('textbox', { name: 'What you had typed' })).toHaveCount(0);

    // Back to Intern's proposal drops it, here as in the window it was typed in: closed and opened again, it is the proposal.
    await untilSettled(page);
    await panel.getByRole('button', { name: 'Back to Intern\'s proposal' }).click();
    await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
    expect(await stored(page)).toBeNull();
    await page.reload();
    const again = await openDetails(page, LEASE);
    await expect(again.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
    await expect(again.getByRole('textbox', { name: 'Description' })).toHaveValue(PROPOSED_DESCRIPTION);
  });
}

/*
  Only an ordinary close is known to leave the last of what was typed. The
  window says so in storage as it goes (on `pagehide`); one that ended with
  no word to the page - killed, the power cut - leaves no such mark, and
  what the next window finds it sets aside, read-only, where no key and no
  press files it. A reload is an ordinary close, so here the mark it leaves
  is taken out again before the next window starts - once, by a script that
  runs ahead of the page's own: storage is then as a kill straight after
  the last letter would leave it, had all of it reached the disk.
*/
test('what was typed is set aside, not in its fields, after a window that went without closing in the ordinary way, and Ctrl+Enter then files the proposal', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/');
  await typeInTheLease(page);
  expect(await stored(page)).toContain(TYPED);
  await page.addInitScript((key) => {
    if (window.sessionStorage.getItem('markTakenOut')) return;
    window.sessionStorage.setItem('markTakenOut', String(window.localStorage.getItem(key)));
    window.localStorage.removeItem(key);
  }, CLOSED);
  await page.reload();
  // The reload did leave the mark: it is its absence that is tried here.
  expect(await page.evaluate(() => window.sessionStorage.getItem('markTakenOut'))).toBe('1');

  // The Lease under the reading it was typed over, and its fields are Intern's proposal all the same.
  const panel = page.getByRole('complementary', { name: 'Review item' });
  await expect(panel).toContainText(LEASE);
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
  await expect(panel.getByRole('textbox', { name: 'Description' })).toHaveValue(PROPOSED_DESCRIPTION);
  const kept = page.getByRole('textbox', { name: 'What you had typed' });
  await expect(kept).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
  await expect(kept).toHaveAttribute('readonly', '');
  expect(await focused(page)).toBe('BODY');
  expect(await page.evaluate((key) => window.localStorage.getItem(key), CLOSED)).toBeNull();

  // Closed in the ordinary way now, it stays set aside: the mark is there as the window goes, and gone once the next has started.
  await page.evaluate((key) => window.addEventListener('pagehide', () => { window.sessionStorage.setItem('markAsItWent', String(window.localStorage.getItem(key))); }), CLOSED);
  await page.reload();
  await expect(page.getByRole('complementary', { name: 'Review item' })).toContainText(LEASE);
  expect(await page.evaluate(() => window.sessionStorage.getItem('markAsItWent'))).toBe('1');
  expect(await page.evaluate((key) => window.localStorage.getItem(key), CLOSED)).toBeNull();
  await expect(kept).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);

  // Ctrl+Enter with nothing focused files what the panel shows: the proposal, not the name that was put back.
  expect(await focused(page)).toBe('BODY');
  await page.keyboard.press('Control+Enter');
  await expect(page.locator('.toasts')).toContainText('Renamed 1 document.');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: /^Completed/ }).click();
  await expect(queueCell(page, LEASE, 'proposed')).toContainText(`${PROPOSED}.pdf`);
  await expect(queueCell(page, LEASE, 'proposed')).not.toContainText('v2 signed');
});

test('what was typed for a document that then changed by itself is still set aside beside it after Intern is closed and opened', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/?queue=changes');
  const panel = await typeInTheLease(page);
  const kept = page.getByRole('textbox', { name: 'What you had typed' });
  // Read again by itself, to another name: the fields show the new reading, and what was typed is beside them.
  await queueByItself(page, 'read again');
  await expect(kept).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
  await queueByItself(page, 'new reading');
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(REREAD);
  await page.reload();

  // The queue starts as the demo's does, the Lease under its first reading. What was set aside stays set aside:
  // it is not put back into the fields of a reading it was set aside from.
  const again = page.getByRole('complementary', { name: 'Review item' });
  await expect(again).toContainText(LEASE);
  await expect(again.getByRole('group', { name: 'Kept from before this document changed' })).toBeVisible();
  await expect(kept).toHaveValue(`${TYPED}\n\n${DESCRIBED}`);
  await expect(kept).toHaveAttribute('readonly', '');
  await expect(again.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
  await expect(again.getByRole('textbox', { name: 'Description' })).toHaveValue(PROPOSED_DESCRIPTION);
  expect(await focused(page)).toBe('BODY');

  // Only the person drops it: discarded, it is gone from the next start too.
  await untilSettled(page);
  await again.getByRole('button', { name: 'Discard what I had' }).click();
  await expect(kept).toHaveCount(0);
  expect(await stored(page)).toBeNull();
  await page.reload();
  await expect(page.getByRole('complementary', { name: 'Review item' })).toContainText(LEASE);
  await expect(kept).toHaveCount(0);
});

test('what was typed over one reading is set aside, not put in the fields, when Intern starts with the document under another', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/?queue=changes');
  // The Lease is read again before anything is typed, and the name is typed over that reading.
  const panel = await openDetails(page, LEASE);
  await queueByItself(page, 'new reading');
  const name = panel.getByRole('textbox', { name: 'Filename' });
  await expect(name).toHaveValue(REREAD);
  await name.click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' (signed copy)');
  await expect(page.getByRole('textbox', { name: 'What you had typed' })).toHaveCount(0);
  await page.reload();

  // The demo starts with the Lease under its first reading: another than the one typed over.
  const again = page.getByRole('complementary', { name: 'Review item' });
  await expect(again).toContainText(LEASE);
  await expect(again.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
  await expect(again.getByRole('textbox', { name: 'Description' })).toHaveValue(PROPOSED_DESCRIPTION);
  const kept = page.getByRole('textbox', { name: 'What you had typed' });
  await expect(kept).toHaveValue(`${REREAD} (signed copy)\n\nA lease of 123 Main St.`);
  expect(await focused(page)).toBe('BODY');
  // Approving files the reading on screen, as it stands.
  await untilSettled(page);
  await again.getByRole('button', { name: /Approve & rename/ }).click();
  await expect(page.locator('.toasts')).toContainText('Renamed 1 document.');
  await page.getByRole('button', { name: /^Completed, \d+$/ }).click();
  await expect(queueCell(page, LEASE, 'proposed')).toHaveText(`${PROPOSED}.pdf`);
});

test('a name typed and then filed is not there after Intern is closed and opened', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/');
  const panel = await typeInTheLease(page);
  expect(await stored(page)).toContain(' v2 signed');
  await panel.getByRole('button', { name: /Approve & rename/ }).click();
  await expect(page.locator('.toasts')).toContainText('Renamed 1 document.');
  // Filed under what was typed, and out of storage with the filing, not at the next start.
  expect(await stored(page)).toBeNull();
  await page.reload();

  // The demo's queue starts with the Lease in review again, as it first was: nothing typed is waiting for it.
  const again = await openDetails(page, LEASE);
  await expect(again.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
  await expect(again.getByRole('textbox', { name: 'Description' })).toHaveValue(PROPOSED_DESCRIPTION);
  await expect(page.getByRole('textbox', { name: 'What you had typed' })).toHaveCount(0);
  await expect(again.getByRole('button', { name: 'Back to Intern\'s proposal' })).toHaveCount(0);
});

test('Rename all ready leaves out a ready document whose name was typed before Intern was closed, and says so', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/');
  const panel = await openDetails(page, EMPLOYMENT);
  await panel.getByRole('textbox', { name: 'Filename' }).click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' (corrected)');
  const all = page.getByRole('button', { name: 'Rename all ready' });
  // As it is in the window it was typed in.
  await expect(all).toHaveText('Rename all ready 2');
  const title = await all.getAttribute('title');
  await page.reload();

  await expect(all).toHaveText('Rename all ready 2');
  expect(await all.getAttribute('title')).toBe(title);
  expect(title).toBe('Renames the 2 ready documents as Intern proposed. 1 with a name or a description you typed is left for you to file.');
  await expect(page.getByRole('button', { name: '3 ready' })).toBeVisible();
  // The button has only just come with the window (untilSettled).
  await untilSettled(page);
  await all.click();
  await expect(page.locator('.toasts')).toContainText('Renamed 2 documents. 1 was left for you: it has a name or a description you typed and did not file.');
  await expect(queueCell(page, EMPLOYMENT, 'status')).toContainText('Ready');
  await expect(queueCell(page, EMPLOYMENT, 'proposed')).toHaveText('2024-04-12 Employment Agreement with John Smith.pdf');
  // Its name is still the one typed, and filing it files that.
  const again = await openDetails(page, EMPLOYMENT);
  await expect(again.getByRole('textbox', { name: 'Filename' })).toHaveValue('2024-04-12 Employment Agreement with John Smith (corrected)');
});

test('what was typed is not shown in a queue its document is not in, and is dropped there', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/');
  await typeInTheLease(page);
  expect(await stored(page)).toContain(' v2 signed');
  // Another queue altogether: a morning's scans, none of them the Lease.
  await page.goto('/?reviewBatch=1');
  await expect(page.getByRole('grid', { name: 'Queue' })).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'What you had typed' })).toHaveCount(0);
  await expect(page.getByText(' v2 signed')).toHaveCount(0);
  expect(await stored(page)).toBeNull();
  // And it does not come back with the Lease.
  await page.goto('/');
  const panel = await openDetails(page, LEASE);
  await expect(panel.getByRole('textbox', { name: 'Filename' })).toHaveValue(PROPOSED);
});
