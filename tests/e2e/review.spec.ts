import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails } from './grid';

/*
  The review panel scrolls under its decision - Approve & rename, Keep
  original - which sticks to the bottom over whatever is behind it. The
  browser scrolls a focused control into view only when none of it is in
  the panel, so one focused under the buttons, or half under them, stayed
  there: "Analysis details" reached by Tab, and the question "Use on its
  own" asks, with its buttons cut off. A control focused from the keyboard,
  or from code after a key, is now scrolled clear of the buttons; a control
  of that question brings the whole question clear.
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

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`the question "Use on its own" asks is in sight, its field and buttons and all, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const actions = panel.getByRole('button', { name: /^Use on its own:/ });
    await expect(actions).toHaveCount(2);
    for (const label of await actions.evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''))) {
      await panel.getByRole('button', { name: label }).focus();
      await page.keyboard.press('Enter');
      const question = panel.getByRole('group', { name: label });
      await expect(question.getByRole('textbox')).toBeFocused();
      expect(await hiddenFocus(page), label).toBe('');
      // The sentence it answers, the yes and Cancel under the field, are in sight too.
      for (const part of [question.locator('p'), question.getByRole('button', { name: 'Use on its own' }), question.getByRole('button', { name: 'Cancel' })]) {
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
  The question opens where its action was, sentence first. The panel used
  to scroll it clear of the decision, and the scroll brought its yes under
  the pointer that had pressed the action: a double-click a little slower
  than the system's, or a press again because nothing seemed to happen,
  used the spelling on its own with the sentence never read, and from then
  on a name it changed could be filed without anyone looking at it. Now
  nothing a press sets off moves what is under the pointer: the next press
  lands on the sentence. And the yes answers only once the spelling is
  typed out (useOnItsOwn.spec.ts).
*/

type Pressed = {
  /** The scroll the press was made at. */
  top: number;
  /** The point pressed. */
  at: { x: number; y: number };
  /** What a press there would land on now. */
  under: string;
  /** Whether the scroller moved. */
  moved: boolean;
  /** Whether the question's field took the focus. */
  fieldFocused: boolean;
  /** Whether the question reaches past what the scroller shows of it: under the decision, or past the dialog's body. */
  reaches: boolean;
  /** Whether scrolling the question clear of the decision, as the panel used to, would have left its yes under the pointer. */
  scrollWouldBringTheYes: boolean;
};

/**
 * Open the named question with one real press on its action at every scroll
 * of `scroller`, `step` px apart, at which the action can be pressed, and
 * say what each press left under the pointer. The question is taken back
 * with its Cancel between presses.
 */
async function pressAtEveryScroll(page: Page, scroller: Locator, label: string, step: number): Promise<Pressed[]> {
  const pressed: Pressed[] = [];
  const max = await scroller.evaluate((element) => element.scrollHeight - element.clientHeight);
  for (let top = 0; top <= max; top += step) {
    const at = await scroller.evaluate((element, [scroll, name]) => {
      element.scrollTop = Number(scroll);
      const action = [...element.querySelectorAll<HTMLButtonElement>('button[aria-label]')].find((button) => button.getAttribute('aria-label') === name);
      if (!action) throw new Error(`no action ${name}`);
      const box = action.getBoundingClientRect();
      const [x, y] = [box.left + box.width / 2, box.top + box.height / 2];
      return document.elementFromPoint(x, y) === action ? { x, y, top: element.scrollTop } : undefined;
    }, [String(top), label]);
    if (!at) continue;
    await page.mouse.click(at.x, at.y);
    pressed.push(await scroller.evaluate((element, point) => {
      const question = element.querySelector('.rule-confirm');
      if (!question) throw new Error('the press opened no question');
      const yes = question.querySelector<HTMLElement>('button.primary')!;
      const cancel = [...question.querySelectorAll<HTMLElement>('button')].find((button) => button.textContent === 'Cancel')!;
      const hit = document.elementFromPoint(point.x, point.y);
      const under = hit === yes ? 'the yes' : hit === cancel ? 'Cancel' : hit?.closest('.rule-confirm p') ? 'the sentence' : hit ? `${hit.tagName}.${hit.className}` : 'nothing';
      // Where the question opened, before any scroll that followed the press.
      const drift = element.scrollTop - point.top;
      const opened = (box: DOMRect) => ({ left: box.left, right: box.right, top: box.top + drift, bottom: box.bottom + drift });
      const decision = element.querySelector('.inspector-actions');
      const limit = decision ? decision.getBoundingClientRect().top : element.getBoundingClientRect().bottom;
      const cancelOpened = opened(cancel.getBoundingClientRect());
      const reaches = cancelOpened.bottom > limit;
      // The scroll the panel used to make for its Cancel: clear of the decision by 12px, its top kept in the panel.
      let scrollWouldBringTheYes = false;
      if (decision) {
        const shift = Math.min(cancelOpened.bottom + 12 - limit, cancelOpened.top - 12 - element.getBoundingClientRect().top);
        const was = opened(yes.getBoundingClientRect());
        scrollWouldBringTheYes = shift > 0 && point.x >= was.left && point.x <= was.right && point.y >= was.top - shift && point.y <= was.bottom - shift;
      }
      const result = { top: point.top, at: { x: point.x, y: point.y }, under, moved: drift !== 0, fieldFocused: document.activeElement === question.querySelector('input'), reaches, scrollWouldBringTheYes };
      cancel.click();
      return result;
    }, at));
  }
  return pressed;
}

