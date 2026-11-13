import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { openDetails } from './grid';

/*
  The yes of "Use on its own" takes a press of the pointer only once the
  pointer came onto it after the question appeared. Each round before this
  stopped one way the yes came under a pointer that had not moved to it -
  the review panel's scroll, then a press again after a refused one - and
  the next round found another: in Settings, a press on a second spelling's
  "Use on its own" closed the question above it, the row moved up, and the
  new question's yes opened under the pointer; a question opened from the
  keyboard was scrolled under a resting mouse. Whatever brings the yes under
  a still pointer, a press there is now no answer; a person who moves the
  pointer onto it after reading is answered on the first press.
*/

const CONTOSO = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
const RIDGELINE = 'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography';

type Point = { x: number; y: number };

const centre = async (locator: Locator): Promise<Point> => {
  const box = (await locator.boundingBox())!;
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
};

/** What a press at `at` would land on: the yes or the sentence of a question, or else the element. */
const under = (page: Page, at: Point) => page.evaluate(({ x, y }) => {
  const hit = document.elementFromPoint(x, y);
  const question = hit?.closest('.rule-confirm');
  if (question && hit?.closest('button.primary')) return `the yes of ${question.getAttribute('aria-label')}`;
  if (question && hit?.closest('p')) return `the sentence of ${question.getAttribute('aria-label')}`;
  return hit?.getAttribute('aria-label') ?? `${hit?.tagName}.${hit?.className}`;
}, at);

/** A single press where the pointer is, held as a hand holds one. */
const pressHere = async (page: Page, clickCount = 1) => {
  await page.mouse.down({ clickCount }); await page.waitForTimeout(80); await page.mouse.up({ clickCount });
};

/** Open Settings at "Learned spellings", scrolled so both spellings' actions, and the question below them, are in sight. */
async function learnedSpellings(page: Page) {
  await page.goto('/?learnedSpellings=held');
  await page.getByRole('button', { name: 'Settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Settings' });
  await dialog.locator('summary', { hasText: 'Learned spellings' }).click();
  await dialog.locator('.dialog-body').evaluate((body, label) => {
    const action = [...body.querySelectorAll('button')].find((button) => button.getAttribute('aria-label') === label)!;
    body.scrollTop += action.getBoundingClientRect().top - body.getBoundingClientRect().top - 120;
  }, CONTOSO);
  return dialog;
}

/** The state each learned spelling's row says it is in. */
const states = (dialog: Locator) => dialog.locator('.learned-rules .rule-state').allTextContents();

