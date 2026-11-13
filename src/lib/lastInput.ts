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
*/

/**
 * Where a pointer was seen, what kind it is, whether it was seen by a press
 * there rather than a move, and whether it was hovering: moved with nothing
 * pressed, as a mouse and some pens do and a finger never does.
 */
export type PointerSeen = { x: number; y: number; type: string; press: boolean; hover: boolean };

let pressed = false;
let pressBegan: number | undefined;
const pointers = new Map<string, PointerSeen>();
const watchers = new Set<(seen: PointerSeen, under: EventTarget | null) => void>();

const see = (event: PointerEvent, press: boolean) => {
  const seen = { x: event.clientX, y: event.clientY, type: event.pointerType, press, hover: !press && event.buttons === 0 };
  pointers.set(seen.type, seen);
  for (const watch of watchers) watch(seen, event.target);
};

if (typeof window !== 'undefined') {
  window.addEventListener('pointermove', (event) => see(event, false), { capture: true, passive: true });
  window.addEventListener('pointerdown', (event) => { pressed = true; pressBegan = performance.now(); see(event, true); }, { capture: true, passive: true });
  window.addEventListener('keydown', () => { pressed = false; }, { capture: true, passive: true });
}

/** True from a press of the pointer until the next key is pressed. */
export const afterAPress = (): boolean => pressed;

/** When the last press of the pointer began (`performance.now()`), while no key has been pressed since. */
export const pressBeganAt = (): number | undefined => pressed ? pressBegan : undefined;

/** Where each kind of pointer was last seen, for those seen at all. */
export const lastPointers = (): PointerSeen[] => [...pointers.values()];

/**
 * Calls `watch` each time a pointer is reported - a move, or a press -
 * with the element the browser found under it. A report where that pointer
 * already was is passed on too. Returns the call that stops it.
 */
export function watchPointer(watch: (seen: PointerSeen, under: EventTarget | null) => void): () => void {
  watchers.add(watch);
  return () => { watchers.delete(watch); };
}
