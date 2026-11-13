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

/** Whether the note beside a question's buttons says `text` where it can be seen, its first line and its last: in the window, and not under the decision or anything else. */
const noteInSight = (question: Locator, text: string) => question.locator('.rule-confirm-again').evaluate((note, said) => {
  const box = note.getBoundingClientRect();
  const seen = (y: number) => { const hit = document.elementFromPoint(box.left + 12, y); return hit !== null && note.contains(hit); };
  return note.textContent === said && box.top >= 0 && box.bottom <= window.innerHeight && seen(box.top + 6) && seen(box.bottom - 6);
}, text);

const MOVE_OFF_AND_BACK = 'Move well off Use on its own, then back.';

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
    // It says what to do, where it can be seen, and the yes stays where it was.
    expect(await noteInSight(question, MOVE_OFF_AND_BACK)).toBe(true);
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);

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

/*
  What is under the pointer is what the browser finds there, not the yes's
  box. With the More menu open, the decision stuck to the bottom of the
  review panel grows over the question's yes. A press on Keep original,
  where it lies over the yes, closes the menu; the yes it uncovers is under
  the pointer, and a press again, because nothing seemed to happen, is no
  answer. A pointer seen on Keep original was not on the yes.
*/
/** Open the question in the review panel by a press on its action, its buttons brought above the decision with the wheel. */
async function questionAboveTheDecision(page: Page) {
  await page.goto('/?learnedSpellings=held');
  const panel = await openDetails(page, 'SOW final v3.pdf');
  const action = panel.getByRole('button', { name: CONTOSO });
  await action.scrollIntoViewIfNeeded();
  await panel.evaluate((element, label) => {
    const button = [...element.querySelectorAll('button')].find((candidate) => candidate.getAttribute('aria-label') === label)!;
    element.scrollTop += button.getBoundingClientRect().bottom + 8 - element.querySelector('.inspector-actions')!.getBoundingClientRect().top;
  }, CONTOSO);
  const opening = await centre(action);
  await page.mouse.click(opening.x, opening.y);
  const question = panel.getByRole('group', { name: CONTOSO });
  const yes = question.getByRole('button', { name: 'Use on its own' });
  await expect(question).toBeVisible();
  const box = (await yes.boundingBox())!;
  const decision = await panel.locator('.inspector-actions').evaluate((element) => element.getBoundingClientRect().top);
  if (box.y + box.height + 4 > decision) {
    await page.mouse.wheel(0, Math.ceil(box.y + box.height + 4 - decision));
    await expect.poll(async () => (await yes.boundingBox())!.y + (await yes.boundingBox())!.height).toBeLessThanOrEqual(decision);
  }
  return { panel, question, yes };
}

/** A point on `a` that lies over `b`'s box, when there is one. */
const over = (a: { x: number; y: number; width: number; height: number }, b: { x: number; y: number; width: number; height: number }) => {
  const x0 = Math.max(a.x, b.x) + 2, x1 = Math.min(a.x + a.width, b.x + b.width) - 2;
  const y0 = Math.max(a.y, b.y) + 2, y1 = Math.min(a.y + a.height, b.y + b.height) - 2;
  return x0 < x1 && y0 < y1 ? { x: (x0 + x1) / 2, y: (y0 + y1) / 2 } : undefined;
};

