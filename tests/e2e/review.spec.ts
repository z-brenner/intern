import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails, queueCell } from './grid';

/*
  The review panel scrolls under its decision - Approve & rename, Keep
  original - which sticks to the bottom over whatever is behind it. The
  browser scrolls a focused control into view only when none of it is in
  the panel, so one focused under the buttons, or half under them, stayed
  there: "Analysis details" reached by Tab. A control focused from the
  keyboard, or from code after a key, is now scrolled clear of the buttons.
  After a press nothing is scrolled until a key is pressed (below).
*/

/** Where the focused control is hidden, or '' when it can be seen: the points just inside its top and bottom edges are the control's own. */
const hiddenFocus = (page: Page) => page.evaluate(() => {
  const focused = document.activeElement as HTMLElement | null;
  if (!focused) return 'nothing focused';
  const box = focused.getBoundingClientRect();
  for (const y of [box.top + Math.min(box.height / 2, 6), box.bottom - Math.min(box.height / 2, 6)]) {
    const hit = document.elementFromPoint(box.left + box.width / 2, y);
    if (hit && (hit === focused || focused.contains(hit))) continue;
    const label = focused.getAttribute('aria-label') ?? focused.textContent?.trim().slice(0, 40);
    return `${focused.tagName} "${label}" is under ${hit?.closest('.inspector-actions') ? 'the decision' : hit ? hit.tagName : 'nothing on screen'}`;
  }
  return '';
});

/** Tab from the open panel's heading to its decision, and say each control Tab reached that could not be seen. */
async function tabThroughOpenPanel(page: Page, panel: Locator, name: string): Promise<string[]> {
  const hidden: string[] = [];
  await panel.evaluate((element) => { element.scrollTop = 0; element.querySelector<HTMLElement>('h2')?.focus(); });
  for (let step = 0; step < 40; step += 1) {
    await page.keyboard.press('Tab');
    const inContent = await page.evaluate(() => {
      const focused = document.activeElement;
      return Boolean(focused?.closest('.inspector') && !focused.closest('.inspector-actions'));
    });
    if (!inContent) break;
    const problem = await hiddenFocus(page);
    if (problem) hidden.push(`${name}: ${problem}`);
  }
  return hidden;
}

/** Open the named document's panel and Tab through it. */
async function tabThroughPanel(page: Page, name: string, prepare?: (panel: Locator) => Promise<void>): Promise<string[]> {
  const panel = await openDetails(page, name);
  await prepare?.(panel);
  return tabThroughOpenPanel(page, panel, name);
}

const narrow = (page: Page) => page.evaluate(() => window.matchMedia('(max-width: 1100px)').matches);