/** The presses that left anything but the sentence under the pointer, moved the scroller, or left the focus out of the question's field. */
const wrongPresses = (pressed: Pressed[]) => pressed
  .filter((press) => press.under !== 'the sentence' || press.moved || !press.fieldFocused)
  .map((press) => `at ${press.top}: ${press.under}${press.moved ? ', scrolled' : ''}${press.fieldFocused ? '' : ', focus not in the field'}`);

for (const size of [{ width: 1400, height: 900 }, { width: 1024, height: 768 }, { width: 800, height: 600 }]) {
  test(`a press on "Use on its own" moves nothing, and leaves the question's sentence under the pointer, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const labels = await panel.getByRole('button', { name: /^Use on its own:/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''));
    expect(labels).toHaveLength(2);
    for (const label of labels) {
      const pressed = await pressAtEveryScroll(page, panel, label, 4);
      expect(pressed.length, label).toBeGreaterThan(10);
      // The hazard is there to be met: at some scrolls the question opens
      // reaching under the decision, and scrolling it clear would leave its
      // yes under the pointer.
      expect(pressed.filter((press) => press.reaches).length, label).toBeGreaterThan(0);
      expect(pressed.filter((press) => press.scrollWouldBringTheYes).length, label).toBeGreaterThan(0);
      expect(wrongPresses(pressed), label).toEqual([]);
    }
  });
}

// The same question in Settings: the browser scrolled its Cancel into view
// when it opened past the bottom of the dialog's body, and what came under
// the pointer was whatever the scroll brought there.
for (const size of [{ width: 1280, height: 720 }, { width: 800, height: 600 }]) {
  test(`a press on "Use on its own" in Settings moves nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await dialog.locator('summary', { hasText: 'Learned spellings' }).click();
    const body = dialog.locator('.dialog-body');
    const labels = await dialog.getByRole('button', { name: /^Use on its own:/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''));
    expect(labels).toHaveLength(2);
    let reaching = 0;
    for (const label of labels) {
      const pressed = await pressAtEveryScroll(page, body, label, 8);
      expect(pressed.length, label).toBeGreaterThan(10);
      reaching += pressed.filter((press) => press.reaches).length;
      expect(wrongPresses(pressed), label).toEqual([]);
    }
    expect(reaching).toBeGreaterThan(0);
  });
}

/** The press on the action at the middle of the scrolls where the panel's old scroll would have brought the yes under the pointer. */
async function whereTheYesCameUnderThePointer(page: Page, panel: Locator, label: string) {
  const pressed = (await pressAtEveryScroll(page, panel, label, 4)).filter((press) => press.scrollWouldBringTheYes);
  expect(pressed.length, label).toBeGreaterThan(0);
  return pressed[Math.floor(pressed.length / 2)];
}