for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`a press on Keep original over the yes, and a press again on the yes it uncovered, check nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const { panel, question, yes } = await questionAboveTheDecision(page);
    const more = await centre(panel.getByRole('button', { name: 'More review actions' }));
    await page.mouse.move(more.x, more.y, { steps: 6 });
    await page.mouse.click(more.x, more.y);
    const menu = panel.getByRole('group', { name: 'More review actions' });
    await expect(menu).toBeVisible();
    const keep = panel.getByRole('button', { name: /Keep original/ });
    const at = over((await keep.boundingBox())!, (await yes.boundingBox())!)!;
    expect(at, 'Keep original lies over the yes').toBeDefined();
    await page.mouse.move(at.x, at.y, { steps: 8 });
    expect(await under(page, at)).not.toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(300);
    await pressHere(page);
    await expect(menu).toHaveCount(0);
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(700);
    await pressHere(page);
    await page.waitForTimeout(250);
    await expect(question).toBeVisible();
    await expect(panel.locator('.spelling-checked')).toHaveCount(0);
  });
}

/*
  Settings opened over the review panel, its backdrop over the question's
  yes. A press there does nothing; Escape closes Settings, and the yes is
  under a pointer that never moved onto it.
*/
for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`a press where Settings lay over the yes, after Settings closed, checks nothing, at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await page.goto('/?learnedSpellings=held');
    const panel = await openDetails(page, 'SOW final v3.pdf');
    const action = panel.getByRole('button', { name: CONTOSO });
    await action.scrollIntoViewIfNeeded();
    await panel.evaluate((element, label) => {
      const button = [...element.querySelectorAll('button')].find((candidate) => candidate.getAttribute('aria-label') === label)!;
      element.scrollTop += button.getBoundingClientRect().top - element.getBoundingClientRect().top - 80;
    }, CONTOSO);
    const opening = await centre(action);
    await page.mouse.click(opening.x, opening.y);
    const question = panel.getByRole('group', { name: CONTOSO });
    const at = await centre(question.getByRole('button', { name: 'Use on its own' }));
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.getByRole('button', { name: 'Settings' }).evaluate((button: HTMLElement) => button.click());
    const dialog = page.getByRole('dialog', { name: 'Settings' });
    await expect(dialog).toBeVisible();
    await page.mouse.move(at.x, at.y, { steps: 10 });
    expect(await under(page, at)).not.toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(400);
    await pressHere(page);
    await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(700);
    await pressHere(page);
    await page.waitForTimeout(250);
    await expect(question).toBeVisible();
    await expect(panel.locator('.spelling-checked')).toHaveCount(0);
  });
}

/*
  A press with another button on a yes the pointer did not move to clicks
  nothing, so nothing refused it, and the focus Chromium gives a pressed
  button stayed on the yes: Enter or Space after it was a yes from the
  keyboard. A press never gives the yes the focus.
*/
for (const surface of ['review panel', 'Settings'] as const) {
  test(`a right press on the yes, and Enter, check nothing, in the ${surface}`, async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    const host = surface === 'Settings' ? await learnedSpellings(page) : await (async () => { await page.goto('/?learnedSpellings=held'); return openDetails(page, 'SOW final v3.pdf'); })();
    const action = host.getByRole('button', { name: CONTOSO });
    await action.scrollIntoViewIfNeeded();
    const scroller = surface === 'Settings' ? host.locator('.dialog-body') : host;
    const top = await scroller.evaluate((element) => element.scrollTop);
    const question = host.getByRole('group', { name: CONTOSO });
    const yes = question.getByRole('button', { name: 'Use on its own' });
    const cancel = question.getByRole('button', { name: 'Cancel' });
    // Where the yes ends up when the question is opened from the keyboard; then the mouse rests there.
    await action.focus();
    await page.keyboard.press('Enter');
    const at = await centre(yes);
    await page.keyboard.press('Escape');
    await scroller.evaluate((element, scrollTop) => { element.scrollTop = scrollTop; }, top);
    await page.mouse.move(at.x, at.y);
    await action.focus();
    await page.keyboard.press('Enter');
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(700);
    for (const button of ['right', 'middle'] as const) {
      await page.mouse.down({ button }); await page.waitForTimeout(80); await page.mouse.up({ button });
      await expect(cancel).toBeFocused();
    }
    await page.keyboard.press('Enter');
    await expect(question).toHaveCount(0);
    if (surface === 'Settings') expect((await states(host))[0]).toBe('In use · asks before filing');
    else await expect(host.locator('.spelling-checked')).toHaveCount(0);
  });
}

