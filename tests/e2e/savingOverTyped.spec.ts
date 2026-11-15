import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails, queueCell } from './grid';

/*
  With automatic renaming on, Intern renames every ready document nobody has
  approved, and it does not know what a person typed in review and did not
  file. So Settings says, above Save, when a save would let it rename such a
  document: automatic renaming switched on over a ready one, or Strict
  changed to Balanced, with automatic renaming on, over one waiting in
  review. The warning is words only, and it goes when the choice is taken
  back. Its coming and going moves nothing a person could be about to press:
  the footer grows upward over the sections' last lines once, and keeps
  that room, blank, when the warning has gone. The journey further down
  holds what that leaves: nothing that acts comes under a pointer at rest
  where the warning was.

  The demo does not act out automatic renaming: nothing in it is renamed by
  itself. So this journey ends at the save. What becomes of the document and
  of what was typed is run in savingOverTyped.test.tsx, on rows as the
  desktop backend sends them.
*/

const LEASE = 'Lease Agreement - 123 Main St.pdf';
const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
const ONE = '1 document has a name or a description you typed and did not file. When you save, Intern can rename it under the name it proposed. You can file it first, or take back what you typed. What you typed is kept with the document, but is not filed.';
const TWO = '2 documents have a name or a description you typed and did not file. When you save, Intern can rename them under the names it proposed. You can file them first, or take back what you typed. What you typed is kept with each document, but is not filed.';

