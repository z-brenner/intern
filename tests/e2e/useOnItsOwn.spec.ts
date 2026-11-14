import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails } from './grid';

/*
  The yes of "Use on its own" answers only once the spelling is typed out.
  Rounds that let the yes take some presses and not others - not the second
  of a double-click, not within half a second of the question, only from a
  pointer moved onto it - kept finding presses that got through: a slow
  double-click, a scroll, a tap through a menu, a window closing over it.
  Here no press, on the action that opens the question or on the yes
  itself, checks a spelling, in the review panel or in Settings, at two
  window sizes; typing the spelling does, with a click on the yes or with
  Enter. Nor does text put in at once with no keys for it. And while the
  question asks, no undo or redo changes a field behind it.
*/

const CONTOSO = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
const RIDGELINE = 'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography';

type Point = { x: number; y: number };
type Surface = {
  /** What holds the question: the review panel, or Settings. */
  root: Locator;
  /** What scrolls it. */
  scroller: Locator;
  /** How many spellings it shows as used on their own. */
  checked(): Promise<number>;
};

/** The review panel at the held statement of work. */
async function reviewPanel(page: Page): Promise<Surface> {
  await page.goto('/?learnedSpellings=held');
  const root = await openDetails(page, 'SOW final v3.pdf');
  // Two of its spellings are unchecked, each with an action or, while it asks, a question.
  return { root, scroller: root, checked: async () => 2 - await root.locator('.spelling-checks [data-rule], .spelling-checks .rule-confirm').count() };
}

/** Settings, at its learned spellings. */
async function settings(page: Page): Promise<Surface> {
  await page.goto('/?learnedSpellings=held');
  await page.getByRole('button', { name: 'Settings' }).click();
  const root = page.getByRole('dialog', { name: 'Settings' });
  await root.locator('summary', { hasText: 'Learned spellings' }).click();
  return { root, scroller: root.locator('.dialog-body'), checked: () => root.locator('.rule-state', { hasText: /^In use$/ }).count() };
}

const surfaces: [string, (page: Page) => Promise<Surface>][] = [['review panel', reviewPanel], ['Settings', settings]];
const sizes = [{ width: 1400, height: 900 }, { width: 800, height: 600 }];