/*
  The pointer came onto the yes, and the yes moved under it and back - the
  wheel rolled away and back. A pointer the yes left is not on it, and the
  yes that came back under it came by itself. Moved off it and back, it is.
*/
test('a press after the yes scrolled away from the pointer on it and back checks nothing until the pointer comes onto it again, in Settings', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  const dialog = await learnedSpellings(page);
  const opening = await centre(dialog.getByRole('button', { name: CONTOSO }));
  await page.mouse.click(opening.x, opening.y);
  const question = dialog.getByRole('group', { name: CONTOSO });
  const yes = question.getByRole('button', { name: 'Use on its own' });
  const sentence = await centre(question.locator('p'));
  const at = await centre(yes);
  await page.mouse.move(sentence.x, sentence.y, { steps: 3 });
  await page.mouse.move(at.x, at.y, { steps: 3 });
  expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
  await page.mouse.wheel(0, -300);
  await expect.poll(() => under(page, at)).not.toBe(`the yes of ${CONTOSO}`);
  await page.waitForTimeout(300);
  await page.mouse.wheel(0, 300);
  await expect.poll(() => under(page, at)).toBe(`the yes of ${CONTOSO}`);
  await page.waitForTimeout(700);
  await pressHere(page);
  await page.waitForTimeout(250);
  await expect(question).toBeVisible();
  expect((await states(dialog))[0]).toBe('In use · asks before filing');
  await page.waitForTimeout(600);
  await page.mouse.move(sentence.x, sentence.y, { steps: 3 });
  await page.mouse.move(at.x, at.y, { steps: 3 });
  await pressHere(page);
  await expect(question).toHaveCount(0);
  expect((await states(dialog))[0]).toBe('In use');
});

/*
  A scroll that leaves the yes's edge a pixel from a still mouse, and a
  twitch of the hand across it. The mouse moved, but onto a yes that came
  to it: it has to be seen where it is with the yes where it is now first.
*/
test('a twitch across the edge of a yes that scrolled beside a still mouse checks nothing, in the review panel', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  await page.goto('/?learnedSpellings=held');
  const panel = await openDetails(page, 'SOW final v3.pdf');
  const action = panel.getByRole('button', { name: CONTOSO });
  const question = panel.getByRole('group', { name: CONTOSO });
  const yes = question.getByRole('button', { name: 'Use on its own' });
  await panel.evaluate((element, label) => {
    const button = [...element.querySelectorAll('button')].find((candidate) => candidate.getAttribute('aria-label') === label)!;
    element.scrollTop += button.getBoundingClientRect().top - element.getBoundingClientRect().top - 200;
  }, CONTOSO);
  const opening = await centre(action);
  await page.mouse.move(opening.x, opening.y, { steps: 3 });
  await page.mouse.down(); await page.mouse.up();
  await expect(question).toBeVisible();
  const box = (await yes.boundingBox())!;
  const rest = { x: box.x + box.width / 2, y: box.y + box.height + 60 };
  await page.mouse.move(rest.x, rest.y, { steps: 5 });
  await page.mouse.wheel(0, -40);
  await page.waitForTimeout(400);
  // Where the wheel stopped, its top a pixel below the mouse.
  await panel.evaluate((element, y) => { element.scrollTop -= y + 1 - element.querySelector('.rule-confirm button.primary')!.getBoundingClientRect().top; }, rest.y);
  await page.waitForTimeout(700);
  const twitch = { x: rest.x, y: rest.y + 2 };
  await page.mouse.move(twitch.x, twitch.y);
  expect(await under(page, twitch)).toBe(`the yes of ${CONTOSO}`);
  await pressHere(page);
  await page.waitForTimeout(250);
  await expect(question).toBeVisible();
  await expect(panel.locator('.spelling-checked')).toHaveCount(0);
});

/** Open the review panel, or Settings, with Contoso's action in sight and clear of the review panel's decision. */
async function host(page: Page, surface: 'review panel' | 'Settings') {
  if (surface === 'Settings') return learnedSpellings(page);
  await page.goto('/?learnedSpellings=held');
  const panel = await openDetails(page, 'SOW final v3.pdf');
  await panel.getByRole('button', { name: CONTOSO }).scrollIntoViewIfNeeded();
  await panel.evaluate((element, label) => {
    const button = [...element.querySelectorAll('button')].find((candidate) => candidate.getAttribute('aria-label') === label)!;
    element.scrollTop += button.getBoundingClientRect().top - element.getBoundingClientRect().top - 80;
  }, CONTOSO);
  return panel;
}

/** Whether a press checked Contoso's spelling. */
const checked = async (root: Locator, surface: 'review panel' | 'Settings') => surface === 'Settings'
  ? (await states(root))[0] === 'In use'
  : (await root.locator('.spelling-checked').count()) > 0;

