/*
  What the person last did with the pointer - a mouse, a pen, a finger -
  and with the keys: whether the last input was a press or a key, when that
  press began, and where each kind of pointer was last seen.

  A press is aimed at what is under the pointer, and the next press is
  aimed there too: the second of a double-click, however slow, or a press
  that follows the first because nothing seemed to happen. So nothing a
  press sets off scrolls what is under the pointer: not the focus the press
  itself moves, and not the focus code moves in answer to it - a question
  opening where its action was, with the focus on its Cancel. The review
  panel scrolled that Cancel clear of the decision stuck to its bottom, and
  brought the question's yes under the pointer that had pressed the action.
  Focus moved after a key is scrolled into sight as usual: whoever pressed
  the key is not aiming the pointer.

  Where the pointer is matters for the same reason. What is under a pointer
  that has not moved was not chosen by it: it may have come there by a
  scroll, by a row above it closing, by a question opening under it, by a
  menu over it closing. Only a pointer that moved, or a finger that touched
  down somewhere new, says where the person is aiming. Each kind of pointer
  is its own: a finger's tap says nothing of where the mouse is. Every time
  a pointer is reported is passed on with the element the browser found
  under it, also when it is reported where it already was: that says what
  is now under a pointer that stayed put, which is not a move.

  And the page sees a pointer that hovers - a mouse, or a pen that reports
  hovering - only while it is over the page, with nothing of the system's
  over it. When the page loses sight of it, it may move anywhere unseen: over
  another window or the system's menu lying over the page, which then goes
  away and leaves it where the page did not see it go. Where it is next
  reported is where it was found, not where it moved to, so the page forgets
  where it last saw it and tells the watchers it is lost. The page loses
  sight of it when it leaves the page (`pointerout` with nothing to go to),
  when the window loses the focus, when the system's menu opens (a
  `contextmenu` the page did not take for itself), when the browser takes it
  (`pointercancel`, a drag), when a press it saw begin ended where the page
  could not see it - a list or a menu the press opened took the release, and
  the pointer is next reported with nothing pressed - and when the window
  moved on the screen: snapped by a key or moved by another program, the
  page moved under a pointer that did not move. A finger, and a pen that
  does not hover, is seen only where it touches down: each touch is where it
  was aimed, and nothing is lost between touches.
*/

/**
 * Where a pointer was seen, what kind it is, whether it was seen by a press
 * there rather than a move, and whether it was hovering: moved with nothing
 * pressed, as a mouse and some pens do and a finger never does.
 */
export type PointerSeen = { x: number; y: number; type: string; press: boolean; hover: boolean };

/** What a watcher of the pointer is told: each time a pointer is reported, and each time the page loses sight of one. */
export type PointerWatch = {
  seen(at: PointerSeen, under: EventTarget | null): void;
  lost(type: string): void;
};

let pressed = false;
let pressBegan: number | undefined;
const pointers = new Map<string, PointerSeen>();
/** The kinds of pointer that hover: a mouse, and a pen once it has been seen hovering. */
const hovering = new Set<string>(['mouse']);
/** The pointers that hover whose press the page saw begin, and not yet end. */
const holding = new Set<string>();
const watchers = new Set<PointerWatch>();
/** Where the window was on the screen when a pointer was last reported. */
let windowAt = typeof window === 'undefined' ? { x: 0, y: 0 } : { x: window.screenX, y: window.screenY };

const lose = (types: Iterable<string>) => {
  for (const type of [...types]) {
    if (!hovering.has(type)) continue;
    pointers.delete(type);
    holding.delete(type);
    for (const watch of watchers) watch.lost(type);
  }
};

const see = (event: PointerEvent, press: boolean) => {
  const type = event.pointerType;
  const hover = !press && event.buttons === 0;
  if (hover) hovering.add(type);
  if (window.screenX !== windowAt.x || window.screenY !== windowAt.y) {
    windowAt = { x: window.screenX, y: window.screenY };
    lose(hovering);
  }
  // Seen pressing again, or with nothing pressed, without its press seen to end.
  if (holding.has(type) && (press || event.buttons === 0)) lose([type]);
  const seen = { x: event.clientX, y: event.clientY, type, press, hover };
  pointers.set(type, seen);
  for (const watch of watchers) watch.seen(seen, event.target);
  if (press && hovering.has(type)) holding.add(type);
};

if (typeof window !== 'undefined') {
  window.addEventListener('pointermove', (event) => see(event, false), { capture: true, passive: true });
  window.addEventListener('pointerdown', (event) => { pressed = true; pressBegan = performance.now(); see(event, true); }, { capture: true, passive: true });
  window.addEventListener('pointerup', (event) => { holding.delete(event.pointerType); }, { capture: true, passive: true });
  window.addEventListener('pointercancel', (event) => lose([event.pointerType]), { capture: true, passive: true });
  window.addEventListener('pointerout', (event) => { if (event.relatedTarget === null) lose([event.pointerType]); }, { capture: true, passive: true });
  // The window's own blur: an element's does not bubble, and is not caught here.
  window.addEventListener('blur', () => lose(hovering));
  // After the page's own handlers: a menu the page shows instead is the page's.
  window.addEventListener('contextmenu', (event) => { if (!event.defaultPrevented) lose(hovering); });
  window.addEventListener('keydown', () => { pressed = false; }, { capture: true, passive: true });
}

/** True from a press of the pointer until the next key is pressed. */
export const afterAPress = (): boolean => pressed;

/** When the last press of the pointer began (`performance.now()`), while no key has been pressed since. */
export const pressBeganAt = (): number | undefined => pressed ? pressBegan : undefined;

/** Where each kind of pointer was last seen, for those seen at all and not lost since. */
export const lastPointers = (): PointerSeen[] => [...pointers.values()];

/**
 * Tells `watch` each time a pointer is reported - a move, or a press - with
 * the element the browser found under it, a report where that pointer
 * already was included; and each time the page loses sight of a pointer
 * that hovers, which it then forgets. Returns the call that stops it.
 */
export function watchPointer(watch: PointerWatch): () => void {
  watchers.add(watch);
  return () => { watchers.delete(watch); };
}
