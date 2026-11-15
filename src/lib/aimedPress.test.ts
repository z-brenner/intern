import { fireEvent } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AGAIN_MS, REACH_PX, SETTLE_MS, createAim, firstPress, takeOnlyAimedPresses } from './aimedPress';
import type { Box } from './aimedPress';

/*
  A press of the pointer acts only on a control that has been under the
  place pressed for the settle time, or that the person asked for and went
  to. The rule is tried here with a clock of its own: every time below is
  one the test names, and no timer runs.
*/

const box = (left: number, top: number, width = 100, height = 30): Box => ({ left, top, right: left + width, bottom: top + height });

describe('a press on a control that acts', () => {
  // The places a press is made at: the middle of a button at (500, 400), and of one at (500, 300).
  const HERE = [550, 415] as const;
  const ABOVE = [550, 315] as const;

  it('is taken at once on a control that has not moved, however soon after another did', () => {
    const aim = createAim();
    const approve = {};
    const other = {};
    aim.seen([[approve, box(500, 400)], [other, box(100, 100)]], 0);
    // Something else moves by itself, a moment before the press.
    aim.seen([[approve, box(500, 400)], [other, box(100, 160)]], 10_000);

    expect(aim.refuses(approve, ...HERE, 10_001)).toBe(false);
  });

  it('is refused on a control that has just moved under the pointer, and taken once it has been there for the settle time', () => {
    const aim = createAim();
    const approve = {};
    aim.seen([[approve, box(500, 300)]], 0);
    aim.pointerAt(...HERE);
    aim.seen([[approve, box(500, 400)]], 10_000);

    expect(aim.refuses(approve, ...HERE, 10_000)).toBe(true);
    expect(aim.refuses(approve, ...HERE, 10_250)).toBe(true);
    expect(aim.refuses(approve, ...HERE, 10_000 + SETTLE_MS - 1)).toBe(true);
    expect(aim.refuses(approve, ...HERE, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is refused on a control that has just appeared, and on one that went and came back', () => {
    const aim = createAim();
    const approve = {};
    const discard = {};
    aim.seen([[approve, box(500, 400)]], 0);
    // "Discard what I had" appears by itself where the name was.
    aim.seen([[approve, box(500, 400)], [discard, box(500, 300)]], 10_000);
    expect(aim.refuses(discard, ...ABOVE, 10_250)).toBe(true);
    expect(aim.refuses(discard, ...ABOVE, 10_000 + SETTLE_MS)).toBe(false);

    // Gone, and back in the very same place a long while later: it has appeared again.
    aim.seen([[approve, box(500, 400)]], 20_000);
    aim.seen([[approve, box(500, 400)], [discard, box(500, 300)]], 30_000);
    expect(aim.refuses(discard, ...ABOVE, 30_250)).toBe(true);
    expect(aim.refuses(discard, ...ABOVE, 30_000 + SETTLE_MS)).toBe(false);
    // One never seen at all has appeared with the press itself.
    expect(aim.refuses({}, ...HERE, 40_000)).toBe(true);
  });

  it('is refused on a control that moved by itself though the pointer has since moved onto it: a hand already on its way moves too', () => {
    const aim = createAim();
    const renameAll = {};
    aim.seen([[renameAll, box(100, 400)]], 0);
    aim.pointerAt(900, 50);
    // By itself: nobody's press or key was being handled.
    aim.seen([[renameAll, box(500, 400)]], 10_000);
    aim.pointerAt(700, 200);
    aim.pointerAt(...HERE);

    expect(aim.refuses(renameAll, ...HERE, 10_250)).toBe(true);
    expect(aim.refuses(renameAll, ...HERE, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is taken on a control the person asked for, as soon as the pointer has gone to it: an item of a menu just opened', () => {
    const aim = createAim();
    const more = {};
    const analyzeAgain = {};
    aim.seen([[more, box(700, 400, 30)]], 0);
    // The press on More, and the menu it opens in the same turn, away from the pointer.
    aim.pointerAt(715, 415);
    aim.seen([[more, box(700, 400, 30)], [analyzeAgain, box(500, 300)]], 10_000, true);

    expect(aim.refuses(analyzeAgain, ...ABOVE, 10_090)).toBe(false);
  });

  it('is taken on an item that opened under the pointer once the pointer has gone along it, and not where it rests', () => {
    const aim = createAim();
    const more = {};
    const remove = {};
    aim.seen([[more, box(700, 400, 30)]], 0);
    // The press on More, at its middle; the menu's last item opens where More was, the width of the panel.
    aim.pointerAt(715, 415);
    aim.seen([[more, box(700, 350, 30)], [remove, box(400, 400, 330)]], 10_000, true);

    expect(aim.refuses(remove, 715, 415, 10_300)).toBe(true);
    expect(aim.refuses(remove, 715 - (REACH_PX - 1), 415, 10_300)).toBe(true);
    expect(aim.refuses(remove, 715 - REACH_PX, 415, 10_300)).toBe(false);
    expect(aim.refuses(remove, 450, 415, 10_300)).toBe(false);
    // Opened by itself there, it is taken nowhere on it until it has been there for the settle time.
    const byItself = createAim();
    byItself.pointerAt(715, 415);
    byItself.seen([[remove, box(400, 400, 330)]], 10_000);
    expect(byItself.refuses(remove, 450, 415, 10_300)).toBe(true);
    expect(byItself.refuses(remove, 450, 415, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is refused on a control the person\'s own press brought under the pointer: the second click of a double-click lands there', () => {
    const aim = createAim();
    const discard = {};
    const cancelProcessing = {};
    aim.seen([[discard, box(500, 400)], [cancelProcessing, box(500, 520)]], 0);
    // The first click, on Discard; the strip goes, and Cancel processing comes up under the pointer.
    aim.pointerAt(...HERE);
    aim.seen([[cancelProcessing, box(500, 400)]], 10_000, true);

    expect(aim.refuses(cancelProcessing, ...HERE, 10_120)).toBe(true);
    // A pointer that trembles between the clicks has not gone anywhere.
    aim.pointerAt(HERE[0] + 2, HERE[1] + 1);
    expect(aim.refuses(cancelProcessing, HERE[0] + 2, HERE[1] + 1, 10_180)).toBe(true);
    expect(aim.refuses(cancelProcessing, ...HERE, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is judged by the settle time alone where no pointer has been seen: a first tap on what the keyboard just opened', () => {
    const aim = createAim();
    const remove = {};
    // Asked for with a key; no pointer has been anywhere yet.
    aim.seen([[remove, box(500, 400)]], 10_000, true);

    expect(aim.refuses(remove, ...HERE, 10_250)).toBe(true);
    expect(aim.refuses(remove, ...HERE, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is taken on the next document\'s button in the same place: approving one after another with the pointer at rest', () => {
    const aim = createAim();
    // Approve & rename, the same control for whichever document the panel shows.
    const approve = {};
    aim.seen([[approve, box(500, 400)]], 0);
    aim.pointerAt(...HERE);
    for (let press = 0; press < 5; press += 1) {
      const at = 10_000 + press * 200;
      expect(aim.refuses(approve, ...HERE, at)).toBe(false);
      // The decision is carried out: the button is disabled, goes while the
      // document just decided is shown as filed, and is back for the next one.
      aim.seen([[approve, box(500, 400), true, `document ${press}`]], at + 1, true);
      aim.seen([[{}, box(500, 440)]], at + 90);
      aim.seen([[approve, box(500, 400), false, `document ${press + 1}`]], at + 150);
    }
    expect(aim.refuses(approve, ...HERE, 11_000)).toBe(false);
  });

  it('is refused on a button that went while it was disabled and came back for the same document: read again by itself while something else was carried out', () => {
    const aim = createAim();
    const approve = {};
    aim.seen([[approve, box(500, 400), false, 'lease']], 0);
    aim.pointerAt(...HERE);
    // Another action of the person's has the buttons disabled; the open
    // document is read again by itself and is back with another name.
    aim.seen([[approve, box(500, 400), true, 'lease']], 10_000, true);
    aim.seen([], 10_120);
    aim.seen([[approve, box(500, 400), true, 'lease']], 10_330);
    aim.seen([[approve, box(500, 400), false, 'lease']], 10_400);

    expect(aim.refuses(approve, ...HERE, 10_443)).toBe(true);
    expect(aim.refuses(approve, ...HERE, 10_330 + SETTLE_MS)).toBe(false);

    // Back for another document, the same going is passed over: the person decided, and this is the next.
    const next = createAim();
    next.seen([[approve, box(500, 400), false, 'lease']], 0);
    next.pointerAt(...HERE);
    next.seen([[approve, box(500, 400), true, 'lease']], 10_000, true);
    next.seen([], 10_120);
    next.seen([[approve, box(500, 400), false, 'employment']], 10_330);
    expect(next.refuses(approve, ...HERE, 10_443)).toBe(false);

    // A button that says no document is never passed over: it cannot be told to be another's.
    const unsaid = createAim();
    unsaid.seen([[approve, box(500, 400)]], 0);
    unsaid.pointerAt(...HERE);
    unsaid.seen([[approve, box(500, 400), true]], 10_000, true);
    unsaid.seen([], 10_120);
    unsaid.seen([[approve, box(500, 400)]], 10_330);
    expect(unsaid.refuses(approve, ...HERE, 10_443)).toBe(true);
    // Nor one that said a document as it went and says none as it comes back, or the other way about.
    for (const [went, back] of [['lease', undefined], [undefined, 'lease']] as const) {
      const half = createAim();
      half.seen([[approve, box(500, 400), false, went]], 0);
      half.pointerAt(...HERE);
      half.seen([[approve, box(500, 400), true, went]], 10_000, true);
      half.seen([], 10_120);
      half.seen([[approve, box(500, 400), false, back]], 10_330);
      expect(half.refuses(approve, ...HERE, 10_443), `${went} then ${back}`).toBe(true);
    }
  });

  it('is refused on a button that went and came back by itself, or came back late, or somewhere else', () => {
    // Its document was read again by itself and is back with another reading: the button could be pressed when it went.
    const byItself = createAim();
    const approve = {};
    byItself.seen([[approve, box(500, 400)]], 0);
    byItself.pointerAt(...HERE);
    byItself.seen([], 10_000);
    byItself.seen([[approve, box(500, 400)]], 10_150);
    expect(byItself.refuses(approve, ...HERE, 10_160)).toBe(true);
    expect(byItself.refuses(approve, ...HERE, 10_150 + SETTLE_MS)).toBe(false);

    // In the middle of a decision, and back for the next document only after the settle time: it has appeared.
    const late = createAim();
    late.seen([[approve, box(500, 400), false, 'lease']], 0);
    late.pointerAt(...HERE);
    late.seen([[approve, box(500, 400), true, 'lease']], 10_000, true);
    late.seen([], 10_050);
    late.seen([[approve, box(500, 400), false, 'employment']], 10_050 + SETTLE_MS);
    expect(late.refuses(approve, ...HERE, 10_060 + SETTLE_MS)).toBe(true);
    expect(late.refuses(approve, ...HERE, 10_050 + 2 * SETTLE_MS)).toBe(false);

    // In the middle of a decision, and back at once for the next document in another place: under a pointer that was not on it before.
    const elsewhere = createAim();
    elsewhere.seen([[approve, box(500, 300), false, 'lease']], 0);
    elsewhere.pointerAt(...HERE);
    elsewhere.seen([[approve, box(500, 300), true, 'lease']], 10_000, true);
    elsewhere.seen([], 10_050);
    elsewhere.seen([[approve, box(500, 400), false, 'employment']], 10_150);
    expect(elsewhere.refuses(approve, ...HERE, 10_160)).toBe(true);
    expect(elsewhere.refuses(approve, ...HERE, 10_150 + SETTLE_MS)).toBe(false);
  });

  it('goes on being judged rightly however many documents are decided and however long each takes', () => {
    const aim = createAim();
    const approve = {};
    aim.seen([[approve, box(500, 400)]], 0);
    aim.pointerAt(...HERE);
    // Two hundred decisions, each a going of 80ms, a second apart.
    let at = 10_000;
    for (let decided = 0; decided < 200; decided += 1, at += 1000) {
      aim.seen([[approve, box(500, 400), true, `document ${decided}`]], at, true);
      aim.seen([], at + 20);
      aim.seen([[approve, box(500, 400), false, `document ${decided + 1}`]], at + 100);
      expect(aim.refuses(approve, ...HERE, at + 110)).toBe(false);
    }
    // One that took 400ms, and the page looked at again before the press: still where the person left it.
    aim.seen([[approve, box(500, 400), true, 'document 200']], at, true);
    aim.seen([], at + 20);
    aim.seen([[approve, box(500, 400), false, 'document 201']], at + 420);
    aim.seen([[approve, box(500, 400), false, 'document 201'], [{}, box(100, 100)]], at + 530);
    expect(aim.refuses(approve, ...HERE, at + 540)).toBe(false);
    at += 5000;
    // Then the document is read again by itself: gone for a moment while it could be pressed.
    aim.seen([], at);
    aim.seen([[approve, box(500, 400), false, 'document 201']], at + 100);
    expect(aim.refuses(approve, ...HERE, at + 110)).toBe(true);
  });

  it('is taken on a button that moved a little and is still under the place pressed, and refused where another place on it was pressed', () => {
    const aim = createAim();
    const approve = {};
    aim.seen([[approve, box(500, 400)]], 0);
    aim.pointerAt(...HERE);
    // The next document's panel is a line shorter: the button is 10px higher, and still under the pointer.
    aim.seen([[approve, box(500, 390)]], 10_000);

    expect(aim.refuses(approve, ...HERE, 10_100)).toBe(false);
    // Its top edge was not part of it before: a press there lands where something else was.
    expect(aim.refuses(approve, 550, 392, 10_100)).toBe(true);
    expect(aim.refuses(approve, 550, 392, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('is refused on a control that was covered, or not on the page, a moment ago', () => {
    const aim = createAim();
    const renameAll = {};
    // Under the drawer: drawn, and not there to be pressed.
    aim.seen([[renameAll, undefined]], 0);
    aim.seen([[renameAll, box(500, 400)]], 10_000);

    expect(aim.refuses(renameAll, ...HERE, 10_250)).toBe(true);
    expect(aim.refuses(renameAll, ...HERE, 10_000 + SETTLE_MS)).toBe(false);
  });

  it('counts from the last time the control came under the place pressed, through any number of moves', () => {
    const aim = createAim();
    const chip = {};
    aim.seen([[chip, box(500, 400)]], 0);
    // The panel is scrolled a little at a time: under the place pressed all the while.
    for (let step = 1; step <= 10; step += 1) aim.seen([[chip, box(500, 400 - step)]], 10_000 + step * 16);
    expect(aim.refuses(chip, ...HERE, 10_200)).toBe(false);

    // Scrolled away and back: it left the place pressed, and came back to it at 20,300.
    aim.seen([[chip, box(500, 200)]], 20_000);
    aim.seen([[chip, box(500, 395)]], 20_300);
    aim.seen([[chip, box(500, 400)]], 20_400);
    expect(aim.refuses(chip, ...HERE, 20_300 + SETTLE_MS - 1)).toBe(true);
    expect(aim.refuses(chip, ...HERE, 20_300 + SETTLE_MS)).toBe(false);
  });

  it('is never dead for longer than the settle time after the last change, as after a window is resized', () => {
    const aim = createAim();
    const approve = {};
    aim.seen([[approve, box(500, 400)]], 0);
    // Half a minute of a window being dragged wider and narrower.
    let at = 10_000;
    for (let step = 0; step < 1800; step += 1, at += 16) aim.seen([[approve, box(500 + (step % 7) * 40, 400 + (step % 5) * 40)]], at);
    const last = at - 16;
    const [x, y] = [500 + (1799 % 7) * 40 + 50, 400 + (1799 % 5) * 40 + 15];

    expect(aim.refuses(approve, x, y, last + 1)).toBe(true);
    expect(aim.refuses(approve, x, y, last + SETTLE_MS)).toBe(false);
  });
});

/*
  The same rule in a page. jsdom lays nothing out, so each control is given
  its box, and what is on top at a point is the last thing placed there.
*/
describe('a press in a window that takes only aimed presses', () => {
  let stop = () => {};
  let now = 0;
  const placed = new Map<Element, Box>();
  const put = (element: Element, at: Box | undefined) => { if (at) { placed.delete(element); placed.set(element, at); } else placed.delete(element); };
  const start = () => {
    now = 0;
    vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
      const at = placed.get(this);
      return (at ? { ...at, x: at.left, y: at.top, width: at.right - at.left, height: at.bottom - at.top } : { left: 0, top: 0, right: 0, bottom: 0, x: 0, y: 0, width: 0, height: 0 }) as DOMRect;
    });
    (document as { elementFromPoint?: unknown }).elementFromPoint = (x: number, y: number) => [...placed].reverse().find(([element, at]) => element.isConnected && x >= at.left && x < at.right && y >= at.top && y < at.bottom)?.[0] ?? null;
    stop = takeOnlyAimedPresses(window, () => now);
  };
  let hearing = new AbortController();
  afterEach(() => { stop(); hearing.abort(); hearing = new AbortController(); placed.clear(); delete (document as { elementFromPoint?: unknown }).elementFromPoint; vi.restoreAllMocks(); document.body.replaceChildren(); });

  /** A button on the page: one that acts unless told otherwise. */
  const button = (name: string, at: Box | undefined, acts = true) => {
    const control = document.createElement('button');
    control.textContent = name;
    if (acts) control.setAttribute('data-acts', 'true');
    document.body.append(control);
    put(control, at);
    return control;
  };
  /**
   * What the page under the window gets of a press: its own listeners, as
   * React's are, on the document. A button going down and a click, and with
   * `releases` the button let go again as well, which nothing acts on.
   */
  const heard = (releases = false) => {
    const events: string[] = [];
    for (const type of ['pointerdown', 'mousedown', ...(releases ? ['pointerup', 'mouseup'] : []), 'click', 'dblclick']) document.addEventListener(type, (event) => events.push(`${type}:${(event.target as Element).textContent}`), { signal: hearing.signal });
    return events;
  };
  /** A whole press of the mouse at a place, on whatever is on top there, as a browser sends it. */
  const press = (x: number, y: number, more: { pointerType?: string; detail?: number } = {}) => {
    const target = document.elementFromPoint(x, y) ?? document.body;
    const at = { clientX: x, clientY: y, button: 0, buttons: 1, pointerId: 1, isPrimary: true, pointerType: more.pointerType ?? 'mouse' };
    // A pointerdown that is refused sends no mouse events after it.
    const down = fireEvent.pointerDown(target, at);
    if (down) fireEvent.mouseDown(target, at);
    fireEvent.pointerUp(target, { ...at, buttons: 0 });
    if (down) fireEvent.mouseUp(target, { ...at, buttons: 0 });
    fireEvent.click(document.elementFromPoint(x, y) ?? document.body, { ...at, buttons: 0, detail: more.detail ?? 1 });
    return down;
  };
  /** The page changes; the window sees it as a browser would have it seen, before anything else happens. */
  const changed = async () => { await Promise.resolve(); };

  it('gives the page no press and no click on a control that has just come under the pointer, and the focus stays', async () => {
    start();
    const field = document.createElement('textarea');
    document.body.append(field);
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const events = heard(true);
    field.focus();
    now = 10_000;
    // By itself the button is now where Keep original was.
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();

    // Only the button let go again, which acts on nothing.
    now = 10_250;
    expect(press(550, 415)).toBe(false);
    expect(events).toEqual(['pointerup:Approve & rename']);
    expect(document.activeElement).toBe(field);
    events.length = 0;

    // After the settle time, and not at once after the refused press, the same press is an ordinary one.
    now = 10_250 + AGAIN_MS;
    expect(press(550, 415)).toBe(true);
    expect(events).toEqual(['pointerdown:Approve & rename', 'mousedown:Approve & rename', 'pointerup:Approve & rename', 'mouseup:Approve & rename', 'click:Approve & rename']);
  });

  it('does not take a press that went down too soon, though it is let go after the settle time', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const events = heard();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    const at = { clientX: 550, clientY: 415, button: 0, buttons: 1, pointerId: 1, isPrimary: true };

    now = 10_000 + SETTLE_MS - 10;
    expect(fireEvent.pointerDown(approve, at)).toBe(false);
    now = 10_000 + SETTLE_MS + 100;
    fireEvent.pointerUp(approve, { ...at, buttons: 0 });
    fireEvent.click(approve, { ...at, buttons: 0, detail: 1 });
    fireEvent.dblClick(approve, { ...at, buttons: 0, detail: 2 });
    expect(events).toEqual([]);
  });

  it('is told of a move by a scroll, a resize and a character typed, none of which changes a part of the page', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    const field = document.createElement('textarea');
    document.body.append(field);
    await changed();
    const told: Array<[string, () => void]> = [
      ['a scroll', () => fireEvent.scroll(document.body)],
      ['a resize', () => fireEvent(window, new Event('resize'))],
      ['a character typed', () => fireEvent.input(field, { target: { value: 'a' } })],
    ];
    for (const [what, tell] of told) {
      now += 10_000;
      put(approve, box(500, 400));
      tell();
      // Told when it moved, the press half a second later is taken; found only at a press, that press would be refused.
      now += SETTLE_MS;
      expect(press(550, 415), what).toBe(true);
      put(approve, box(500, 300));
      now += 10_000;
      expect(press(550, 315), what).toBe(false);
    }
  });

  it('leaves alone a press on anything that does not act, and a click made from the keyboard', async () => {
    start();
    const events = heard(true);
    now = 10_000;
    const close = button('Close review', box(500, 400), false);
    const approve = button('Approve & rename', box(500, 300));
    await changed();

    now = 10_010;
    expect(press(550, 415)).toBe(true);
    expect(events).toEqual(['pointerdown:Close review', 'mousedown:Close review', 'pointerup:Close review', 'mouseup:Close review', 'click:Close review']);
    // Enter or the space bar on the button: a click with no press of the pointer in it.
    events.length = 0;
    fireEvent.click(approve, { detail: 0 });
    expect(events).toEqual(['click:Approve & rename']);
    expect(close.isConnected).toBe(true);
  });

  it('finds at the press a change it was not told of, and refuses that press', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const events = heard();
    now = 10_000;
    // Moved with no change to the page: nothing said so.
    put(approve, box(500, 400));

    expect(press(550, 415)).toBe(false);
    now = 10_000 + SETTLE_MS;
    expect(press(550, 415)).toBe(true);
    expect(events.filter((event) => event.startsWith('click'))).toEqual(['click:Approve & rename']);
  });

  it('refuses a press on a control that was under something else until a moment ago', async () => {
    start();
    const renameAll = button('Rename all ready', box(500, 400));
    const drawer = document.createElement('aside');
    document.body.append(drawer);
    put(drawer, box(400, 0, 400, 800));
    await changed();
    const events = heard();
    now = 10_000;
    // The drawer closes by itself; the button under it has not moved.
    drawer.remove();
    put(drawer, undefined);
    await changed();

    now = 10_250;
    expect(press(550, 415)).toBe(false);
    expect(events).toEqual([]);
    now = 10_250 + AGAIN_MS;
    expect(press(550, 415)).toBe(true);
    expect(renameAll.isConnected).toBe(true);
  });

  it('takes a press on what the person\'s own press opened, once the pointer has gone to it, and not where it opened under the pointer', async () => {
    start();
    const more = button('More', box(700, 400, 30), false);
    const menu: HTMLButtonElement[] = [];
    more.addEventListener('click', () => { menu.push(button('Analyze again', box(500, 300)), button('Remove', box(690, 395, 60, 40))); });
    await changed();
    const events = heard();
    now = 10_000;
    expect(press(715, 415)).toBe(true);
    await changed();
    events.length = 0;

    // Analyze again opened away from the pointer: taken as soon as the pointer is on it.
    now = 10_090;
    fireEvent.pointerMove(menu[0], { clientX: 550, clientY: 315, pointerId: 1, isPrimary: true });
    expect(press(550, 315)).toBe(true);
    // Remove opened over More, under the pointer that pressed it: a second click there is not taken.
    events.length = 0;
    now = 10_180;
    expect(press(715, 415)).toBe(false);
    expect(events).toEqual([]);
  });

  it('refuses the click of a finger that lifts over a control its own press uncovered', async () => {
    start();
    const discard = button('Discard waiting', box(500, 400));
    const menu = button('A menu over it', box(450, 350, 300, 200), false);
    // The menu closes as it is pressed, as a menu does at a press outside its items.
    menu.addEventListener('pointerdown', () => { menu.remove(); put(menu, undefined); });
    await changed();
    const events = heard();

    now = 10_000;
    expect(press(550, 415, { pointerType: 'touch' })).toBe(true);
    expect(events.filter((event) => event.startsWith('click'))).toEqual([]);
    expect(discard.isConnected).toBe(true);
  });

  it('is not made good by a second finger: the first finger\'s press stays refused', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    const elsewhere = button('Close review', box(100, 100), false);
    await changed();
    const events = heard();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    const first = { clientX: 550, clientY: 415, button: 0, buttons: 1, pointerId: 7, isPrimary: true, pointerType: 'touch' };

    now = 10_250;
    expect(fireEvent.pointerDown(approve, first)).toBe(false);
    // Another finger comes down somewhere else while the first is held, long enough for the settle time to pass.
    now = 10_000 + SETTLE_MS + 200;
    fireEvent.pointerDown(elsewhere, { clientX: 150, clientY: 115, button: 0, buttons: 1, pointerId: 8, isPrimary: false, pointerType: 'touch' });
    events.length = 0;
    fireEvent.pointerUp(approve, { ...first, buttons: 0 });
    fireEvent.click(approve, { ...first, buttons: 0, detail: 1 });
    expect(events).toEqual([]);
  });

  it('does not take for the person\'s own a change it finds only at the next press', async () => {
    start();
    const renameAll = button('Rename all ready', box(100, 300));
    const field = document.createElement('textarea');
    document.body.append(field);
    await changed();
    const events = heard();
    now = 10_000;
    fireEvent.pointerMove(document.body, { clientX: 900, clientY: 50, pointerId: 1, isPrimary: true });
    // A key that is stopped on its way through the page: the window does not see it leave.
    field.addEventListener('keydown', (event) => event.stopPropagation());
    fireEvent.keyDown(field, { key: 'Escape' });
    // Then, by itself and with no change to the page, the button comes somewhere the pointer was not.
    put(renameAll, box(500, 400));

    now = 10_001;
    expect(press(550, 415)).toBe(false);
    expect(events).toEqual([]);
  });

  it('does not take for the person\'s own a change the page draws after a key of theirs that was stopped on its way', async () => {
    start();
    const cancel = button('Cancel processing', box(100, 300));
    const field = document.createElement('textarea');
    document.body.append(field);
    await changed();
    const events = heard();
    now = 10_000;
    fireEvent.pointerMove(document.body, { clientX: 900, clientY: 50, pointerId: 1, isPrimary: true });
    // A key the page stops - Tab in what was kept of a name - so the window does not see it leave.
    field.addEventListener('keydown', (event) => event.stopPropagation());
    fireEvent.keyDown(field, { key: 'Tab' });
    // The document is read again by itself, and the page draws it once the key has been handled: the button comes where the name was.
    put(cancel, box(500, 400));
    cancel.setAttribute('data-moved', '');
    await changed();

    // The pointer goes there and presses, a quarter of a second later.
    now = 10_250;
    fireEvent.pointerMove(cancel, { clientX: 550, clientY: 415, pointerId: 1, isPrimary: true });
    expect(press(550, 415)).toBe(false);
    expect(events).toEqual([]);
    now = 10_250 + AGAIN_MS;
    expect(press(550, 415)).toBe(true);
  });

  it('lets the right button take no focus on a control that has just come under the pointer', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const events = heard();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    const at = { clientX: 550, clientY: 415, pointerId: 1, isPrimary: true, button: 2, buttons: 2 };

    // The browser moves the focus at the mousedown it sends after a pointerdown nobody refused.
    now = 10_250;
    expect(fireEvent.pointerDown(approve, at)).toBe(false);
    expect(events).toEqual([]);
    now = 10_250 + AGAIN_MS;
    expect(fireEvent.pointerDown(approve, at)).toBe(true);
    expect(events).toEqual(['pointerdown:Approve & rename']);
  });

  it('judges the main button pressed while another is held, which sends no pointerdown of its own', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const events = heard();
    const at = { clientX: 550, clientY: 415, pointerId: 1, isPrimary: true };
    // It comes as a move that names the button, and as a mousedown: either is enough to refuse it.
    for (const comesAs of ['a move that names the button', 'a mousedown'] as const) {
      put(approve, box(500, 300));
      approve.setAttribute('data-moved', 'back');
      await changed();
      // The right button goes down on the button and is held.
      now += 9_000;
      expect(fireEvent.pointerDown(approve, { ...at, clientY: 315, button: 2, buttons: 2 }), comesAs).toBe(true);
      now += 1_000;
      put(approve, box(500, 400));
      approve.setAttribute('data-moved', comesAs);
      await changed();
      events.length = 0;

      now += 250;
      if (comesAs === 'a mousedown') expect(fireEvent.mouseDown(approve, { ...at, button: 0, buttons: 3 })).toBe(false);
      else expect(fireEvent.pointerMove(approve, { ...at, button: 0, buttons: 3 })).toBe(false);
      fireEvent.mouseUp(approve, { ...at, button: 0, buttons: 2 });
      fireEvent.click(approve, { ...at, button: 0, buttons: 2, detail: 1 });
      expect(events, comesAs).toEqual([]);
      fireEvent.pointerUp(approve, { ...at, button: 2, buttons: 0 });
    }
  });

  it('knows Approve & rename as one button through the new one the panel draws for each document, and no other button so', async () => {
    start();
    const events = heard();
    const drawn = (name: string, mark: string, of: string) => { const control = button(name, box(500, 400)); control.setAttribute('data-acts', mark); control.setAttribute('data-acts-for', of); return control; };
    // The button drawn again is for the next document, or - the last - for the same one, read again by itself while something else had it disabled.
    for (const [mark, next, taken] of [['approve', 'employment', true], ['true', 'employment', false], ['approve', 'lease', false]] as const) {
      events.length = 0;
      let approve = drawn('Approve & rename', mark, 'lease');
      await changed();
      now += 10_000;
      expect(press(550, 415)).toBe(true);
      // The decision is carried out: disabled, gone while the document is shown as filed, and drawn again for the next.
      approve.disabled = true;
      approve.setAttribute('disabled', '');
      await changed();
      now += 40;
      approve.remove();
      put(approve, undefined);
      await changed();
      now += 60;
      approve = drawn('Apply rename', mark, next);
      await changed();

      now += 20;
      expect(press(550, 415), `${mark} for ${next}`).toBe(taken);
      expect(events.filter((event) => event.startsWith('click'))).toEqual(taken ? ['click:Approve & rename', 'click:Apply rename'] : ['click:Approve & rename']);
      approve.remove();
      put(approve, undefined);
      await changed();
    }
  });

  /** Each click the page gets, and whether a button that acts once takes it for a first press. */
  const clicks = () => {
    const got: string[] = [];
    document.addEventListener('click', (event) => got.push(`click ${event.detail} on ${(event.target as Element).textContent}: ${firstPress({ detail: event.detail, nativeEvent: event }) ? 'a first press' : 'not a first press'}`), { signal: hearing.signal });
    return got;
  };

  it('takes for a first press the one made again at once after a refused one, which the browser counts second, and no press after that', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const got = clicks();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();

    // Refused; made again within the double-click time, once the button has been there for the settle time; and a third time.
    now = 10_300;
    expect(press(550, 415, { detail: 1 })).toBe(false);
    now = 10_700;
    expect(press(550, 415, { detail: 2 })).toBe(true);
    now = 10_900;
    expect(press(550, 415, { detail: 3 })).toBe(true);
    expect(got).toEqual(['click 2 on Approve & rename: a first press', 'click 3 on Approve & rename: not a first press']);

    // Two refused, and the third taken: still nothing was done before it.
    got.length = 0;
    now = 20_000;
    put(approve, box(500, 300));
    approve.setAttribute('data-moved', 'back');
    await changed();
    now = 20_150;
    expect(press(550, 315, { detail: 1 })).toBe(false);
    now = 20_350;
    expect(press(550, 315, { detail: 2 })).toBe(false);
    now = 20_350 + AGAIN_MS;
    expect(press(550, 315, { detail: 3 })).toBe(true);
    expect(got).toEqual(['click 3 on Approve & rename: a first press']);
  });

  it('refuses the second click of a double-click whose first was refused, though the settle time has run out by then', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const got = clicks();
    // The button comes under the pointer by itself; a double-click begins 405 ms later and ends at 542.
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    now = 10_405;
    expect(press(550, 415, { detail: 1 })).toBe(false);
    now = 10_542;
    expect(press(550, 415, { detail: 2 })).toBe(false);
    expect(got).toEqual([]);
    // Three quick presses, the last after the settle time: each within the time of the one before, none taken.
    now = 20_000;
    put(approve, box(500, 300));
    approve.setAttribute('data-moved', 'back');
    await changed();
    for (const [at, detail] of [[20_253, 1], [20_394, 2], [20_563, 3]] as const) { now = at; expect(press(550, 315, { detail })).toBe(false); }
    expect(got).toEqual([]);
    // One made after a pause is a press made again: taken, and a first press.
    now = 20_563 + AGAIN_MS;
    expect(press(550, 315, { detail: 1 })).toBe(true);
    expect(got).toEqual(['click 1 on Approve & rename: a first press']);
    // A press on another control that acts, at once after a refused one, is refused too; on one that does not act it is not.
    const keep = button('Keep original', box(700, 300));
    const row = button('A row', box(100, 100), false);
    await changed();
    now = 30_000;
    put(approve, box(500, 500));
    approve.setAttribute('data-moved', 'again');
    await changed();
    now = 30_200;
    expect(press(550, 515, { detail: 1 })).toBe(false);
    now = 30_300;
    expect(press(150, 115, { detail: 1 })).toBe(true);
    now = 30_350;
    expect(press(750, 315, { detail: 1 })).toBe(false);
    now = 30_350 + AGAIN_MS;
    expect(press(750, 315, { detail: 1 })).toBe(true);
    expect(keep.isConnected && row.isConnected).toBe(true);
  });

  it('does not forget a key pressed while the refused press was still down: the click that ends that press is counted 1', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const got = clicks();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    const at = { clientX: 550, clientY: 415, button: 0, buttons: 1, pointerId: 1, isPrimary: true };

    // The button goes down too soon and is refused; Ctrl+Enter decides the document while it is still down.
    now = 10_340;
    expect(fireEvent.pointerDown(approve, at)).toBe(false);
    now = 10_375;
    fireEvent.keyDown(document.body, { key: 'Enter', ctrlKey: true });
    fireEvent.keyUp(document.body, { key: 'Enter', ctrlKey: true });
    // Let go on the next document's button, in the same place: the browser counts that click 1, and it begins a run.
    now = 10_480;
    fireEvent.pointerUp(approve, { ...at, buttons: 0 });
    fireEvent.click(approve, { ...at, buttons: 0, detail: 1 });
    expect(got).toEqual([]);
    // Pressed again, counted 2, 300 ms and more after the first: the rule takes it, and it is no first press.
    now = 10_700;
    expect(press(550, 415, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 2 on Approve & rename: not a first press']);
  });

  it('does not take for a first press a click counted second after a refused press and a key: the key may have decided the document', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    await changed();
    const got = clicks();
    now = 10_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    now = 10_340;
    expect(press(550, 415, { detail: 1 })).toBe(false);
    // Ctrl+Enter decides the document from the keyboard; the next document's button is in the same place.
    now = 10_500;
    fireEvent.keyDown(document.body, { key: 'Enter', ctrlKey: true });
    fireEvent.keyUp(document.body, { key: 'Enter', ctrlKey: true });
    now = 10_700;
    expect(press(550, 415, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 2 on Approve & rename: not a first press']);
    // With no key between, the same press is a first press (the test above); a key before the run began changes nothing.
    got.length = 0;
    now = 20_000;
    put(approve, box(500, 300));
    approve.setAttribute('data-moved', 'back');
    await changed();
    fireEvent.keyDown(document.body, { key: 'j' });
    now = 20_300;
    expect(press(550, 315, { detail: 1 })).toBe(false);
    now = 20_700;
    expect(press(550, 315, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 2 on Approve & rename: a first press']);
  });

  it('does not take the second click of a double-click for a first press, on the button the first was on or on what the first opened', async () => {
    start();
    const approve = button('Approve & rename', box(500, 400));
    const more = button('More', box(700, 400, 30), false);
    more.addEventListener('click', () => { button('Remove from queue', box(690, 395, 60, 40)); }, { once: true });
    await changed();
    const got = clicks();

    now = 10_000;
    expect(press(550, 415, { detail: 1 })).toBe(true);
    now = 10_120;
    expect(press(550, 415, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 1 on Approve & rename: a first press', 'click 2 on Approve & rename: not a first press']);

    // More opens Remove under the pointer. A person whose double-click time
    // is longer than the settle time makes the second click after it: the
    // rule takes the press, and it is still the second click of a double-click.
    got.length = 0;
    now = 20_000;
    expect(press(715, 415, { detail: 1 })).toBe(true);
    await changed();
    now = 20_000 + SETTLE_MS;
    expect(press(715, 415, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 1 on More: a first press', 'click 2 on Remove from queue: not a first press']);
  });

  it('does not take for a first press a click counted second after a press that reached the page with no click of its own, or after a click from the keyboard', async () => {
    start();
    const approve = button('Approve & rename', box(500, 300));
    const row = button('A row', box(100, 100), false);
    await changed();
    const got = clicks();
    const at = { clientX: 550, clientY: 415, button: 0, buttons: 1, pointerId: 1, isPrimary: true };

    // The button goes down on the page and comes up with no click: a drag.
    now = 10_000;
    fireEvent.pointerDown(row, { ...at, clientX: 150, clientY: 115 });
    fireEvent.pointerUp(document.body, { ...at, buttons: 0 });
    now = 10_200;
    expect(press(550, 315, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 2 on Approve & rename: not a first press']);

    // A press is refused; Enter on a button does something; the press made again is counted second.
    got.length = 0;
    now = 20_000;
    put(approve, box(500, 400));
    approve.setAttribute('data-moved', '');
    await changed();
    now = 20_300;
    expect(press(550, 415, { detail: 1 })).toBe(false);
    fireEvent.click(row, { detail: 0 });
    now = 20_700;
    expect(press(550, 415, { detail: 2 })).toBe(true);
    expect(got).toEqual(['click 0 on A row: a first press', 'click 2 on Approve & rename: not a first press']);
  });

  it('does not take for a first press a click counted second after the main button was pressed, while another was held, on something that took it', async () => {
    start();
    button('Approve & rename', box(500, 400));
    const row = button('A row', box(100, 100), false);
    await changed();
    const got = clicks();
    const at = { clientX: 150, clientY: 115, pointerId: 1, isPrimary: true };

    // The right button is held on the page; the main one goes down on a row and comes up with no click, then is pressed on the button.
    now = 10_000;
    fireEvent.pointerDown(row, { ...at, button: 2, buttons: 2 });
    fireEvent.mouseDown(row, { ...at, button: 2, buttons: 2 });
    now = 10_100;
    fireEvent.mouseDown(row, { ...at, button: 0, buttons: 3 });
    now = 10_200;
    const approve = document.elementFromPoint(550, 415)!;
    fireEvent.mouseDown(approve, { ...at, clientX: 550, clientY: 415, button: 0, buttons: 3 });
    fireEvent.click(approve, { ...at, clientX: 550, clientY: 415, button: 0, buttons: 2, detail: 2 });
    expect(got).toEqual(['click 2 on Approve & rename: not a first press']);

    // Nor after a click whose press the window did not see begin: a second finger's.
    got.length = 0;
    fireEvent.pointerUp(row, { ...at, button: 2, buttons: 0 });
    now = 20_000;
    fireEvent.click(approve, { clientX: 550, clientY: 415, detail: 1 });
    fireEvent.click(approve, { clientX: 550, clientY: 415, detail: 2 });
    expect(got).toEqual(['click 1 on Approve & rename: a first press', 'click 2 on Approve & rename: not a first press']);
  });

  it('asks only the count of a click where no window applies the rule: the first, or one from the keyboard', () => {
    const click = (detail: number) => firstPress({ detail, nativeEvent: new MouseEvent('click', { detail }) });
    expect([click(0), click(1), click(2), click(3)]).toEqual([true, true, false, false]);
  });

  it('stops looking when it is told to', async () => {
    start();
    stop();
    const events = heard();
    button('Approve & rename', box(500, 400));
    await changed();

    expect(press(550, 415)).toBe(true);
    expect(events.filter((event) => event.startsWith('click'))).toEqual(['click:Approve & rename']);
  });
});