/** Settings, at "Naming and filing", where the two choices are. */
async function openSettings(page: Page) {
  await page.getByRole('button', { name: 'Settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.locator('summary', { hasText: 'Naming and filing' }).click();
  const level = dialog.getByRole('group', { name: 'Which documents are ready' });
  return {
    dialog,
    strict: level.getByRole('radio', { name: /^Strict/ }),
    balanced: level.getByRole('radio', { name: /^Balanced/ }),
    automatic: dialog.getByRole('checkbox', { name: 'Automatically rename ready documents' }),
    save: dialog.getByRole('button', { name: 'Save settings' }),
    warning: dialog.getByRole('alert').filter({ hasText: 'you typed and did not file' }),
    sections: dialog.locator('.dialog-body'),
  };
}

/**
 * Where everything in Settings that can be pressed or typed in is, in the
 * window: every control of the open sections, in view or scrolled out of it.
 * Those of a closed section are not drawn anywhere.
 */
const controls = (dialog: Locator) => dialog.locator('button, summary, input, select, textarea').evaluateAll((all) => all.filter((control) => control.checkVisibility()).map((control) => {
  const box = control.getBoundingClientRect();
  return `${control.tagName} ${control.getAttribute('aria-label') ?? control.closest('label')?.textContent?.trim().slice(0, 30) ?? control.textContent?.trim().slice(0, 30) ?? ''} at ${Math.round(box.left)},${Math.round(box.top)}`;
}));

test('Settings warns before a save that lets Intern rename a document with a name typed and not filed, and the warning goes with the choice', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');

  // A name typed for the lease, which waits in review, and not filed.
  const panel = await openDetails(page, LEASE);
  await panel.getByLabel('Filename').fill('2023-09-15 Lease of 123 Main St');
  await expect(panel.getByRole('button', { name: "Back to Intern's proposal" })).toBeVisible();

  let settings = await openSettings(page);
  await expect(settings.strict).toBeChecked();
  await expect(settings.automatic).not.toBeChecked();
  await expect(settings.warning).toHaveCount(0);
  // Balanced alone renames nothing by itself.
  await settings.balanced.check();
  await expect(settings.warning).toHaveCount(0);
  // The two levels and the checkbox in the middle of the view, where each can be pressed without the sections
  // being scrolled for it, warning or no warning: what moves from here on, the warning moved.
  await settings.automatic.evaluate((input) => input.scrollIntoView({ block: 'center' }));
  const before = await controls(settings.dialog);

  // Balanced with automatic renaming on: the lease can become ready, and Intern can then rename it.
  await settings.automatic.check();
  await expect(settings.warning).toHaveText(ONE);
  await expect(settings.warning).toBeInViewport({ ratio: 1 });
  // Above Save, which is still there to press, and nothing new to press with it.
  await expect(settings.save).toBeEnabled();
  await expect(settings.warning.locator('xpath=following-sibling::*[1]')).toHaveText('Save settings');
  await expect(settings.warning.locator('button, a, input')).toHaveCount(0);
  // Nothing in Settings moved to make room for it: not Save, not the choices, not a section.
  expect(await controls(settings.dialog)).toEqual(before);

  // Either choice taken back, it is gone, and nothing moves as it goes or comes back.
  await settings.strict.check();
  await expect(settings.warning).toHaveCount(0);
  expect(await controls(settings.dialog)).toEqual(before);
  await settings.balanced.check();
  await expect(settings.warning).toHaveText(ONE);
  expect(await controls(settings.dialog)).toEqual(before);
  await settings.automatic.uncheck();
  await expect(settings.warning).toHaveCount(0);
  expect(await controls(settings.dialog)).toEqual(before);
  // Nothing was saved, and nothing behind Settings changed.
  await page.keyboard.press('Escape');
  await expect(settings.dialog).toBeHidden();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Needs review');
  await expect(panel.getByLabel('Filename')).toHaveValue('2023-09-15 Lease of 123 Main St');

  // A ready document typed for as well: automatic renaming alone reaches that one, and Balanced adds the lease.
  const employment = await openDetails(page, EMPLOYMENT);
  await employment.getByLabel('Filename').fill('2024-04-12 Employment Agreement of John Smith');
  settings = await openSettings(page);
  await expect(settings.warning).toHaveCount(0);
  await settings.automatic.check();
  await expect(settings.warning).toHaveText(ONE);
  await settings.balanced.check();
  await expect(settings.warning).toHaveText(TWO);

  // At the window's smallest it is in view with Save, whatever part of the sections is.
  await page.setViewportSize({ width: 800, height: 600 });
  await expect(settings.warning).toHaveText(TWO);
  await expect(settings.warning).toBeInViewport({ ratio: 1 });
  await expect(settings.save).toBeInViewport({ ratio: 1 });
  // The sections are scrolled to their end while the warning shows, which is further than they went before it
  // took its room; the pointer is left on the last button there; and the warning goes, here by the space bar on
  // the checkbox, which keeps the focus and scrolls nothing. The room stays the footer's, so the sections stay
  // where they are, and the button stays under the pointer.
  await settings.automatic.focus();
  await settings.sections.hover();
  await page.mouse.wheel(0, 10_000);
  await expect.poll(() => settings.sections.evaluate((body) => Math.round(body.scrollHeight - body.clientHeight - body.scrollTop))).toBeLessThanOrEqual(1);
  const guide = settings.dialog.getByRole('button', { name: 'Open the guide' });
  await expect(guide).toBeInViewport({ ratio: 1 });
  await guide.hover();
  const underPointer = () => guide.evaluate((button) => { const box = button.getBoundingClientRect(); return document.elementFromPoint(box.left + box.width / 2, box.top + box.height / 2) === button; });
  expect(await underPointer()).toBe(true);
  const atTheEnd = await controls(settings.dialog);
  await page.keyboard.press('Space');
  await expect(settings.automatic).not.toBeChecked();
  await expect(settings.warning).toHaveCount(0);
  expect(await controls(settings.dialog)).toEqual(atTheEnd);
  expect(await underPointer()).toBe(true);
  // And when it comes back, the same.
  await page.keyboard.press('Space');
  await expect(settings.automatic).toBeChecked();
  await expect(settings.warning).toHaveText(TWO);
  expect(await controls(settings.dialog)).toEqual(atTheEnd);
  expect(await underPointer()).toBe(true);
  await page.setViewportSize({ width: 1280, height: 800 });

  // The warning does not hold the save back. Saved, the lease is ready with a note, and nothing in the demo is
  // renamed by itself: both names typed are still in their fields, to file or to take back.
  await settings.save.click();
  await expect(settings.dialog).toBeHidden();
  await expect(queueCell(page, LEASE, 'status')).toHaveText('Ready with a note');
  await expect(page.getByRole('button', { name: 'Completed, 1' })).toBeVisible();
  await expect(page.locator('.toasts')).toHaveCount(0);
  const leaseAgain = await openDetails(page, LEASE);
  await expect(leaseAgain.getByLabel('Filename')).toHaveValue('2023-09-15 Lease of 123 Main St');
  const employmentAgain = await openDetails(page, EMPLOYMENT);
  await expect(employmentAgain.getByLabel('Filename')).toHaveValue('2024-04-12 Employment Agreement of John Smith');

  // Saved as they are, the settings have nothing more to warn of.
  settings = await openSettings(page);
  await expect(settings.balanced).toBeChecked();
  await expect(settings.automatic).toBeChecked();
  await expect(settings.warning).toHaveCount(0);
});

/*
  The warning comes and goes by itself too: a document with a name typed for
  it is read again, or filed from another computer, while Settings is open.
  It is words, and a pointer can be at rest on them. When it went, the footer
  used to give its room back to the sections, and whatever was scrolled
  behind it came under that pointer: with a real press 0.1 s later, Check for
  updates checked, and Forget forgot a learned spelling. Nothing in Settings
  is marked as a control that acts, so no press there is refused for being
  too soon. Now the room it took stays where it was, blank, until Settings
  is closed: nothing is uncovered, and nothing moves.

  `?queue=changes` is the demo's queue with `window.internQueue` to change it
  as the backend does. The press is a real one, at a place, not at a control.
*/
type QueueByItself = { change(id: string, change: Record<string, unknown>): void };
/** The lease is read again, or comes back from being read, with nobody asking. */
const byItself = (page: Page, how: 'read again' | 'back with a reading') => page.evaluate((how) => {
  const queue = (window as unknown as { internQueue: QueueByItself }).internQueue;
  if (how === 'read again') queue.change('lease', { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined });
  else queue.change('lease', { status: 'review', stage: undefined, proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', reason: 'Two dates were found.' });
}, how);
/** What is on top at a place: a control by its tag and name, or the class of whatever else is there. */
const at = (page: Page, place: { x: number; y: number }) => page.evaluate(({ x, y }) => {
  const top = document.elementFromPoint(x, y);
  const control = top?.closest('button, summary, input, select, textarea, label, a');
  if (control) return `${control.tagName} ${(control.getAttribute('aria-label') ?? control.textContent ?? '').trim().slice(0, 40)}`;
  return top ? `no control: ${top.tagName}.${top.className}` : 'nothing';
}, place);
const edges = async (locator: Locator) => { const box = (await locator.boundingBox())!; return `${Math.round(box.y)}..${Math.round(box.y + box.height)}`; };

for (const [width, height] of [[1280, 800], [800, 600]] as const) {
  test(`a warning that goes by itself leaves nothing that acts where it was, at ${width} by ${height}`, async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto('/?queue=changes');
    await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
    const panel = await openDetails(page, LEASE);
    await panel.getByLabel('Filename').fill('2023-09-15 Lease of 123 Main St');
    await page.setViewportSize({ width, height });
    const settings = await openSettings(page);
    const footer = settings.dialog.locator('.dialog-actions');
    await settings.balanced.check();
    await settings.automatic.check();
    await expect(settings.warning).toHaveText(ONE);

    // A button that acts, scrolled to where the warning is: behind it, out of sight. The pointer comes to rest on
    // the warning's words, over that button.
    const check = settings.dialog.getByRole('button', { name: 'Check for updates' });
    const words = (await settings.warning.boundingBox())!;
    await check.evaluate((button, y) => {
      const box = button.getBoundingClientRect();
      button.closest('.dialog-body')!.scrollTop += box.top + box.height / 2 - y;
    }, words.y + words.height / 2);
    const behind = (await check.boundingBox())!;
    const place = { x: behind.x + behind.width / 2, y: behind.y + behind.height / 2 };
    expect(Math.abs(place.y - (words.y + words.height / 2))).toBeLessThanOrEqual(1);
    expect(place.x).toBeGreaterThan(words.x + 13);
    expect(place.x).toBeLessThan(words.x + words.width - 10);
    await page.mouse.move(place.x, place.y);
    expect(await at(page, place)).toBe('no control: P.save-warning');
    const shown = await controls(settings.dialog);
    const room = { footer: await edges(footer), sections: await edges(settings.sections), save: await edges(settings.save) };

    // The lease is read again: it is not waiting for a name, and the warning goes, by itself.
    await byItself(page, 'read again');
    await expect(settings.warning).toHaveCount(0);
    // Its room is still there, with nothing in it: no button has come under the pointer, and nothing moved.
    expect(await at(page, place)).toMatch(/^no control: /);
    expect({ footer: await edges(footer), sections: await edges(settings.sections), save: await edges(settings.save) }).toEqual(room);
    expect(await controls(settings.dialog)).toEqual(shown);
    // A press there, with the pointer not moved, does nothing: no update was checked for.
    await page.mouse.down();
    await page.mouse.up();
    await page.waitForTimeout(400);
    await expect(settings.dialog.getByRole('status', { name: 'Update status' })).toHaveCount(0);
    await expect(check).toHaveText('Check for updates');
    // The choices stand, and Save is where it was.
    await expect(settings.balanced).toBeChecked();
    await expect(settings.automatic).toBeChecked();

    // It comes back by itself, in the room it had, and again nothing moves.
    await byItself(page, 'back with a reading');
    await expect(settings.warning).toHaveText(ONE);
    expect(await at(page, place)).toBe('no control: P.save-warning');
    expect({ footer: await edges(footer), sections: await edges(settings.sections), save: await edges(settings.save) }).toEqual(room);
    expect(await controls(settings.dialog)).toEqual(shown);

    // The button is still there to be pressed on purpose, once scrolled into view.
    await check.scrollIntoViewIfNeeded();
    await check.click();
    await expect(settings.dialog.getByRole('status', { name: 'Update status' })).toBeVisible();
  });
}