for (const size of [{ width: 1024, height: 768 }, { width: 800, height: 600 }]) {
  test(`each control in the review panel is in sight when Tab reaches it, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/');
    const hidden: string[] = [];
    for (const name of ['Employment Agreement - John Smith.pdf', 'Lease Agreement - 123 Main St.pdf', 'NDA - Acme Corp.docx', 'Service Agreement - BlueSky LLC.pdf', 'Completed lease.pdf']) {
      if (name === 'Completed lease.pdf') await page.getByRole('button', { name: /^Completed/ }).click();
      hidden.push(...await tabThroughPanel(page, name));
      if (await narrow(page)) await page.keyboard.press('Escape');
    }
    expect(hidden).toEqual([]);
  });
}

// Clear of the decision however tall it is now: a parked document's has a
// third row, Check again, above the usual two.
test('a control is scrolled clear of a taller decision too', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/');
  const hidden = await tabThroughPanel(page, 'Lease Agreement - 123 Main St.pdf', async (panel) => {
    const decision = panel.locator('.inspector-actions');
    const before = (await decision.boundingBox())!.height;
    await decision.evaluate((element) => { element.style.paddingBottom = '66px'; });
    expect((await decision.boundingBox())!.height).toBeGreaterThan(before + 40);
  });
  expect(hidden).toEqual([]);
});

// Never so far that the start of the control leaves the panel: a field
// taller than the room above the decision - a description dragged taller -
// shows its first lines.
test('a field taller than the room above the decision shows its start', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/');
  const panel = await openDetails(page, 'Lease Agreement - 123 Main St.pdf');
  const description = panel.getByRole('textbox', { name: 'Description' });
  await description.evaluate((field) => { field.style.height = '500px'; });
  await panel.evaluate((element) => { element.scrollTop = 0; element.querySelector<HTMLElement>('h2')?.focus(); });
  for (let step = 0; step < 20 && !await description.evaluate((field) => field === document.activeElement); step += 1) await page.keyboard.press('Tab');
  await expect(description).toBeFocused();
  const start = await description.evaluate((field) => {
    const box = field.getBoundingClientRect();
    return document.elementFromPoint(box.left + box.width / 2, box.top + 6) === field;
  });
  expect(start).toBe(true);
});

// A press is left where it is: a control half under the decision, scrolled
// clear as the press began, moved out from under the pointer, and the press
// landed on nothing. Only while it lasts.
test('a press on a control half under the decision lands on it', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/');
  const panel = await openDetails(page, 'Lease Agreement - 123 Main St.pdf');
  const details = panel.locator('details', { has: page.locator('summary', { hasText: 'Analysis details' }) });
  await expect(details).not.toHaveAttribute('open');
  await panel.evaluate((element) => {
    element.querySelector<HTMLElement>('h2')?.focus();
    const summary = element.querySelector('details > summary')!;
    const decision = element.querySelector('.inspector-actions')!;
    element.scrollTop += summary.getBoundingClientRect().top - decision.getBoundingClientRect().top + 10;
  });
  const box = (await details.locator('summary').boundingBox())!;
  const decisionTop = (await panel.locator('.inspector-actions').boundingBox())!.y;
  expect(box.y).toBeLessThan(decisionTop);
  expect(box.y + box.height).toBeGreaterThan(decisionTop);
  await page.mouse.click(box.x + 40, box.y + 4);
  await expect(details).toHaveAttribute('open', '');
  // The press is over: the next control focused from the keyboard is
  // scrolled clear again.
  expect(await tabThroughOpenPanel(page, panel, 'Lease Agreement - 123 Main St.pdf')).toEqual([]);
});

/*
  The browser's undo and redo go to the field edited last in the page,
  wherever the focus is, and take the focus there. With Settings open over
  the review panel, Ctrl+Z in a field of Settings with nothing of its own to
  undo undid the edit to the panel's Filename behind the dialog and put the
  caret in it; what was typed next went into the name, and Enter filed the
  document, with Settings still open over it. While a dialog is open, no
  field behind it changes.
*/
for (const size of [{ width: 1400, height: 900 }, { width: 1280, height: 720 }]) {
  test(`no undo or redo in Settings changes the review panel's name behind it, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/');
    const panel = await openDetails(page, 'Lease Agreement - 123 Main St.pdf');
    const name = panel.getByRole('textbox', { name: 'Filename' });
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2');
    const edited = await name.inputValue();
    expect(edited).toMatch(/ v2$/);
    const status = () => queueCell(page, 'Lease Agreement - 123 Main St.pdf', 'status').textContent();
    const before = await status();

    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await dialog.locator('summary', { hasText: 'Shared folders and troubleshooting' }).click();
    const machine = dialog.getByRole('textbox', { name: "This machine's name" });
    const close = dialog.getByRole('button', { name: 'Close settings' });
    for (const [where, focusOn] of [['in this machine\'s name', () => machine.click()], ['on Close', () => close.focus()]] as const) {
      for (const key of ['Control+Z', 'Control+Y', 'Control+Shift+Z']) {
        await focusOn();
        await page.keyboard.press(key);
        await page.waitForTimeout(100);
        await expect(name, `${key} ${where}`).toHaveValue(edited);
        await expect(name, `${key} ${where}`).not.toBeFocused();
      }
    }
    // What is typed next, and Enter, are Settings' own: the document is not filed.
    const machineName = await machine.inputValue();
    await machine.click();
    await page.keyboard.press('Control+Z');
    await page.keyboard.type('x');
    await expect(machine).toHaveValue(`${machineName}x`);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await expect(name).toHaveValue(edited);
    expect(await status()).toBe(before);
    // The field's own undo is the browser's, as before.
    await machine.click();
    await page.keyboard.press('Control+Z');
    await expect(machine).toHaveValue(machineName);

    // Closed, the name's own undo is the browser's again.
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
    await name.click();
    await page.keyboard.press('Control+Z');
    await expect(name).not.toHaveValue(edited);
    expect(await status()).toBe(before);
  });
}