/*
  The page sees the mouse only while it is over the page with nothing of
  the system's over it. Moved over another window lying over the panel -
  the document the spelling is checked against - or over the system's menu,
  or with Intern behind another window, it moves where the page cannot see;
  when that goes away, the mouse rests where the page did not see it go, on
  the yes. Where it is next seen is where it was found, not a move onto the
  yes: a press there checks nothing until the mouse is moved away and back.
*/
const losses: [string, (page: Page, beyond: Point) => Promise<void>][] = [
  ['the mouse left the page', async (page, beyond) => { await page.mouse.move(beyond.x, beyond.y); }],
  ['Intern lost the focus', async (page) => { await page.evaluate(() => window.dispatchEvent(new FocusEvent('blur'))); }],
  ['the system\'s menu opened', async (page) => { await page.mouse.down({ button: 'right' }); await page.mouse.up({ button: 'right' }); }],
];
for (const surface of ['review panel', 'Settings'] as const) {
  for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
    test(`a mouse found on the yes after the page lost sight of it checks nothing until it is moved away and back, in the ${surface} at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const root = await host(page, surface);
      const opening = await centre(root.getByRole('button', { name: CONTOSO }));
      await page.mouse.move(opening.x, opening.y, { steps: 4 });
      await pressHere(page);
      const question = root.getByRole('group', { name: CONTOSO });
      await expect(question).toBeVisible();
      const at = await centre(question.getByRole('button', { name: 'Use on its own' }));
      expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
      const sentence = await centre(question.locator('p'));
      for (const [how, lose] of losses) {
        await page.mouse.move(sentence.x, sentence.y, { steps: 5 });
        await lose(page, { x: size.width + 40, y: sentence.y });
        await page.waitForTimeout(300);
        // Found on the yes: one report there, then a press.
        await page.mouse.move(at.x, at.y);
        await page.waitForTimeout(700);
        await pressHere(page);
        await page.waitForTimeout(250);
        await expect(question, how).toBeVisible();
        expect(await noteInSight(question, MOVE_OFF_AND_BACK), how).toBe(true);
        expect(await checked(root, surface), how).toBe(false);
      }
      await page.waitForTimeout(600);
      await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
      await page.mouse.move(at.x, at.y, { steps: 4 });
      await pressHere(page);
      await expect.poll(() => checked(root, surface)).toBe(true);
    });
  }
}

/*
  A question opened from the keyboard, needing no scroll, its yes appearing
  with its top edge a pixel or two below a resting mouse, and a press with a
  hand's drift: one report two or three pixels down, onto the yes, then the
  press. The mouse moved onto the yes, but the yes came to it. A pointer
  comes onto the yes from away from it - at least the yes's height off it -
  with the yes where it is.
*/
for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
  test(`a mouse that drifts onto a yes that appeared beside it checks nothing, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const dialog = await learnedSpellings(page);
    const action = dialog.getByRole('button', { name: CONTOSO });
    const question = dialog.getByRole('group', { name: CONTOSO });
    const yes = question.getByRole('button', { name: 'Use on its own' });
    const body = dialog.locator('.dialog-body');
    const top = await body.evaluate((element) => element.scrollTop);
    const openFromTheKeyboard = async () => {
      await action.evaluate((button) => button.focus({ preventScroll: true }));
      await page.keyboard.press('Enter');
      await expect(question.getByRole('button', { name: 'Cancel' })).toBeFocused();
    };
    await page.mouse.move(2, 2);
    await openFromTheKeyboard();
    expect(await body.evaluate((element) => element.scrollTop), 'no scroll was needed').toBe(top);
    const box = (await yes.boundingBox())!;
    await page.keyboard.press('Escape');
    await expect(question).toHaveCount(0);
    for (const gap of [1, 2]) {
      const rest = { x: box.x + box.width / 2, y: box.y - gap };
      await page.mouse.move(rest.x, rest.y, { steps: 5 });
      await openFromTheKeyboard();
      const drifted = { x: rest.x, y: rest.y + gap + 1 };
      expect(await under(page, rest)).not.toBe(`the yes of ${CONTOSO}`);
      expect(await under(page, drifted)).toBe(`the yes of ${CONTOSO}`);
      await page.waitForTimeout(700);
      await page.mouse.move(drifted.x, drifted.y);
      await pressHere(page);
      await page.waitForTimeout(250);
      await expect(question).toBeVisible();
      expect(await noteInSight(question, MOVE_OFF_AND_BACK)).toBe(true);
      expect((await states(dialog))[0]).toBe('In use · asks before filing');
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);
    }
    // Moved onto it from away, it is answered.
    await openFromTheKeyboard();
    await page.waitForTimeout(600);
    const sentence = await centre(question.locator('p'));
    await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, { steps: 4 });
    await pressHere(page);
    await expect(question).toHaveCount(0);
    expect((await states(dialog))[0]).toBe('In use');
  });
}

