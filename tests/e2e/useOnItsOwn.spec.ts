import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails } from './grid';

/*
  "Use on its own" is asked in Settings, under Learned spellings, and
  nowhere else: asked in the review window, the spelling typed into it
  reached review's own keys, which move on, put the caret in a document's
  name and file it. The review window says why a held name waits and where
  "Use on its own" is.

  Its yes answers only once the spelling is typed out. Rounds that let the
  yes take some presses and not others - not the second of a double-click,
  not within half a second of the question, only from a pointer moved onto
  it - kept finding presses that got through: a slow double-click, a
  scroll, a tap through a menu, a window closing over it. Here no press, on
  the action that opens the question or on the yes itself, checks a
  spelling, at two window sizes; typing the spelling does, with a click on
  the yes or with Enter. Nor does text put in at once with no keys for it,
  nor a paste. While the question asks, what is typed after a press took
  the focus off its field goes into it, and no undo or redo changes a field
  behind it.
*/

const CONTOSO = 'Always use: Contoso Worldwide, Inc. written as Contoso';
const RIDGELINE = 'Always use: Ridgeline Cartography LLC written as Ridgeline Cartography';

type Point = { x: number; y: number };
type Surface = {
  /** What holds the question: Settings. */
  root: Locator;
  /** What scrolls it. */
  scroller: Locator;
  /** How many spellings it shows as used on their own. */
  checked(): Promise<number>;
};

/** Settings, at its learned spellings, of a fresh page or, with `fresh` false, of the page as it is. */
async function settings(page: Page, fresh = true): Promise<Surface> {
  if (fresh) await page.goto('/?learnedSpellings=held');
  await page.getByRole('button', { name: 'Settings' }).click();
  const root = page.getByRole('dialog', { name: 'Settings' });
  await root.locator('summary', { hasText: 'Learned spellings' }).click();
  return { root, scroller: root.locator('.dialog-body'), checked: () => root.locator('.rule-state', { hasText: /^In use$/ }).count() };
}

const sizes = [{ width: 1400, height: 900 }, { width: 800, height: 600 }];

