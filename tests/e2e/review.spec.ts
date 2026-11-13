import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails } from './grid';

/*
  The review panel scrolls under its decision - Approve & rename, Keep
  original - which sticks to the bottom over whatever is behind it. The
  browser scrolls a focused control into view only when none of it is in
  the panel, so one focused under the buttons, or half under them, stayed
  there: "Analysis details" reached by Tab, and the Cancel of "Use on its
  own", with the sentence it answers cut off. A control focused from the
  keyboard or from code is now scrolled clear of the buttons.
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

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`the question "Use on its own" asks is in sight, Cancel and all, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const actions = panel.getByRole('button', { name: /^Use on its own:/ });
    await expect(actions).toHaveCount(2);
    for (const label of await actions.evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''))) {
      await panel.getByRole('button', { name: label }).focus();
      await page.keyboard.press('Enter');
      const question = panel.getByRole('group', { name: label });
      await expect(question.getByRole('button', { name: 'Cancel' })).toBeFocused();
      expect(await hiddenFocus(page), label).toBe('');
      // The sentence it answers, and the yes beside Cancel, are in sight too.
      for (const part of [question.locator('p'), question.getByRole('button', { name: 'Use on its own' })]) {
        const hit = await part.evaluate((element) => {
          const box = element.getBoundingClientRect();
          const bottom = document.elementFromPoint(box.left + box.width / 2, box.bottom - 4);
          return Boolean(bottom && (bottom === element || element.contains(bottom)));
        });
        expect(hit, `${label}: ${await part.evaluate((element) => element.textContent?.slice(0, 40))}`).toBe(true);
      }
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);
    }
  });
}

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
  The question opens where its action was, and the panel then scrolls it
  clear of the decision - which can bring its yes under the pointer that
  pressed the action. The second press of a double-click on the action, or
  a second press straight after the first, then used the spelling on its
  own with the sentence never read, and from then on a name it changed
  could be filed without anyone looking at it. The yes takes a press only
  when it is meant for it: not part of a double-click, and not within a
  double-click's time of the question appearing or of a press it refused.
*/

/** Scroll positions of the panel at which a press on the action leaves the question's yes under the pointer. */
const yesUnderThePointer = (panel: Locator, label: string) => panel.evaluate(async (element, name) => {
  const frame = () => new Promise((resolve) => requestAnimationFrame(resolve));
  const found: number[] = [];
  for (let top = 0; top <= element.scrollHeight - element.clientHeight; top += 2) {
    element.scrollTop = top;
    await frame();
    const action = [...element.querySelectorAll<HTMLButtonElement>('button[aria-label]')].find((button) => button.getAttribute('aria-label') === name);
    if (!action) throw new Error(`no action ${name}`);
    const box = action.getBoundingClientRect();
    const [x, y] = [box.left + box.width / 2, box.top + box.height / 2];
    if (document.elementFromPoint(x, y) !== action) continue;
    action.click();
    await frame();
    const question = element.querySelector('.rule-confirm');
    const yes = question?.querySelector('button.primary');
    if (yes && document.elementFromPoint(x, y) === yes) found.push(top);
    question?.querySelector<HTMLButtonElement>('button:not(.primary)')?.click();
    await frame();
  }
  return found;
}, label);

/** The centre of the named action, with the panel scrolled to `top`. */
async function actionAt(panel: Locator, label: string, top: number) {
  await panel.evaluate((element, at) => { element.scrollTop = at; }, top);
  const box = (await panel.getByRole('button', { name: label }).boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`a press meant for "Use on its own" opens the question and answers nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const checked = panel.locator('.spelling-checked');
    const labels = await panel.getByRole('button', { name: /^Use on its own:/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''));
    expect(labels).toHaveLength(2);
    for (const label of labels) {
      const question = panel.getByRole('group', { name: label });
      const yes = question.getByRole('button', { name: 'Use on its own' });
      const tops = await yesUnderThePointer(panel, label);
      // The hazard is there to be met: at some scroll, one press on the
      // action leaves the yes under the pointer.
      expect(tops.length, label).toBeGreaterThan(0);
      const top = tops[Math.floor(tops.length / 2)];
      let at = await actionAt(panel, label, top);
      await page.mouse.click(at.x, at.y);
      await expect(question).toBeVisible();
      expect(await page.evaluate(({ x, y }) => Boolean(document.elementFromPoint(x, y)?.matches('.rule-confirm button.primary')), at), label).toBe(true);
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);

      const presses: [string, () => Promise<void>][] = [
        ['a double-click', () => page.mouse.dblclick(at.x, at.y)],
        ['a triple-click', () => page.mouse.click(at.x, at.y, { clickCount: 3 })],
        ['two single presses straight after each other', async () => { await page.mouse.click(at.x, at.y); await page.mouse.click(at.x, at.y); }],
        // A double-click as slow as a system can be set to count one.
        ['a slow double-click', async () => {
          await page.mouse.move(at.x, at.y);
          await page.mouse.down({ clickCount: 1 }); await page.mouse.up({ clickCount: 1 });
          await page.waitForTimeout(800);
          await page.mouse.down({ clickCount: 2 }); await page.mouse.up({ clickCount: 2 });
        }],
      ];
      for (const [what, pressIt] of presses) {
        at = await actionAt(panel, label, top);
        await pressIt();
        await expect(question, `${label}: ${what}`).toBeVisible();
        await page.waitForTimeout(150);
        await expect(checked, `${label}: ${what}`).toHaveCount(0);
        await expect(question.getByRole('button', { name: 'Cancel' }), `${label}: ${what}`).toBeFocused();
        await page.keyboard.press('Escape');
        await expect(question).toHaveCount(0);
      }

      // A press after the sentence had time to be read is a yes.
      at = await actionAt(panel, label, top);
      await page.mouse.click(at.x, at.y);
      await page.waitForTimeout(700);
      await yes.click();
      await expect(checked).toContainText('on its own from now on');
      await expect(panel.getByRole('button', { name: label })).toHaveCount(0);
    }
  });
}