/*
  The wheel brings the yes's top edge to rest a pixel or two below a still
  mouse, and the press that follows jitters: a pixel sideways, still off
  the yes, then two or three down, onto it. The mouse was seen off the yes
  where it is now, and moved onto it, but from beside it.
*/
for (const surface of ['review panel', 'Settings'] as const) {
  for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
    test(`a jittery press onto a yes the wheel brought beside a still mouse checks nothing, in the ${surface} at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const root = await host(page, surface);
      const opening = await centre(root.getByRole('button', { name: CONTOSO }));
      await page.mouse.move(opening.x, opening.y, { steps: 4 });
      await pressHere(page);
      const question = root.getByRole('group', { name: CONTOSO });
      const yes = question.getByRole('button', { name: 'Use on its own' });
      await expect(question).toBeVisible();
      await page.waitForTimeout(700);
      for (const gap of [1, 2]) {
        const box = (await yes.boundingBox())!;
        const rest = { x: box.x + box.width / 2, y: box.y - 40 };
        await page.mouse.move(rest.x, rest.y, { steps: 6 });
        await page.mouse.wheel(0, 40 - gap);
        await expect.poll(async () => (await yes.boundingBox())!.y - rest.y).toBe(gap);
        await page.waitForTimeout(300);
        const onto = { x: rest.x + 1, y: rest.y + gap + 2 };
        await page.mouse.move(rest.x + 1, rest.y);
        await page.mouse.move(onto.x, onto.y);
        expect(await under(page, onto)).toBe(`the yes of ${CONTOSO}`);
        await pressHere(page);
        await page.waitForTimeout(250);
        await expect(question).toBeVisible();
        expect(await noteInSight(question, MOVE_OFF_AND_BACK)).toBe(true);
        expect(await checked(root, surface)).toBe(false);
        // The wheel back, for the next.
        await page.mouse.wheel(0, gap - 40);
        await page.waitForTimeout(300);
      }
      await page.waitForTimeout(600);
      const sentence = await centre(question.locator('p'));
      const at = await centre(yes);
      await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
      await page.mouse.move(at.x, at.y, { steps: 4 });
      await pressHere(page);
      await expect.poll(() => checked(root, surface)).toBe(true);
    });
  }
}

/*
  A touch screen: a finger has no hover, and comes onto the yes by
  touching down on it, from where it last touched off it. Each pointer is
  its own: a finger's tap says nothing of where the mouse is.
*/
test.describe('with a touch screen', () => {
  test.use({ hasTouch: true });

  test('a tap on Keep original over the yes checks nothing, though its click lands on the yes it uncovers', async ({ page }) => {
    await page.setViewportSize({ width: 1400, height: 900 });
    const { panel, question, yes } = await questionAboveTheDecision(page);
    await page.waitForTimeout(600);
    const more = await centre(panel.getByRole('button', { name: 'More review actions' }));
    await page.touchscreen.tap(more.x, more.y);
    await expect(panel.getByRole('group', { name: 'More review actions' })).toBeVisible();
    const keep = panel.getByRole('button', { name: /Keep original/ });
    const at = over((await keep.boundingBox())!, (await yes.boundingBox())!)!;
    expect(at, 'Keep original lies over the yes').toBeDefined();
    await page.waitForTimeout(400);
    await page.touchscreen.tap(at.x, at.y);
    await page.waitForTimeout(300);
    await expect(question).toBeVisible();
    await expect(panel.locator('.spelling-checked')).toHaveCount(0);
  });

  for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
    test(`a finger opens the question with its yes under the resting mouse; a pixel's move of the mouse and a press check nothing, in Settings at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const dialog = await learnedSpellings(page);
      const question = dialog.getByRole('group', { name: CONTOSO });
      const yes = question.getByRole('button', { name: 'Use on its own' });
      const tap = await centre(dialog.getByRole('button', { name: CONTOSO }));
      await page.touchscreen.tap(tap.x, tap.y);
      const at = await centre(yes);
      await page.keyboard.press('Escape');
      await expect(question).toHaveCount(0);
      await page.mouse.move(at.x, at.y, { steps: 5 });
      await page.touchscreen.tap(tap.x, tap.y);
      expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
      await page.waitForTimeout(600);
      await page.mouse.move(at.x + 1, at.y);
      await pressHere(page);
      await page.waitForTimeout(250);
      await expect(question).toBeVisible();
      expect((await states(dialog))[0]).toBe('In use · asks before filing');
      // A tap on it is a finger coming onto it, from where it last touched.
      await page.waitForTimeout(600);
      await page.touchscreen.tap(at.x, at.y);
      await expect(question).toHaveCount(0);
      expect((await states(dialog))[0]).toBe('In use');
    });
  }

  /*
    A finger that last touched beside the yes - on the question, a few
    pixels above it - and taps it is refused, and the note says what to do,
    in two lines where it can be seen. Done, the tap is answered.
  */
  for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
    test(`a finger refused is told what to do, and is answered once it is done, in the review panel at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const panel = await host(page, 'review panel');
      const tap = await centre(panel.getByRole('button', { name: CONTOSO }));
      await page.touchscreen.tap(tap.x, tap.y);
      const question = panel.getByRole('group', { name: CONTOSO });
      const yes = question.getByRole('button', { name: 'Use on its own' });
      await expect(question).toBeVisible();
      await page.waitForTimeout(700);
      const box = (await yes.boundingBox())!;
      const on = { x: box.x + 14, y: box.y + box.height / 2 };
      expect(await under(page, on)).toBe(`the yes of ${CONTOSO}`);
      await page.touchscreen.tap(box.x + 12, box.y - 3);
      await page.waitForTimeout(600);
      await page.touchscreen.tap(on.x, on.y);
      await page.waitForTimeout(250);
      await expect(question).toBeVisible();
      expect(await noteInSight(question, 'Tap well away, then Use on its own.')).toBe(true);
      const note = question.locator('.rule-confirm-again');
      expect(await note.evaluate((element) => Math.round(element.getBoundingClientRect().height / parseFloat(getComputedStyle(element).lineHeight)))).toBeLessThanOrEqual(2);
      await expect(panel.locator('.spelling-checked')).toHaveCount(0);
      await page.waitForTimeout(600);
      const sentence = (await question.locator('p').boundingBox())!;
      await page.touchscreen.tap(sentence.x + 6, sentence.y + 6);
      await page.waitForTimeout(300);
      await page.touchscreen.tap(on.x, on.y);
      await expect(panel.locator('.spelling-checked')).toContainText('Intern will use \u201cContoso\u201d on its own from now on.');
    });
  }

  /*
    A press with the middle button on the yes clicks nothing, and it left
    the press on the yes standing, under the mouse that had come onto it
    and rested there. A finger tapped Keep original where it lay over the
    yes: its press closed the More menu, and its click landed on the yes,
    taken as the end of the mouse's press. A click on the yes ends only a
    press that began on the yes.
  */
  for (const size of [{ width: 1400, height: 900 }, { width: 800, height: 600 }]) {
    test(`a finger's tap on Keep original over the yes, after a middle press on the yes, checks nothing, at ${size.width}x${size.height}`, async ({ page }) => {
      await page.setViewportSize(size);
      const { panel, question, yes } = await questionAboveTheDecision(page);
      await page.waitForTimeout(700);
      const box = (await yes.boundingBox())!;
      const more = await centre(panel.getByRole('button', { name: 'More review actions' }));
      const menu = panel.getByRole('group', { name: 'More review actions' });
      await page.touchscreen.tap(more.x, more.y);
      await expect(menu).toBeVisible();
      const at = over((await panel.getByRole('button', { name: /Keep original/ }).boundingBox())!, box)!;
      expect(at, 'Keep original lies over the yes').toBeDefined();
      await page.keyboard.press('Escape');
      await expect(menu).toHaveCount(0);
      // The mouse comes onto the yes from the sentence, beside where the finger will tap, and rests.
      const sentence = await centre(question.locator('p'));
      const rest = { x: Math.min(box.x + box.width - 6, at.x + 20), y: box.y + box.height / 2 };
      await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
      await page.mouse.move(rest.x, rest.y, { steps: 6 });
      expect(await under(page, rest)).toBe(`the yes of ${CONTOSO}`);
      await page.mouse.down({ button: 'middle' }); await page.waitForTimeout(80); await page.mouse.up({ button: 'middle' });
      await page.waitForTimeout(300);
      await page.touchscreen.tap(more.x, more.y);
      await expect(menu).toBeVisible();
      expect(await under(page, at)).not.toBe(`the yes of ${CONTOSO}`);
      await page.waitForTimeout(600);
      await page.touchscreen.tap(at.x, at.y);
      await page.waitForTimeout(300);
      await expect(menu).toHaveCount(0);
      await expect(question).toBeVisible();
      await expect(panel.locator('.spelling-checked')).toHaveCount(0);
    });
  }
});

