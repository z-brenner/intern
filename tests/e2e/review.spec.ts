import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { detailsToggle, openDetails, queueCell } from './grid';

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
  The browser's undo goes to the field edited last wherever the focus is,
  with no dialog open too. Ctrl+Z in the review panel's Description, in the
  queue's filter or after a press on the panel's text undid the Filename's
  last edit and put the caret in it, and the next Enter - a new line, in
  the Description - filed the document under the name half undone. An undo
  or a redo changes only the field with the focus.
*/
for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`no undo or redo from elsewhere in the review panel changes its name, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/');
    const lease = 'Lease Agreement - 123 Main St.pdf';
    const panel = await openDetails(page, lease);
    const name = panel.getByRole('textbox', { name: 'Filename' });
    const description = panel.getByRole('textbox', { name: 'Description' });
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2');
    const edited = await name.inputValue();
    expect(edited).toMatch(/ v2$/);
    const status = () => queueCell(page, lease, 'status').textContent();
    const before = await status();
    const elsewhere: [string, () => Promise<void>][] = [
      ['in the Description', () => description.click()],
      ['on the panel heading', () => panel.getByRole('heading', { name: 'Review item' }).focus()],
      ['after a press on the panel text', () => panel.locator('.selected-file').click()],
    ];
    // Narrow, the queue is inert under the drawer.
    if (!await narrow(page)) elsewhere.push(['in the queue filter', () => page.getByRole('searchbox', { name: 'Filter queue' }).click()]);
    for (const [where, focusOn] of elsewhere) {
      for (const key of ['Control+Z', 'Control+Y', 'Control+Shift+Z']) {
        await focusOn();
        await page.keyboard.press(key);
        await page.waitForTimeout(100);
        await expect(name, `${key} ${where}`).toHaveValue(edited);
        await expect(name, `${key} ${where}`).not.toBeFocused();
      }
    }
    // Enter in the Description after Ctrl+Z there is a new line in it, and files nothing.
    await description.click();
    const described = await description.inputValue();
    await page.keyboard.press('Control+Z');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await expect(description).toBeFocused();
    await expect(description).not.toHaveValue(described);
    await expect(name).toHaveValue(edited);
    expect(await status()).toBe(before);
    // In each field, its own undo is the browser's: the new line in the
    // Description, then the edit to the name. (The undo goes to the last
    // edit in the page, so the name's waits until the Description's is
    // undone.)
    await page.keyboard.press('Control+Z');
    await expect(description).toHaveValue(described);
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
  Narrow, the review drawer opened again for a document it had been open
  for, and put the caret in its name. Opened again under Settings - the
  window made narrower while the question asked - it took the focus from the
  question, and Enter, meant for the question, filed the held document.
  Nothing behind Settings takes the focus now, and the drawer does not open
  again by itself at all: it is asked for at the narrow width it opens at.
*/
test('no review drawer opens again under Settings, and nothing takes the focus from it', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/?learnedSpellings=held');
  const name = (await openDetails(page, HELD)).getByRole('textbox', { name: 'Filename' });
  const shown = await name.inputValue();
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
  await page.waitForTimeout(300);
  await expect(page.getByRole('dialog', { name: 'Review item' })).toHaveCount(0);
  await expect(field).toBeFocused();
  await page.keyboard.type('oso');
  await expect(field).toHaveValue('Contoso');
  await page.keyboard.press('Enter');
  await expect(dialog.getByRole('group', { name: CONTOSO })).toHaveCount(0);
  await expect(dialog).toBeVisible();
  await page.waitForTimeout(300);
  await expect(page.locator('.toasts')).toHaveCount(0);
  expect(await status()).toBe(before);
  // Asked for again once Settings is closed, the held document's name is as it was.
  await dialog.getByRole('button', { name: 'Close settings' }).click();
  await expect(page.getByRole('dialog', { name: 'Review item' })).toHaveCount(0);
  await openDetails(page, HELD);
  await expect(name).toHaveValue(shown);
  await expect(page.locator('.toasts')).toHaveCount(0);
});

/*
  What was typed for a name and a description and not filed is the window's,
  not the review panel's, so it is still there when the panel comes back.
  The panel kept it, and the panel leaves the screen without anything being
  decided: a wide panel when the window is made narrower than 1100px - half
  of a 1920px screen is 960px - a drawer when it is made wider, and either
  by Close review. Each time both fields came back as Intern's proposal.
*/
const LEASE = 'Lease Agreement - 123 Main St.pdf';
const WIDE = { width: 1400, height: 900 };
const NARROW = { width: 800, height: 600 };

for (const [label, first, second] of [['a wide panel, the window made narrower and wider again', WIDE, NARROW], ['a drawer, the window made wider and narrower again', NARROW, WIDE]] as const) {
  test(`what was typed for a name and a description is still there: ${label}`, async ({ page }) => {
    await page.setViewportSize(first);
    await page.goto('/');
    const panel = page.locator('#review-panel');
    const name = panel.getByRole('textbox', { name: 'Filename' });
    const description = panel.getByRole('textbox', { name: 'Description' });
    await openDetails(page, LEASE);
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2 signed');
    await description.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' Countersigned copy received 3 May; clause 7 amended by hand.');
    const typed = { name: await name.inputValue(), description: await description.inputValue() };
    expect(typed.name).toMatch(/TenantCo Inc v2 signed$/);
    expect(typed.description).toMatch(/amended by hand\.$/);
    const stillThere = async () => {
      await expect(name).toHaveValue(typed.name);
      await expect(description).toHaveValue(typed.description);
    };
    const status = () => queueCell(page, LEASE, 'status').textContent();
    const before = await status();

    // A drawer opens only when asked for at the narrow width: back there, it
    // is closed until it is. A wide panel left open is there again.
    const backAtFirst = async () => {
      await page.setViewportSize(first);
      if (first !== NARROW) return;
      await page.waitForTimeout(200);
      await expect(panel).toHaveCount(0);
      await detailsToggle(page).click();
    };

    // At the other width the panel is off screen: narrow, a drawer opens
    // only when asked; wide, the panel was never left open.
    await page.setViewportSize(second);
    await expect(panel).toHaveCount(0);
    await backAtFirst();
    await stillThere();

    // Asked for at the other width, it has what was typed there too.
    await page.setViewportSize(second);
    await expect(panel).toHaveCount(0);
    await detailsToggle(page).click();
    await stillThere();
    await backAtFirst();
    await stillThere();

    // Closed by its own button and opened again.
    await panel.getByRole('button', { name: 'Close review' }).click();
    await expect(panel).toHaveCount(0);
    await detailsToggle(page).click();
    await stillThere();

    // Nothing was filed by any of it, and Enter in the name files what was typed.
    await expect(page.locator('.toasts')).toHaveCount(0);
    expect(await status()).toBe(before);
    await name.click();
    await page.keyboard.press('Enter');
    await expect(page.locator('.toasts')).toContainText('Renamed 1 document.');
    // Narrow, review has moved on in the drawer, over the queue.
    if (await page.getByRole('dialog', { name: 'Review item' }).isVisible()) await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Completed/ }).click();
    await expect(queueCell(page, LEASE, 'proposed')).toContainText(`${typed.name}.pdf`);
    await expect(queueCell(page, LEASE, 'description')).toContainText(typed.description);
  });
}

/*
  A name or a description with the caret in it leaves the screen only by
  something the person did. The panel closes by itself when the window
  changes width - a snap to half the screen, with the caret still in the
  description - and the caret went to nothing: the rest of the sentence was
  review's keys (j and k move the selection, Enter opens a document at its
  name and files it). What the fields held stays on screen instead, above
  the queue, read-only, with the caret in it.
*/
for (const [label, first, second] of [['a wide panel closed by a narrower window', WIDE, NARROW], ['a drawer closed by a wider window', NARROW, WIDE]] as const) {
  test(`what is being typed keeps the caret, and what is typed next decides nothing: ${label}`, async ({ page }) => {
    await page.setViewportSize(first);
    await page.goto('/');
    const panel = page.locator('#review-panel');
    const name = panel.getByRole('textbox', { name: 'Filename' });
    const description = panel.getByRole('textbox', { name: 'Description' });
    await openDetails(page, LEASE);
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2 signed');
    await description.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' Countersigned copy received 3 May');
    const typed = { name: await name.inputValue(), description: await description.inputValue() };
    const statuses = () => page.locator('.queue-grid [data-column="status"]').allTextContents();
    const before = await statuses();
    const selected = page.locator('.queue-grid tr[aria-current="true"]');
    await expect(selected).toContainText(LEASE);

    await page.setViewportSize(second);
    await expect(panel).toHaveCount(0);
    const kept = page.getByRole('textbox', { name: 'What you had typed' });
    await expect(kept).toBeFocused();
    await expect(kept).toHaveValue(`${typed.name}\n\n${typed.description}`);
    await expect(page.getByRole('group', { name: `Kept from ${LEASE}` })).toContainText('What you typed is kept with the document');

    // Tab, to go on to the next field; the rest of the sentence; and each route that decided a document nobody had looked at.
    await page.keyboard.press('Tab');
    await page.keyboard.type(' subject to both.');
    await page.keyboard.press('Control+Enter');
    await page.keyboard.press('ArrowUp');
    await page.keyboard.press('Control+Enter');
    await page.keyboard.type(' as taken by Deutsche Bank');
    await page.keyboard.press('Enter');
    await page.keyboard.press('Enter');
    await page.keyboard.press('Alt+k');
    await page.keyboard.press('Delete');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
    await expect(kept).toBeFocused();
    await expect(kept).toHaveValue(`${typed.name}\n\n${typed.description}`);
    await expect(panel).toHaveCount(0);
    await expect(page.getByRole('group', { name: 'Confirm removal' })).toHaveCount(0);
    await expect(page.locator('.toasts')).toHaveCount(0);
    await expect(selected).toContainText(LEASE);
    expect(await statuses()).toEqual(before);

    // Done takes it away; the focus goes to the view's own button, where Enter files nothing.
    await page.getByRole('button', { name: 'Done', exact: true }).click();
    await expect(kept).toHaveCount(0);
    await expect(page.getByRole('button', { name: /^Queue, \d+$/ })).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(panel).toHaveCount(0);
    await expect(page.locator('.toasts')).toHaveCount(0);
    // Asked for again, the panel has what was typed.
    await openDetails(page, LEASE);
    await expect(name).toHaveValue(typed.name);
    await expect(description).toHaveValue(typed.description);
    expect(await statuses()).toEqual(before);
  });
}

/*
  A drawer is asked for at the narrow width it opens at. Asked for once, it
  opened again by itself each time the window was next made narrow, over
  whatever row was selected by then and with the caret in its name - even
  after Close review at the wide width. The Enter meant for the row's cell,
  which opens details there, filed the document.
*/
for (const closedWide of [false, true]) {
  test(`a drawer asked for once does not open again by itself when the window is next made narrow${closedWide ? ', after Close review at the wide width' : ''}`, async ({ page }) => {
    const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
    await page.setViewportSize(NARROW);
    await page.goto('/');
    const drawer = page.getByRole('dialog', { name: 'Review item' });
    await queueCell(page, LEASE).click();
    await page.keyboard.press('Enter');
    await expect(drawer).toContainText(LEASE);

    await page.setViewportSize(WIDE);
    await expect(page.locator('#review-panel')).toHaveCount(0);
    if (closedWide) {
      await detailsToggle(page).click();
      await page.locator('#review-panel').getByRole('button', { name: 'Close review' }).click();
      await expect(page.locator('#review-panel')).toHaveCount(0);
    }
    const cell = queueCell(page, EMPLOYMENT);
    await cell.click();
    const status = () => queueCell(page, EMPLOYMENT, 'status').textContent();
    const before = await status();

    await page.setViewportSize(NARROW);
    await page.waitForTimeout(400);
    await expect(drawer).toHaveCount(0);
    await expect(cell).toBeFocused();
    // The Enter aimed at the cell opens the document's details, and files nothing.
    await page.keyboard.press('Enter');
    await expect(drawer).toContainText(EMPLOYMENT);
    await expect(drawer.getByRole('textbox', { name: 'Filename' })).toBeFocused();
    await page.waitForTimeout(300);
    await expect(page.locator('.toasts')).toHaveCount(0);
    expect(await status()).toBe(before);
  });
}

/*
  Rename all ready files Intern's proposal for each ready document. For one
  whose name a person had typed and not filed it sent the proposal all the
  same, said "Renamed 3 documents", and the typed name was gone. It leaves
  such a document for the person, counts what it will rename, and says what
  it left.
*/
test('Rename all ready leaves a ready document whose name was typed and not filed, and says so', async ({ page }) => {
  const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
  const NDA = 'NDA - Acme Corp.docx';
  await page.setViewportSize(WIDE);
  await page.goto('/');
  const panel = page.locator('#review-panel');
  const name = panel.getByRole('textbox', { name: 'Filename' });
  await openDetails(page, EMPLOYMENT);
  await name.click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' (corrected)');
  const typed = await name.inputValue();
  // Left for the next row, as review does.
  await queueCell(page, NDA).click();
  await expect(panel).toContainText(NDA);
  const all = page.getByRole('button', { name: 'Rename all ready' });
  await expect(all).toHaveText('Rename all ready 2');
  await expect(page.getByRole('button', { name: '3 ready' })).toBeVisible();

  await all.click();
  await expect(page.locator('.toasts')).toContainText('Renamed 2 documents. 1 was left for you: it has a name or a description you typed and did not file.');
  await expect(queueCell(page, EMPLOYMENT, 'status')).toContainText('Ready');
  await expect(queueCell(page, EMPLOYMENT, 'proposed')).toContainText('2024-04-12 Employment Agreement with John Smith.pdf');
  await expect(queueCell(page, NDA)).toHaveCount(0);

  // What was typed is still its name in review, and Enter there files it.
  await openDetails(page, EMPLOYMENT);
  await expect(name).toHaveValue(typed);
  await name.click();
  await page.keyboard.press('Enter');
  await expect(page.locator('.toasts')).toContainText('Renamed 1 document.');
  await page.getByRole('button', { name: /^Completed/ }).click();
  await expect(queueCell(page, EMPLOYMENT, 'proposed')).toContainText(`${typed}.pdf`);
  await expect(queueCell(page, NDA, 'proposed')).toContainText('2024-03-01 Non-Disclosure Agreement with Acme Corp.docx');
});

/*
  The fields go by themselves in other ways too, and the panel stays: Intern
  reads the document again with nobody asking (every untouched provisional
  reading at once, when the accurate model becomes usable - and typing is not
  touching), another machine files it, or its file is taken from the Queue
  folder. Typed after the Lease left the queue, " subject to both." and
  Ctrl+Enter filed the first document in the queue: the j selects it, the
  panel shows it, Ctrl+Enter approves it. After a re-reading, ArrowUp and
  Ctrl+Enter approved the document above; " Bank" and Alt+K kept another's
  original name; " taken", Delete and Enter removed one. `?queue=changes` is
  the demo's queue with `window.internQueue` to change it as the backend does.
*/
type QueueByItself = { change(id: string, change: Record<string, unknown>): void; leave(id: string): void };
const queueByItself = (page: Page, act: 'read again' | 'new reading' | 'left the queue' | 'filed elsewhere') => page.evaluate((how) => {
  const queue = (window as unknown as { internQueue: QueueByItself }).internQueue;
  if (how === 'read again') queue.change('lease', { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined });
  else if (how === 'new reading') queue.change('lease', { status: 'review', stage: undefined, proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', proposalRevision: '1' });
  else if (how === 'filed elsewhere') queue.change('lease', { status: 'completed', filedName: '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf' });
  else queue.leave('lease');
}, act);
/** Tab, to go on to the next field; the rest of a sentence; and each route that decided a document nobody had looked at. */
const typeOn = async (page: Page) => {
  await page.keyboard.press('Tab');
  await page.keyboard.type(' subject to both.');
  await page.keyboard.press('Control+Enter');
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('Control+Enter');
  await page.keyboard.type(' as taken by Deutsche Bank');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Alt+k');
  await page.keyboard.type(' taken');
  await page.keyboard.press('Delete');
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
};
/** Every other document's row, as it reads: its status and the name proposed for it. */
const otherRows = (page: Page) => page.locator('.queue-grid tbody tr').evaluateAll((rows) => rows.map((row) => `${row.querySelector('[data-column="original"]')?.textContent} | ${row.querySelector('[data-column="status"]')?.textContent} | ${row.querySelector('[data-column="proposed"]')?.textContent}`).filter((row) => !row.startsWith('Lease Agreement')));

for (const size of [WIDE, NARROW]) for (const how of ['read again', 'filed elsewhere', 'left the queue'] as const) {
  test(`a document that is ${how === 'left the queue' ? 'taken from the queue' : how} while its description is typed keeps what was typed and the caret, and what is typed next decides nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?queue=changes');
    const panel = await openDetails(page, LEASE);
    const name = panel.getByRole('textbox', { name: 'Filename' });
    const description = panel.getByRole('textbox', { name: 'Description' });
    await name.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' v2 signed');
    await description.click();
    await page.keyboard.press('Control+End');
    await page.keyboard.type(' Countersigned copy received 3 May');
    const typed = `${await name.inputValue()}\n\n${await description.inputValue()}`;
    const before = await otherRows(page);
    const kept = page.getByRole('textbox', { name: 'What you had typed' });
    const nothingDecided = async () => {
      await expect(kept).toBeFocused();
      await expect(kept).toHaveValue(typed);
      await expect(page.getByRole('group', { name: 'Confirm removal' })).toHaveCount(0);
      await expect(page.locator('.toasts')).toHaveCount(0);
      expect(await otherRows(page)).toEqual(before);
    };

    await queueByItself(page, how);
    await expect(description).toHaveCount(0);
    await expect(kept).toHaveAttribute('readonly', '');
    await nothingDecided();
    await typeOn(page);
    await nothingDecided();

    if (how === 'left the queue') {
      // Above the queue: the document and its panel have gone, and nothing is selected.
      await expect(page.locator('#review-panel')).toHaveCount(0);
      await expect(page.getByRole('group', { name: `Kept from ${LEASE}` })).toContainText('left the queue while you were in its name or description');
      await expect(page.locator('.queue-grid tr[aria-current="true"]')).toHaveCount(0);
      await page.getByRole('button', { name: 'Done', exact: true }).click();
      await expect(kept).toHaveCount(0);
      await expect(page.getByRole('button', { name: /^Queue, \d+$/ })).toBeFocused();
      expect(await otherRows(page)).toEqual(before);
      return;
    }
    // In the panel, which still shows the document.
    await expect(panel).toContainText(LEASE);
    await expect(panel.getByRole('group', { name: 'Kept from before this document changed' })).toBeVisible();
    if (how === 'filed elsewhere') return;

    // The new reading is shown as it is, beside what was typed: the caret is not in it, and nothing typed goes into it.
    await queueByItself(page, 'new reading');
    await expect(name).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
    await expect(description).toHaveValue('A lease of 123 Main St.');
    await nothingDecided();
    await typeOn(page);
    await nothingDecided();
    await expect(name).toHaveValue('2023-09-15 Lease between ABC Properties and TenantCo');
    await expect(description).toHaveValue('A lease of 123 Main St.');
    // Escape is the way out from the keyboard: to its own button, which takes it away. Only the person does.
    await page.keyboard.press('Escape');
    await expect(panel.getByRole('button', { name: 'Discard what I had' })).toBeFocused();
    await expect(panel).toBeVisible();
    await page.keyboard.press('Enter');
    await expect(kept).toHaveCount(0);
    await expect(panel.getByRole('heading', { name: 'Review item' })).toBeFocused();
    await expect(page.locator('.toasts')).toHaveCount(0);
    expect(await otherRows(page)).toEqual(before);
  });
}

/*
  A reading the person asks for is another matter: Analyze again starts from
  Intern's proposal, as it always has, and nothing of what was typed over the
  reading before is kept beside it.
*/
test('Analyze again starts from the new reading, with nothing kept beside it', async ({ page }) => {
  await page.setViewportSize(WIDE);
  await page.goto('/?queue=changes');
  const panel = await openDetails(page, LEASE);
  const name = panel.getByRole('textbox', { name: 'Filename' });
  const proposed = await name.inputValue();
  await name.click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type(' v2 signed');
  await panel.getByRole('button', { name: 'More review actions' }).click();
  await panel.getByRole('button', { name: 'Analyze again' }).click();
  await expect(queueCell(page, LEASE, 'status')).toContainText(/review/i, { timeout: 15_000 });
  await openDetails(page, LEASE);
  await expect(name).toHaveValue(proposed);
  await expect(page.getByRole('textbox', { name: 'What you had typed' })).toHaveCount(0);
});