const centre = async (locator: Locator): Promise<Point> => {
  const box = (await locator.boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
};

/**
 * The named action scrolled a quarter of the way down what scrolls it, or
 * with `atTheBottom` to the bottom of what can be seen of it there - just
 * above the review panel's decision - so the question it opens reaches
 * past it.
 */
async function actionInSight(surface: Surface, label: string, atTheBottom = false) {
  await surface.scroller.evaluate((element, [name, bottom]) => {
    const action = [...element.querySelectorAll('button')].find((button) => button.getAttribute('aria-label') === name)!;
    const box = action.getBoundingClientRect();
    const seen = element.getBoundingClientRect();
    const end = element.querySelector('.inspector-actions')?.getBoundingClientRect().top ?? seen.bottom;
    element.scrollTop += bottom ? box.bottom + 4 - end : box.top - seen.top - element.clientHeight / 4;
  }, [label, atTheBottom] as const);
  return surface.root.getByRole('button', { name: label });
}

/** What a press at `at` lands on: the question's yes, its field, its sentence, or else the element. */
const under = (page: Page, at: Point) => page.evaluate(({ x, y }) => {
  const hit = document.elementFromPoint(x, y);
  if (!hit?.closest('.rule-confirm')) return hit?.getAttribute('aria-label') ?? hit?.tagName ?? 'nothing';
  if (hit.closest('button.primary')) return 'the yes';
  if (hit.closest('input')) return 'the field';
  return hit.closest('p') ? 'the sentence' : hit.tagName;
}, at);

/** Whether the whole of `part` can be seen: nothing over the top or the bottom of it, and neither out of the window. */
const seenWhole = (part: Locator) => part.evaluate((element) => {
  const box = element.getBoundingClientRect();
  return [box.top + 3, box.bottom - 3].every((y) => {
    const hit = document.elementFromPoint(box.left + Math.min(box.width / 2, 40), y);
    return hit !== null && (hit === element || element.contains(hit));
  });
});

/** The parts of `question` that cannot be seen whole: its sentence, its field, its yes and its Cancel. */
async function outOfSight(question: Locator) {
  const parts: [string, Locator][] = [
    ['the sentence', question.locator('p')], ['the field', question.getByRole('textbox')],
    ['the yes', question.getByRole('button', { name: 'Use on its own' })], ['Cancel', question.getByRole('button', { name: 'Cancel' })],
  ];
  const hidden: string[] = [];
  for (const [name, part] of parts) if (!await seenWhole(part)) hidden.push(name);
  return hidden;
}

/** Every press there is at `at`, each after a press there that opened the question or found it open. */
const presses: [string, (page: Page, at: Point) => Promise<void>][] = [
  ['a double-click', (page, at) => page.mouse.dblclick(at.x, at.y)],
  ['a triple-click', (page, at) => page.mouse.click(at.x, at.y, { clickCount: 3 })],
  ['a slow double-click, 700 ms apart', async (page, at) => { await page.mouse.click(at.x, at.y); await page.waitForTimeout(700); await page.mouse.click(at.x, at.y); }],
  ['a press again after a second', async (page, at) => { await page.mouse.click(at.x, at.y); await page.waitForTimeout(1000); await page.mouse.click(at.x, at.y); }],
  ['a press, and one held for 600 ms', async (page, at) => { await page.mouse.click(at.x, at.y); await page.mouse.down(); await page.waitForTimeout(600); await page.mouse.up(); }],
  ['a press, and one after the pointer went well away and came back', async (page, at) => {
    await page.mouse.click(at.x, at.y);
    await page.mouse.move(at.x - 150, at.y - 120, { steps: 5 });
    await page.waitForTimeout(300);
    await page.mouse.move(at.x, at.y, { steps: 8 });
    await page.mouse.click(at.x, at.y);
  }],
];

for (const [name, open] of surfaces) {
  for (const size of sizes) {
    test(`no press checks a spelling, and typing it does, in the ${name} at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const surface = await open(page);
      const { root } = surface;
      const before = await surface.checked();
      const question = root.getByRole('group', { name: CONTOSO });
      const field = question.getByRole('textbox');
      const yes = question.getByRole('button', { name: 'Use on its own' });
      const takenBack = async (what: string) => {
        await page.waitForTimeout(150);
        expect(await surface.checked(), what).toBe(before);
        await expect(question, what).toBeVisible();
        await expect(yes, what).toBeDisabled();
        await page.keyboard.press('Escape');
        await expect(question, what).toHaveCount(0);
        await expect(root, what).toBeVisible();
      };

      // On the action: the first press opens the question, and the rest land where it opened.
      for (const [what, pressIt] of presses) {
        const at = await centre(await actionInSight(surface, CONTOSO));
        await pressIt(page, at);
        expect(await under(page, at), what).not.toBe('the yes');
        await takenBack(`on the action: ${what}`);
      }
      // On the yes itself, the question opened from the keyboard.
      for (const [what, pressIt] of presses) {
        await (await actionInSight(surface, CONTOSO)).focus();
        await page.keyboard.press('Enter');
        await expect(field).toBeFocused();
        const at = await centre(yes);
        expect(await under(page, at), what).toBe('the yes');
        await pressIt(page, at);
        await takenBack(`on the yes: ${what}`);
      }

      // Escape after a press on the sentence takes back the question, and not the panel or Settings.
      const action = await actionInSight(surface, CONTOSO);
      await action.click();
      await question.locator('p').click();
      await expect(field).not.toBeFocused();
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);
      await expect(root).toBeVisible();
      await expect(action).toBeFocused();

      // Opened from the keyboard where it reaches past what can be seen, the
      // whole question is brought into sight: its sentence, its field and both buttons.
      await (await actionInSight(surface, CONTOSO, true)).focus();
      await page.keyboard.press('Enter');
      await expect(field).toBeFocused();
      expect(await outOfSight(question), 'opened from the keyboard').toEqual([]);
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);

      // Opened there by a press, it moves nothing, and - where the action
      // could be scrolled to the bottom of what can be seen - reaches past
      // it; the spelling typed into it brings it into sight.
      const atTheBottom = await actionInSight(surface, CONTOSO, true);
      const reaches = await surface.scroller.evaluate((element, name) => {
        const action = [...element.querySelectorAll('button')].find((button) => button.getAttribute('aria-label') === name)!;
        const end = element.querySelector('.inspector-actions')?.getBoundingClientRect().top ?? element.getBoundingClientRect().bottom;
        return action.getBoundingClientRect().bottom > end - 8;
      }, CONTOSO);
      const pressedAt = await centre(atTheBottom);
      await page.mouse.click(pressedAt.x, pressedAt.y);
      await expect(field).toBeFocused();
      expect(await under(page, pressedAt)).not.toBe('the yes');
      if (reaches) expect(await outOfSight(question), 'opened by a press').not.toEqual([]);
      await page.keyboard.type('contoso');
      expect(await outOfSight(question), 'typed into').toEqual([]);

      // Typed out, then the yes clicked: checked.
      await expect(yes).toBeEnabled();
      await yes.click();
      await expect(question).toHaveCount(0);
      await expect.poll(() => surface.checked()).toBe(before + 1);

      // Opened by a press, typed out, then Enter: checked, and Settings is neither saved nor closed.
      const ridgeline = root.getByRole('group', { name: RIDGELINE });
      await (await actionInSight(surface, RIDGELINE)).click();
      await expect(ridgeline.getByRole('textbox')).toBeFocused();
      await page.keyboard.type('Ridgeline Cartography');
      await page.keyboard.press('Enter');
      await expect(ridgeline).toHaveCount(0);
      await expect.poll(() => surface.checked()).toBe(before + 2);
      await expect(root).toBeVisible();
    });
  }
}

/** The field each surface has that a person may have edited, unsaved, before the question asks: the document's name, and this machine's name. */
async function editedBehind(name: string, root: Locator) {
  if (name === 'review panel') return root.getByRole('textbox', { name: 'Filename' });
  await root.locator('summary', { hasText: 'Shared folders and troubleshooting' }).click();
  return root.getByRole('textbox', { name: "This machine's name" });
}

// An undo or a redo goes to the field edited last, wherever the focus is,
// and takes the focus there. Ctrl+Z pressed in the question's field before
// anything was typed in it undid the edit to the document's name, and the
// spelling typed next, with Enter as the question asks, went into the name
// and filed the document as "... Contoso vContoso.pdf"; in Settings it
// undid an unsaved edit to this machine's name.
for (const [name, open] of surfaces) {
  for (const size of sizes) {
    test(`no undo or redo from the question changes a field edited behind it, in the ${name} at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const surface = await open(page);
      const { root } = surface;
      const before = await surface.checked();
      const edited = await editedBehind(name, root);
      await edited.click();
      await page.keyboard.press('End');
      await page.keyboard.type(' v2');
      const typed = await edited.inputValue();
      expect(typed).toMatch(/ v2$/);
      const question = root.getByRole('group', { name: CONTOSO });
      const field = question.getByRole('textbox');
      const ask = async () => {
        await (await actionInSight(surface, CONTOSO)).click();
        await expect(field).toBeFocused();
      };
      const unchanged = async (what: string, value: string) => {
        await page.waitForTimeout(100);
        await expect(edited, what).toHaveValue(value);
        await expect(edited, what).not.toBeFocused();
        await expect(question, what).toBeVisible();
      };

      // Undo in the question's field, and after a press on its sentence took the focus from it.
      await ask();
      await page.keyboard.press('Control+Z');
      await unchanged('Ctrl+Z in the field', typed);
      await expect(field).toBeFocused();
      await question.locator('p').click();
      await expect(field).not.toBeFocused();
      await page.keyboard.press('Control+Z');
      await unchanged('Ctrl+Z after a press on the sentence', typed);
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);

      // Undone in the field it was made in, the edit can be redone there, and not from the question.
      await edited.focus();
      await page.keyboard.press('Control+Z');
      const undone = await edited.inputValue();
      expect(undone).not.toBe(typed);
      await ask();
      for (const key of ['Control+Y', 'Control+Shift+Z']) {
        await page.keyboard.press(key);
        await unchanged(key, undone);
        await expect(field).toBeFocused();
      }
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);
      await edited.focus();
      await page.keyboard.press('Control+Y');
      await expect(edited).toHaveValue(typed);

      // Ctrl+Z, then the spelling and Enter, as the question asks: the spelling is checked, and nothing else changes.
      await ask();
      await page.keyboard.press('Control+Z');
      await page.keyboard.type('Contoso');
      await page.keyboard.press('Enter');
      await expect(question).toHaveCount(0);
      await expect.poll(() => surface.checked()).toBe(before + 1);
      await expect(edited).toHaveValue(typed);
      await expect(root).toBeVisible();
    });
  }
}