/** Scroll the panel back to `top`, and say where the named action's centre is. */
async function actionAt(panel: Locator, label: string, top: number) {
  await panel.evaluate((element, at) => { element.scrollTop = at; }, top);
  const box = (await panel.getByRole('button', { name: label }).boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`no press that follows a press on "Use on its own" answers its question, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const checked = panel.locator('.spelling-checked');
    const labels = await panel.getByRole('button', { name: /^Use on its own:/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''));
    expect(labels).toHaveLength(2);
    for (const label of labels) {
      const question = panel.getByRole('group', { name: label });
      const { top } = await whereTheYesCameUnderThePointer(page, panel, label);
      const second = async (gap: number, clickCount = 1) => {
        await page.waitForTimeout(gap);
        await page.mouse.down({ clickCount }); await page.mouse.up({ clickCount });
      };
      const presses: [string, (at: { x: number; y: number }) => Promise<void>][] = [
        // A double-click slower than the system's double-click time: two single presses.
        ['a double-click a little slower than the system counts', async (at) => { await page.mouse.click(at.x, at.y); await second(520); }],
        ['a press again after a second', async (at) => { await page.mouse.click(at.x, at.y); await second(1000); }],
        ['a double-click, and a press again', async (at) => { await page.mouse.dblclick(at.x, at.y); await second(650); }],
        ['a second press begun at once and held past half a second', async (at) => {
          await page.mouse.click(at.x, at.y);
          await page.waitForTimeout(100);
          await page.mouse.down(); await page.waitForTimeout(600); await page.mouse.up();
        }],
        ['a double-click', (at) => page.mouse.dblclick(at.x, at.y)],
        ['a triple-click', (at) => page.mouse.click(at.x, at.y, { clickCount: 3 })],
        ['two single presses straight after each other', async (at) => { await page.mouse.click(at.x, at.y); await page.mouse.click(at.x, at.y); }],
        // A double-click as slow as a system can be set to count one.
        ['a slow double-click', async (at) => { await page.mouse.click(at.x, at.y); await second(800, 2); }],
      ];
      for (const [what, pressIt] of presses) {
        const at = await actionAt(panel, label, top);
        await pressIt(at);
        await expect(question, `${label}: ${what}`).toBeVisible();
        await page.waitForTimeout(150);
        await expect(checked, `${label}: ${what}`).toHaveCount(0);
        await expect(question.getByRole('button', { name: 'Use on its own' }), `${label}: ${what}`).toBeDisabled();
        expect(await panel.evaluate((element) => element.scrollTop), `${label}: ${what}`).toBe(top);
        expect(await page.evaluate(({ x, y }) => Boolean(document.elementFromPoint(x, y)?.closest('.rule-confirm p')), at), `${label}: ${what}`).toBe(true);
        // Escape takes back the question, and the drawer stays open.
        await page.keyboard.press('Escape');
        await expect(question).toHaveCount(0);
        await expect(panel).toBeVisible();
      }
    }
  });
}

// A finger the same: the second tap lands on the sentence.
test.describe('with a touch screen', () => {
  test.use({ hasTouch: true });
  test('a second tap on "Use on its own" answers nothing', async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const checked = panel.locator('.spelling-checked');
    const label = (await panel.getByRole('button', { name: /^Use on its own:/ }).first().getAttribute('aria-label'))!;
    const question = panel.getByRole('group', { name: label });
    const { top } = await whereTheYesCameUnderThePointer(page, panel, label);
    for (const gap of [650, 100]) {
      const at = await actionAt(panel, label, top);
      await page.touchscreen.tap(at.x, at.y);
      await expect(question).toBeVisible();
      await page.waitForTimeout(gap);
      await page.touchscreen.tap(at.x, at.y);
      await page.waitForTimeout(150);
      await expect(question, `${gap}ms`).toBeVisible();
      await expect(checked, `${gap}ms`).toHaveCount(0);
      expect(await panel.evaluate((element) => element.scrollTop), `${gap}ms`).toBe(top);
      await question.getByRole('button', { name: 'Cancel' }).tap();
      await expect(question).toHaveCount(0);
    }
  });
});