/*
  The way it is meant to be used, where the person's own wheel brings the
  yes to rest under the pointer that pressed the action. The yes came to
  the pointer, and is refused; that refusal said nothing, and a person who
  meant it pressed again and again a yes that did nothing. It says what to
  do, beside the buttons, and doing it is answered.
*/
for (const [surface, size] of [['review panel', { width: 1400, height: 900 }], ['review panel', { width: 800, height: 600 }], ['Settings', { width: 800, height: 600 }]] as const) {
  test(`a yes the wheel brought under the pointer says what to do, and is answered once it is done, in the ${surface} at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    const host = surface === 'Settings' ? await learnedSpellings(page) : await (async () => { await page.goto('/?learnedSpellings=held'); return openDetails(page, 'SOW final v3.pdf'); })();
    const action = host.getByRole('button', { name: CONTOSO });
    await action.scrollIntoViewIfNeeded();
    const at = await centre(action);
    await page.mouse.move(at.x - 40, at.y - 30);
    await page.mouse.move(at.x, at.y, { steps: 4 });
    await pressHere(page);
    const question = host.getByRole('group', { name: CONTOSO });
    const yes = question.getByRole('button', { name: 'Use on its own' });
    await expect(question).toBeVisible();
    await page.waitForTimeout(700);
    const below = await centre(yes);
    await page.mouse.wheel(0, Math.round(below.y - at.y));
    await expect.poll(() => under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(300);
    await pressHere(page);
    await page.waitForTimeout(150);
    await expect(question).toBeVisible();
    expect(await noteInSight(question, MOVE_OFF_AND_BACK)).toBe(true);
    expect(await under(page, at)).toBe(`the yes of ${CONTOSO}`);
    await page.waitForTimeout(600);
    const sentence = await centre(question.locator('p'));
    await page.mouse.move(sentence.x, sentence.y, { steps: 4 });
    await page.mouse.move(at.x, at.y, { steps: 4 });
    await pressHere(page);
    if (surface === 'Settings') {
      await expect(question).toHaveCount(0);
      expect((await states(host))[0]).toBe('In use');
    } else await expect(host.locator('.spelling-checked')).toContainText('Intern will use \u201cContoso\u201d on its own from now on.');
  });
}

/*
  A person quick to move onto the yes and press it, half a second after
  the press that opened the question began - as a double-click is timed,
  from its first press to its second - is answered on that press.
*/
test('a press on the yes half a second after the press that opened the question began is answered', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 900 });
  const dialog = await learnedSpellings(page);
  const at = await centre(dialog.getByRole('button', { name: RIDGELINE }));
  const question = dialog.getByRole('group', { name: RIDGELINE });
  const yes = question.getByRole('button', { name: 'Use on its own' });
  await page.mouse.move(at.x, at.y);
  const began = Date.now();
  await page.mouse.down(); await page.waitForTimeout(200); await page.mouse.up();
  await expect(question).toBeVisible();
  const to = await centre(yes);
  await page.mouse.move(to.x, to.y, { steps: 5 });
  await page.waitForTimeout(Math.max(0, began + 600 - Date.now()));
  await page.mouse.down(); await page.waitForTimeout(150); await page.mouse.up();
  await expect(question).toHaveCount(0);
  await expect(dialog.locator('li[data-rule="2"] .rule-state')).toHaveText('In use');
});