// A paste is no typing. The same paste goes into the document's description.
test('a paste puts nothing in the field', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.setViewportSize({ width: 1400, height: 900 });
  const { root } = await reviewPanel(page);
  await root.getByRole('button', { name: CONTOSO }).click();
  const field = root.getByRole('group', { name: CONTOSO }).getByRole('textbox');
  await page.evaluate(() => navigator.clipboard.writeText('Contoso'));
  await expect(field).toBeFocused();
  await page.keyboard.press('Control+V');
  await page.keyboard.press('Shift+Insert');
  await expect(field).toHaveValue('');
  await expect(root.getByRole('group', { name: CONTOSO }).getByRole('button', { name: 'Use on its own' })).toBeDisabled();
  const description = root.getByRole('textbox', { name: 'Description' });
  await description.focus();
  await page.keyboard.press('Control+Home');
  await page.keyboard.press('Control+V');
  await expect(description).toHaveValue(/^Contoso/);
});

// A key types one character. A word put in at once with no keys for it -
// an input method's clipboard, an on-screen keyboard's, an emoji keyboard's,
// as Chromium is told to put text in "that doesn't come from a key press" -
// is no typing, and one press then checked the spelling. An input method
// composing, or committing what its keys spelled, is.
test('a word put in at once with no keys for it puts nothing in the field, and an input method puts in what it spelled', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  const surface = await reviewPanel(page);
  const { root } = surface;
  const before = await surface.checked();
  const cdp = await page.context().newCDPSession(page);
  const keysTaken = async (count: number) => {
    for (let key = 0; key < count; key += 1) {
      for (const type of ['rawKeyDown', 'keyUp']) await cdp.send('Input.dispatchKeyEvent', { type: type as 'rawKeyDown' | 'keyUp', key: 'Process', code: 'KeyA', windowsVirtualKeyCode: 229, nativeVirtualKeyCode: 229 });
    }
  };
  const question = root.getByRole('group', { name: CONTOSO });
  const field = question.getByRole('textbox');
  const yes = question.getByRole('button', { name: 'Use on its own' });

  await root.getByRole('button', { name: CONTOSO }).click();
  await expect(field).toBeFocused();
  // With no key, and after the two that open an input method's clipboard and take its first entry.
  await cdp.send('Input.insertText', { text: 'Contoso' });
  await keysTaken(2);
  await cdp.send('Input.insertText', { text: 'Contoso' });
  await page.waitForTimeout(100);
  await expect(field).toHaveValue('');
  await expect(yes).toBeDisabled();
  const onTheYes = await centre(yes);
  await page.mouse.click(onTheYes.x, onTheYes.y);
  await page.waitForTimeout(150);
  expect(await surface.checked()).toBe(before);

  // Composed, and committed.
  await cdp.send('Input.imeSetComposition', { text: 'contoso', selectionStart: 7, selectionEnd: 7 });
  await cdp.send('Input.insertText', { text: 'Contoso' });
  await expect(field).toHaveValue('Contoso');
  await expect(yes).toBeEnabled();
  await page.keyboard.press('Escape');
  await expect(question).toHaveCount(0);

  // Committed without a composition shown, after the keys that spelled it.
  await root.getByRole('button', { name: CONTOSO }).click();
  await expect(field).toHaveValue('');
  await keysTaken(7);
  await cdp.send('Input.insertText', { text: 'Contoso' });
  await expect(field).toHaveValue('Contoso');
  await yes.click();
  await expect(question).toHaveCount(0);
  await expect.poll(() => surface.checked()).toBe(before + 1);
});

test.describe('with a touch screen', () => {
  test.use({ hasTouch: true });
  for (const [name, open] of surfaces) {
    for (const size of sizes) {
      test(`no tap checks a spelling, in the ${name} at ${size.width}x${size.height}`, async ({ page }) => {
        await page.setViewportSize(size);
        const surface = await open(page);
        const before = await surface.checked();
        const question = surface.root.getByRole('group', { name: CONTOSO });
        const yes = question.getByRole('button', { name: 'Use on its own' });
        for (const gap of [100, 650]) {
          const at = await centre(await actionInSight(surface, CONTOSO));
          await page.touchscreen.tap(at.x, at.y);
          await expect(question).toBeVisible();
          await page.waitForTimeout(gap);
          await page.touchscreen.tap(at.x, at.y);
          const on = await centre(yes);
          await page.touchscreen.tap(on.x, on.y);
          await page.waitForTimeout(gap);
          await page.touchscreen.tap(on.x, on.y);
          await page.waitForTimeout(150);
          await expect(question, `${gap}ms`).toBeVisible();
          expect(await surface.checked(), `${gap}ms`).toBe(before);
          await question.getByRole('button', { name: 'Cancel' }).tap();
          await expect(question).toHaveCount(0);
        }
      });
    }
  }
});