const centre = async (locator: Locator): Promise<Point> => {
  const box = (await locator.boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
};

/**
 * The named action scrolled a quarter of the way down what scrolls it, or
 * with `atTheBottom` to the bottom of what can be seen of it there, so the
 * question it opens reaches past it.
 */
async function actionInSight(surface: Surface, label: string, atTheBottom = false) {
  await surface.scroller.evaluate((element, [name, bottom]) => {
    const action = [...element.querySelectorAll('button')].find((button) => button.getAttribute('aria-label') === name)!;
    const box = action.getBoundingClientRect();
    const seen = element.getBoundingClientRect();
    element.scrollTop += bottom ? box.bottom + 4 - seen.bottom : box.top - seen.top - element.clientHeight / 4;
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
    ['the yes', question.getByRole('button', { name: 'Always use' })], ['Cancel', question.getByRole('button', { name: 'Cancel' })],
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

for (const size of sizes) {
  test(`no press checks a spelling, and typing it does, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const surface = await settings(page);
    const { root } = surface;
    const before = await surface.checked();
    const question = root.getByRole('group', { name: CONTOSO });
    const field = question.getByRole('textbox');
    const yes = question.getByRole('button', { name: 'Always use' });
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

    // Escape after a press on the sentence takes back the question, and not Settings.
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
      return action.getBoundingClientRect().bottom > element.getBoundingClientRect().bottom - 8;
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

/** The presses that take the focus off the question's field, each from the question open or, for a double-click, from its action. */
const offTheField: [string, (page: Page, surface: Surface, question: Locator) => Promise<void>][] = [
  ['a press on its waiting yes', async (page, surface, question) => {
    await (await actionInSight(surface, CONTOSO)).click();
    const at = await centre(question.getByRole('button', { name: 'Always use' }));
    expect(await under(page, at)).toBe('the yes');
    await page.mouse.click(at.x, at.y);
  }],
  ['a press on its sentence', async (_, surface, question) => {
    await (await actionInSight(surface, CONTOSO)).click();
    await question.locator('p').click();
  }],
  ['a double-click on its action', async (page, surface) => {
    const at = await centre(await actionInSight(surface, CONTOSO));
    await page.mouse.dblclick(at.x, at.y);
  }],
];

// A press on its sentence or its waiting yes, or the second press of a
// double-click on its action, takes the focus off the question's field, and
// what was typed next went elsewhere. A character typed in Settings but in
// another field is the question's, and nothing is filed or saved.
for (const size of sizes) {
  test(`what is typed after a press took the focus off the field goes into it, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const surface = await settings(page);
    const { root } = surface;
    const before = await surface.checked();
    const statuses = () => page.locator('.queue-grid [data-column="status"]').allTextContents();
    const filed = await statuses();
    const question = root.getByRole('group', { name: CONTOSO });
    const field = question.getByRole('textbox');
    const notYet = question.getByRole('status');
    for (const [what, offIt] of offTheField) {
      await offIt(page, surface, question);
      await expect(question, what).toBeVisible();
      if (what !== 'a double-click on its action') await expect(field, what).not.toBeFocused();
      await page.keyboard.type('j');
      await expect(field, what).toBeFocused();
      await page.keyboard.press('Enter');
      await page.keyboard.type('k');
      await page.keyboard.press('Enter');
      await expect(field, what).toHaveValue('jk');
      await expect(notYet, what).toHaveText('That is not Contoso: type it as it is shown.');
      expect(await statuses(), what).toEqual(filed);
      await page.keyboard.press('Escape');
      await expect(question, what).toHaveCount(0);
    }
    // The spelling typed after a press on the sentence, and Enter: the spelling is checked, and nothing else.
    await (await actionInSight(surface, CONTOSO)).click();
    await question.locator('p').click();
    await expect(field).not.toBeFocused();
    await page.keyboard.type('Contoso');
    await expect(field).toHaveValue('Contoso');
    await page.keyboard.press('Enter');
    await expect(question).toHaveCount(0);
    await expect.poll(() => surface.checked()).toBe(before + 1);
    expect(await statuses()).toEqual(filed);
    await expect(root).toBeVisible();
  });
}

/** A field of Settings a person may have edited, unsaved, before the question asks: this machine's name. */
async function editedBehind(root: Locator) {
  await root.locator('summary', { hasText: 'Shared folders and troubleshooting' }).click();
  return root.getByRole('textbox', { name: "This machine's name" });
}

// An undo or a redo goes to the field edited last, wherever the focus is,
// and takes the focus there. Ctrl+Z pressed in the question's field before
// anything was typed in it undid an unsaved edit to this machine's name,
// and the spelling typed next went into it.
for (const size of sizes) {
  test(`no undo or redo from the question changes a field edited behind it, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const surface = await settings(page);
    const { root } = surface;
    const before = await surface.checked();
    const edited = await editedBehind(root);
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

// Once the question is taken back or answered, the undo it kept from
// unsaved settings was the browser's again: Ctrl+Z on its row, pressed to
// take back the check, undid this machine's name instead and put the caret
// in it, and Save stored what was left. An undo or a redo changes only the
// field with the focus, before, while and after the question asks.
for (const size of sizes) {
  test(`no undo or redo after the question changes an unsaved setting, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const surface = await settings(page);
    const { root } = surface;
    const before = await surface.checked();
    const edited = await editedBehind(root);
    await edited.click();
    await page.keyboard.press('End');
    await page.keyboard.type('-laptop');
    const typed = await edited.inputValue();
    const question = root.getByRole('group', { name: CONTOSO });
    const field = question.getByRole('textbox');
    const unchanged = async (what: string) => {
      for (const key of ['Control+Z', 'Control+Y', 'Control+Shift+Z']) {
        await page.keyboard.press(key);
        await page.waitForTimeout(100);
        await expect(edited, `${key} ${what}`).toHaveValue(typed);
        await expect(edited, `${key} ${what}`).not.toBeFocused();
      }
    };
    // Taken back with Escape, with the focus where Escape leaves it, and on the row.
    await (await actionInSight(surface, CONTOSO)).click();
    await page.keyboard.type('Cont');
    await page.keyboard.press('Escape');
    await expect(question).toHaveCount(0);
    await unchanged('after the question was taken back');
    await root.locator('li', { has: page.getByRole('button', { name: CONTOSO }) }).focus();
    await unchanged('on its row');
    // Answered with the spelling and Enter.
    await (await actionInSight(surface, CONTOSO)).click();
    await expect(field).toBeFocused();
    await page.keyboard.type('Contoso');
    await page.keyboard.press('Enter');
    await expect(question).toHaveCount(0);
    await expect.poll(() => surface.checked()).toBe(before + 1);
    await unchanged('after the question was answered');
    await root.getByRole('button', { name: 'Save settings' }).focus();
    await unchanged('on Save');
    // In this machine's name, its own undo is the browser's.
    await edited.click();
    await page.keyboard.press('Control+Z');
    await expect(edited).not.toHaveValue(typed);
  });
}

// A paste is no typing. The same paste goes into another field of Settings.
test('a paste puts nothing in the field', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.setViewportSize({ width: 1400, height: 900 });
  const surface = await settings(page);
  const { root } = surface;
  await (await actionInSight(surface, CONTOSO)).click();
  const field = root.getByRole('group', { name: CONTOSO }).getByRole('textbox');
  await page.evaluate(() => navigator.clipboard.writeText('Contoso'));
  await expect(field).toBeFocused();
  await page.keyboard.press('Control+V');
  await page.keyboard.press('Shift+Insert');
  await expect(field).toHaveValue('');
  await expect(root.getByRole('group', { name: CONTOSO }).getByRole('button', { name: 'Always use' })).toBeDisabled();
  await page.keyboard.press('Escape');
  const machine = await editedBehind(root);
  await machine.focus();
  await page.keyboard.press('Control+Home');
  await page.keyboard.press('Control+V');
  await expect(machine).toHaveValue(/^Contoso/);
});

// A key types one character. A word put in at once with no keys for it -
// an input method's clipboard, an on-screen keyboard's, an emoji keyboard's,
// as Chromium is told to put text in "that doesn't come from a key press" -
// is no typing, and one press then checked the spelling. An input method
// composing, or committing what its keys spelled, is.
test('a word put in at once with no keys for it puts nothing in the field, and an input method puts in what it spelled', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  const surface = await settings(page);
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
  const yes = question.getByRole('button', { name: 'Always use' });

  await (await actionInSight(surface, CONTOSO)).click();
  await expect(field).toBeFocused();
  // With no key, and after the two that open an input method's clipboard and take its first entry.
  await cdp.send('Input.insertText', { text: 'Contoso' });
  await keysTaken(2);
  await cdp.send('Input.insertText', { text: 'Contoso' });
  // Nor after keys that type nothing: an arrow, Enter, Home, Ctrl+A.
  for (const key of ['ArrowLeft', 'Enter', 'Home', 'Control+A']) {
    for (let pressed = 0; pressed < 7; pressed += 1) await page.keyboard.press(key);
    await cdp.send('Input.insertText', { text: 'Contoso' });
  }
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
  await (await actionInSight(surface, CONTOSO)).click();
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
  for (const size of sizes) {
    test(`no tap checks a spelling, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const surface = await settings(page);
      const before = await surface.checked();
      const question = surface.root.getByRole('group', { name: CONTOSO });
      const yes = question.getByRole('button', { name: 'Always use' });
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
});

/*
  Opened by a press, the question opens where its action was, sentence
  first, and nothing the press sets off moves what is under the pointer: the
  browser scrolled the focus into view when it opened past the bottom of
  Settings' body, and what came under the pointer for the next press was
  whatever the scroll brought there.
*/

type Pressed = {
  /** The scroll the press was made at. */
  top: number;
  /** What a press there would land on now. */
  under: string;
  /** Whether the scroller moved. */
  moved: boolean;
  /** Whether the question's field took the focus. */
  fieldFocused: boolean;
  /** Whether the question reaches past what the scroller shows of it. */
  reaches: boolean;
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
      const drift = element.scrollTop - point.top;
      const reaches = cancel.getBoundingClientRect().bottom + drift > element.getBoundingClientRect().bottom;
      const result = { top: point.top, under, moved: drift !== 0, fieldFocused: document.activeElement === question.querySelector('input'), reaches };
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

for (const size of [{ width: 1280, height: 720 }, { width: 800, height: 600 }]) {
  test(`a press on "Use on its own" in Settings moves nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const { root, scroller } = await settings(page);
    const labels = await root.getByRole('button', { name: /^Always use:/ }).evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label') ?? ''));
    expect(labels).toHaveLength(2);
    let reaching = 0;
    for (const label of labels) {
      const pressed = await pressAtEveryScroll(page, scroller, label, 8);
      expect(pressed.length, label).toBeGreaterThan(10);
      reaching += pressed.filter((press) => press.reaches).length;
      expect(wrongPresses(pressed), label).toEqual([]);
    }
    expect(reaching).toBeGreaterThan(0);
  });
}

/*
  The review window asks nothing: a document held for a spelling nobody has
  checked says why it waits and where "Use on its own" is, and no key typed
  there checks a spelling.
*/
for (const size of sizes) {
  test(`the review panel asks nothing of a held name and points to Settings, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const why = panel.getByRole('note', { name: 'Why the name waits' });
    await expect(why).toBeVisible();
    await expect(why).toHaveText('This name waits for you because ATLAS has not been told to use every one of these spellings on its own: Always use is in Settings, under Learned spellings.');
    await expect(panel.getByRole('button', { name: /Always use|Use on its own/ })).toHaveCount(0);
    await expect(panel.getByRole('group', { name: /Always use|Use on its own/ })).toHaveCount(0);
    await expect(panel.getByRole('textbox')).toHaveCount(2);
    // The spelling typed on the panel's heading, and Enter: it goes to the name, and nothing is checked.
    const name = panel.getByRole('textbox', { name: 'Filename' });
    const shown = await name.inputValue();
    await panel.getByRole('heading', { name: 'Review item' }).focus();
    await page.keyboard.type('Contoso');
    await page.keyboard.press('Enter');
    await expect(name).toBeFocused();
    await expect(name).toHaveValue(shown);
    await page.keyboard.press('Escape');
    if (await page.evaluate(() => window.matchMedia('(max-width: 1100px)').matches)) await expect(panel).toHaveCount(0);
    const dialog = (await settings(page, false)).root;
    await expect(dialog.locator('.rule-state', { hasText: 'asks before filing' })).toHaveCount(2);
  });
}