/*
  The route the last review found, and the presses that followed it there:
  a double-click slower than the system's, a double-click and a press again,
  a press again after a second.
*/
const after: [string, (page: Page) => Promise<void>][] = [
  ['a double-click slower than the system\'s', async (page) => { await page.waitForTimeout(700); await pressHere(page); }],
  ['a double-click, and a press again', async (page) => { await page.waitForTimeout(150); await pressHere(page, 2); await page.waitForTimeout(650); await pressHere(page); }],
  ['a press again after a second', async (page) => { await page.waitForTimeout(1000); await pressHere(page); }],
];

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`in Settings, a press on another spelling's "Use on its own" while one is asked about does nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const dialog = await learnedSpellings(page);
    const before = await states(dialog);
    expect(before.filter((state) => state === 'In use · asks before filing')).toHaveLength(2);
    for (const [what, pressAgain] of after) {
      const contoso = dialog.getByRole('group', { name: CONTOSO });
      const opening = await centre(dialog.getByRole('button', { name: CONTOSO }));
      await page.mouse.click(opening.x, opening.y);
      await expect(contoso).toBeVisible();
      await page.waitForTimeout(500);
      // Where Ridgeline's action is now, below the question.
      const ridgeline = dialog.getByRole('button', { name: RIDGELINE });
      await expect(ridgeline, what).toBeDisabled();
      const at = await centre(ridgeline);
      await page.mouse.move(at.x, at.y);
      await pressHere(page);
      expect(await under(page, at), what).toBe(RIDGELINE);
      await pressAgain(page);
      await page.waitForTimeout(150);
      await expect(contoso, what).toBeVisible();
      await expect(dialog.locator('.rule-confirm'), what).toHaveCount(1);
      expect(await states(dialog), what).toEqual(before);
      await page.keyboard.press('Escape');
      await expect(contoso).toHaveCount(0);
      await expect(dialog).toBeVisible();
    }
  });
}

/*
  A question opened from the keyboard is scrolled into sight, Cancel and
  all, and the scroll can bring its yes under a mouse resting on the panel.
  The mouse did not move to it: a press there is no answer. Moved off it
  and back, it is.
*/
for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`a question opened from the keyboard takes no press from a mouse its scroll brought the yes under, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const action = panel.getByRole('button', { name: CONTOSO });
    const question = panel.getByRole('group', { name: CONTOSO });
    const yes = question.getByRole('button', { name: 'Use on its own' });
    // The action just clear of the decision, so the question it opens reaches under it and is scrolled up.
    const start = await panel.evaluate((element, label) => {
      const button = [...element.querySelectorAll('button')].find((candidate) => candidate.getAttribute('aria-label') === label)!;
      const decision = element.querySelector('.inspector-actions')!;
      element.scrollTop += button.getBoundingClientRect().bottom + 16 - decision.getBoundingClientRect().top;
      return element.scrollTop;
    }, CONTOSO);
    /** Tab to the action from the control before it, and press Enter. */
    const openFromTheKeyboard = async () => {
      await panel.evaluate((element, top) => { element.scrollTop = top; }, start);
      await action.evaluate((button) => {
        const focusable = [...button.closest('.inspector')!.querySelectorAll<HTMLElement>('button:not(:disabled), textarea:not(:disabled), input:not(:disabled), summary')];
        focusable[focusable.indexOf(button) - 1].focus({ preventScroll: true });
      });
      await page.keyboard.press('Tab');
      await expect(action).toBeFocused();
      await page.keyboard.press('Enter');
      await expect(question.getByRole('button', { name: 'Cancel' })).toBeFocused();
    };
    // Where the yes ends up, found with the mouse out of the way.
    await page.mouse.move(2, 2);
    await openFromTheKeyboard();
    const scrolled = await panel.evaluate((element) => element.scrollTop);
    const at = await centre(yes);
    await page.keyboard.press('Escape');
    await expect(question).toHaveCount(0);

    // The mouse rests there, over the panel, and the person opens the question from the keyboard.
    await panel.evaluate((element, top) => { element.scrollTop = top; }, start);
    await page.mouse.move(at.x, at.y);
    await openFromTheKeyboard();
    expect(await panel.evaluate((element) => element.scrollTop)).toBe(scrolled);
    // The scroll moved the question by more than the yes is tall: the yes came under the pointer by it.
    expect(scrolled - start).toBeGreaterThan((await yes.boundingBox())!.height);
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(700);
    await pressHere(page);
    await page.waitForTimeout(150);
    await expect(question).toBeVisible();
    await expect(panel.locator('.spelling-checked')).toHaveCount(0);
    await expect(question.getByRole('button', { name: 'Cancel' })).toBeFocused();

    // Off the yes and back onto it, and a press after the half second.
    await page.waitForTimeout(600);
    const sentence = await centre(question.locator('p'));
    await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
    await page.mouse.move(at.x, at.y, { steps: 4 });
    await pressHere(page);
    await expect(panel.locator('.spelling-checked')).toContainText('Intern will use \u201cContoso\u201d on its own from now on.');
  });
}