/*
  Nothing behind Settings takes the focus while it is open: the window
  behind it is inert, and Tab and Shift+Tab go round its own controls from
  wherever the focus is. A press on its heading or its backdrop left the
  focus on nothing, and Shift+Tab walked out of Settings to the review
  panel behind it - Keep original, Approve & rename, the Filename - where
  Enter filed the held document with Settings still open; narrow, to the
  queue's row and to Rename all ready.
*/
const HELD = 'SOW final v3.pdf';
const CONTOSO = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
const focusInSettings = (page: Page) => page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"][aria-label="Settings"]')));

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`Tab and Shift+Tab stay in Settings after a press on its heading or its backdrop, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    // Wide, the held document open in review with its name edited; narrow, its row selected.
    if (size.width > 1100) {
      const name = (await openDetails(page, HELD)).getByRole('textbox', { name: 'Filename' });
      await name.click();
      await page.keyboard.press('End');
      await page.keyboard.type(' DRAFT');
    } else await queueCell(page, HELD).click();
    const statuses = () => page.locator('.queue-grid [data-column="status"]').allTextContents();
    const before = await statuses();
    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await dialog.locator('summary', { hasText: 'Learned spellings' }).click();
    for (const asking of [false, true]) {
      if (asking) {
        await dialog.getByRole('button', { name: CONTOSO }).click();
        await page.keyboard.type('Cont');
      }
      for (const [where, press] of [['its heading', () => dialog.locator('.dialog-head h2').click()], ['its backdrop', () => page.mouse.click(4, 4)]] as const) {
        for (const key of ['Shift+Tab', 'Tab']) {
          await press();
          for (let step = 1; step <= 7; step += 1) {
            await page.keyboard.press(key);
            expect(await focusInSettings(page), `${key} ${step} after a press on ${where}${asking ? ', the question asking' : ''}`).toBe(true);
          }
        }
      }
    }
    await expect(dialog).toBeVisible();
    await expect(page.locator('.toasts')).toHaveCount(0);
    expect(await statuses()).toEqual(before);
  });
}

/*
  Narrow, the review drawer opens again for a document it was open for, and
  puts the caret in its name. Opened again under Settings - the window made
  narrower while the question asked - it took the focus from the question,
  and Enter, meant for the question, filed the held document.
*/
test('the review drawer opened again under Settings takes no focus from it', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/?learnedSpellings=held');
  const name = (await openDetails(page, HELD)).getByRole('textbox', { name: 'Filename' });
  await page.setViewportSize({ width: 1400, height: 900 });
  const status = () => queueCell(page, HELD, 'status').textContent();
  const before = await status();
  await page.getByRole('button', { name: 'Settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.locator('summary', { hasText: 'Learned spellings' }).click();
  await dialog.getByRole('button', { name: CONTOSO }).click();
  const field = dialog.getByRole('group', { name: CONTOSO }).getByRole('textbox');
  await page.keyboard.type('Cont');
  await expect(field).toBeFocused();
  await page.setViewportSize({ width: 800, height: 600 });
  await expect(page.getByRole('dialog', { name: 'Review item' })).toBeVisible();
  await page.waitForTimeout(300);
  await expect(field).toBeFocused();
  const shown = await name.inputValue();
  await page.keyboard.type('oso');
  await expect(field).toHaveValue('Contoso');
  await expect(name).toHaveValue(shown);
  await page.keyboard.press('Enter');
  await expect(dialog.getByRole('group', { name: CONTOSO })).toHaveCount(0);
  await expect(dialog).toBeVisible();
  await page.waitForTimeout(300);
  await expect(page.locator('.toasts')).toHaveCount(0);
  expect(await status()).toBe(before);
  await expect(name).toHaveValue(shown);
});
