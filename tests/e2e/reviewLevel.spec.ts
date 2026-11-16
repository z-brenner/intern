import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';
import { openDetails, queueCell } from './grid';

/*
  Which documents are ready is a choice in Settings. Strict is what ready
  always meant: every check passed. Balanced also makes ready a document
  whose only doubts are about its type or its description, or that the model
  was unsure of. Such a document is shown as ready with a note: what Intern
  noted is on its row and in its details, and nothing about it reads as a
  review. The demo's lease is one: at Strict it waits, in its own words; at
  Balanced it is ready with a note; and back at Strict it waits again.
*/

const LEASE = 'Lease Agreement - 123 Main St.pdf';

/** Settings, at "Naming and filing", where the choice is. */
async function levelInSettings(page: Page) {
  await page.getByRole('button', { name: 'Settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.locator('summary', { hasText: 'Naming and filing' }).click();
  const level = dialog.getByRole('group', { name: 'Which documents are ready' });
  await expect(level).toBeVisible();
  return { dialog, level, strict: level.getByRole('radio', { name: /^Strict/ }), balanced: level.getByRole('radio', { name: /^Balanced/ }) };
}

test('choosing Balanced in Settings makes the lease ready with a note, the choice is kept, and Strict puts it back', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  const statuses = () => page.locator('.queue-grid [data-column="status"]').allTextContents();
  const checks = () => page.locator('.queue-grid [data-column="check"]').allTextContents();
  // Strict, as the demo has always opened: the lease waits, in its own sentence.
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  await expect(page.getByRole('button', { name: '1 needs review' })).toBeVisible();
  await expect(page.getByRole('button', { name: '3 ready', exact: true })).toBeVisible();
  const strictStatuses = await statuses();
  const strictChecks = await checks();

  let settings = await levelInSettings(page);
  await expect(settings.strict).toBeChecked();
  await expect(settings.balanced).not.toBeChecked();
  // It says what Balanced costs before anyone chooses it, and that it decides Rename all ready too.
  await expect(settings.level).toContainText('more of those names will be wrong');
  await expect(settings.level).toContainText('Ready documents are the ones Rename all ready renames.');
  await expect(settings.dialog.getByRole('checkbox', { name: 'Automatically rename ready documents' })).not.toBeChecked();
  // From the keyboard, as any pair of radio buttons: the arrow key moves to Balanced and chooses it.
  await settings.strict.focus();
  await page.keyboard.press('ArrowDown');
  await expect(settings.balanced).toBeFocused();
  await expect(settings.balanced).toBeChecked();
  await expect(settings.strict).not.toBeChecked();
  // Nothing changes behind Settings until it is saved.
  expect(await statuses()).toEqual(strictStatuses);
  await settings.dialog.getByRole('button', { name: 'Save settings' }).click();
  await expect(settings.dialog).toBeHidden();

  // Ready with a note: on its row, in the counts, and in its details.
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Ready with a note');
  await expect(queueCell(page, LEASE)).toHaveAccessibleDescription('Ready with a note');
  await expect(queueCell(page, LEASE, 'check')).toHaveText('Note: Type uncertain +1 more');
  await expect(page.getByRole('button', { name: /needs? review/ })).toHaveCount(0);
  await expect(page.getByRole('button', { name: '4 ready, 1 with a note' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Rename all ready' })).toHaveText('Rename all ready 4');
  // Every other row is as it was.
  expect((await statuses()).filter((_, index) => index !== 1)).toEqual(strictStatuses.filter((_, index) => index !== 1));
  // No part of its name is marked to check: that is for a review.
  await expect(queueCell(page, LEASE, 'proposed').locator('mark')).toHaveCount(0);

  const panel = await openDetails(page, LEASE);
  const note = panel.getByRole('region', { name: "Intern's note" });
  await expect(note.getByRole('heading', { name: 'Ready with a note' })).toBeVisible();
  await expect(note).toContainText('This document is ready: nothing holds it. Intern noted this about its reading.');
  await expect(note).toContainText('The document does not confirm the type in this name as its own.');
  await expect(note).toContainText('The model was unsure of its reading. If you look at this name, start with the date and the parties.');
  await expect(panel.getByRole('heading', { name: 'What to check' })).toHaveCount(0);
  await expect(panel.getByRole('note', { name: 'Part of the name to check' })).toHaveCount(0);
  // What a ready document is offered, by its ready name.
  await expect(panel.getByRole('button', { name: 'Apply rename' })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Approve & rename' })).toHaveCount(0);
  await expect(panel.getByRole('button', { name: 'Keep original' })).toBeVisible();
  // The note is in sight without scrolling for it, above the name.
  await expect(note).toBeInViewport({ ratio: 1 });
  // Nothing was renamed by any of this. (What the save did is said at the foot of the queue: saidAfterwards.spec.)
  await expect(page.getByRole('button', { name: 'Completed, 1' })).toBeVisible();
  await expect(page.locator('.toasts', { hasText: /\brenamed\b|Renamed/ })).toHaveCount(0);

  // The choice is kept.
  settings = await levelInSettings(page);
  await expect(settings.balanced).toBeChecked();
  await expect(settings.strict).not.toBeChecked();
  // And back to Strict, with the pointer this time.
  await settings.strict.check();
  await settings.dialog.getByRole('button', { name: 'Save settings' }).click();
  await expect(settings.dialog).toBeHidden();

  // The queue is the queue it was: the lease waits again, in its own sentence.
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  await expect(page.getByRole('button', { name: '1 needs review' })).toBeVisible();
  await expect(page.getByRole('button', { name: '3 ready', exact: true })).toBeVisible();
  expect(await statuses()).toEqual(strictStatuses);
  expect(await checks()).toEqual(strictChecks);
  await expect(panel.getByRole('region', { name: "Intern's note" })).toHaveCount(0);
  await expect(panel.getByRole('heading', { name: 'What to check' })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Approve & rename' })).toBeVisible();

  settings = await levelInSettings(page);
  await expect(settings.strict).toBeChecked();
  // Closed without saving, a choice is thrown away.
  await settings.balanced.check();
  await page.keyboard.press('Escape');
  await expect(settings.dialog).toBeHidden();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  settings = await levelInSettings(page);
  await expect(settings.strict).toBeChecked();
});

/*
  A document Intern renames by itself at Balanced is renamed with nobody
  looking, so one renamed while it carried a note is easy to find again: it
  is marked where filed documents are listed, and can be listed alone.
  `?reviewLevel=balanced` is the demo's queue at Balanced with one such scan.
*/
test('a document renamed with a note is marked in Completed and in the history, and can be listed alone', async ({ page }) => {
  await page.goto('/?reviewLevel=balanced');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Ready with a note');

  await page.getByRole('button', { name: 'Completed, 2' }).click();
  await expect(page.getByRole('row')).toHaveCount(3);
  await expect(queueCell(page, 'Scan 0098.pdf', 'status')).toHaveText('Completed with a note');
  await expect(queueCell(page, 'Scan 0098.pdf', 'check')).toHaveText('Note: Type supplied by Intern +1 more');
  await expect(queueCell(page, 'Completed lease.pdf', 'status')).toHaveText('Completed');
  // The longer status fits its column on one line: here, beside the open
  // panel (below), and at the window's narrowest, where the column is too.
  const fits = () => page.locator('.queue-grid [data-column="status"]').evaluateAll((cells) => cells.every((cell) => cell.scrollWidth <= cell.clientWidth));
  expect(await fits()).toBe(true);
  const size = page.viewportSize()!;
  await page.setViewportSize({ width: 800, height: 600 });
  await expect(queueCell(page, 'Scan 0098.pdf', 'status')).toHaveText('Completed with a note');
  expect(await fits()).toBe(true);
  await page.setViewportSize(size);

  const filter = page.getByRole('button', { name: '1 with a note' });
  await expect(filter).toHaveAttribute('aria-pressed', 'false');
  await filter.click();
  await expect(filter).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('row')).toHaveCount(2);
  await expect(page.getByText('1 of 2 items')).toBeVisible();
  await expect(queueCell(page, 'Scan 0098.pdf')).toBeVisible();

  const panel = await openDetails(page, 'Scan 0098.pdf');
  expect(await fits()).toBe(true);
  const note = panel.getByRole('region', { name: "Intern's note" });
  await expect(note.getByRole('heading', { name: 'Renamed with a note' })).toBeVisible();
  await expect(note).toContainText('Intern had this noted about its reading when the document was renamed.');
  await expect(note).toContainText('No specific document type was identified.');
  await expect(panel.getByRole('button', { name: 'Undo', exact: true })).toBeVisible();

  // In the history, on the row of that rename.
  await page.getByRole('button', { name: 'History', exact: true }).click();
  const history = page.getByRole('dialog', { name: 'Rename history' });
  const mark = history.getByRole('note');
  await expect(mark).toHaveCount(1);
  await expect(mark).toHaveText('Note: Type supplied by Intern (+1 more)');
  await expect(history.getByRole('row').filter({ has: page.getByRole('note') })).toContainText('Scan 0098.pdf');
  await page.keyboard.press('Escape');
  await expect(history).toBeHidden();

  // The filter is a switch: off again, both filed documents are listed.
  await filter.click();
  await expect(page.getByRole('row')).toHaveCount(3);
});