/*
  The way it is meant to be used: press the action, read the sentence the
  press lands on, move the pointer onto the yes, press it. Taken on the
  first press, in the review panel and in Settings. In the review panel,
  as it first opens at 1400x900, the question opens with its buttons under
  the decision, and the person scrolls the panel with the wheel to reach
  them: the scroll moves the panel under a pointer that stays put, and the
  move onto the yes after it is what counts.
*/
test('the yes moved to after reading is taken on the first press, in the review panel and in Settings', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  await page.goto('/?learnedSpellings=held');
  const panel = await openDetails(page, 'SOW final v3.pdf');
  let question = panel.getByRole('group', { name: CONTOSO });
  let yes = question.getByRole('button', { name: 'Use on its own' });
  let at = await centre(panel.getByRole('button', { name: CONTOSO }));
  await page.mouse.click(at.x, at.y);
  expect(await under(page, at)).toBe(`the sentence of ${CONTOSO}`);
  let to = await centre(yes);
  expect(await under(page, to), 'the yes opens under the decision').not.toBe(`the yes of ${CONTOSO}`);
  await page.mouse.wheel(0, 200);
  await expect.poll(async () => under(page, await centre(yes))).toBe(`the yes of ${CONTOSO}`);
  await page.waitForTimeout(700);
  to = await centre(yes);
  await page.mouse.move(to.x, to.y, { steps: 6 });
  await pressHere(page);
  await expect(panel.locator('.spelling-checked')).toContainText('Intern will use \u201cContoso\u201d on its own from now on.');

  const dialog = await learnedSpellings(page);
  question = dialog.getByRole('group', { name: RIDGELINE });
  yes = question.getByRole('button', { name: 'Use on its own' });
  at = await centre(dialog.getByRole('button', { name: RIDGELINE }));
  await page.mouse.click(at.x, at.y);
  expect(await under(page, at)).toBe(`the sentence of ${RIDGELINE}`);
  await page.waitForTimeout(700);
  to = await centre(yes);
  expect(await under(page, to)).toBe(`the yes of ${RIDGELINE}`);
  await page.mouse.move(to.x, to.y, { steps: 6 });
  await pressHere(page);
  await expect(question).toHaveCount(0);
  await expect(dialog.locator('li[data-rule="2"] .rule-state')).toHaveText('In use');
});

/*
  A press on the sentence let go off the question clicks nothing in it,
  and leaves the focus on the row (Settings) or the group (the review
  drawer) around it. Escape there closed Settings, with what was not saved,
  or the drawer. It takes back the question, and nothing else.
*/
test('Escape after a press on the sentence let go off the question takes back only the question, in Settings', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  const dialog = await learnedSpellings(page);
  await dialog.locator('summary', { hasText: 'Naming and filing' }).click();
  const names = dialog.getByRole('textbox', { name: "Your organization's names" });
  await names.fill('Fabrikam LLP');
  const action = dialog.getByRole('button', { name: CONTOSO });
  await action.scrollIntoViewIfNeeded();
  const at = await centre(action);
  await page.mouse.click(at.x, at.y);
  const question = dialog.getByRole('group', { name: CONTOSO });
  const sentence = await centre(question.locator('p'));
  const row = (await dialog.locator('li[data-rule="1"] .rule-change').boundingBox())!;
  await page.mouse.move(sentence.x, sentence.y);
  await page.mouse.down();
  await page.mouse.move(row.x + 4, row.y + row.height / 2, { steps: 4 });
  await page.mouse.up();
  await expect(dialog.locator('li[data-rule="1"]')).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(question).toHaveCount(0);
  await expect(dialog).toBeVisible();
  await expect(action).toBeFocused();
  await expect(names).toHaveValue('Fabrikam LLP');
  // With no question open, Escape closes Settings as before.
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
});

test('Escape after a press on the sentence let go off the question takes back only the question, in the review drawer', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/?learnedSpellings=held');
  const panel = await openDetails(page, 'SOW final v3.pdf');
  await expect(panel).toHaveAttribute('role', 'dialog');
  const action = panel.getByRole('button', { name: CONTOSO });
  await action.scrollIntoViewIfNeeded();
  const at = await centre(action);
  await page.mouse.click(at.x, at.y);
  const question = panel.getByRole('group', { name: CONTOSO });
  const sentence = await centre(question.locator('p'));
  const note = (await panel.getByRole('note', { name: 'Learned spellings applied' }).boundingBox())!;
  await page.mouse.move(sentence.x, sentence.y);
  await page.mouse.down();
  await page.mouse.move(note.x + 10, note.y + note.height / 2, { steps: 4 });
  await page.mouse.up();
  await expect(panel.getByRole('group', { name: 'Use a learned spelling on its own' })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(question).toHaveCount(0);
  await expect(panel).toBeVisible();
  await expect(action).toBeFocused();
});
