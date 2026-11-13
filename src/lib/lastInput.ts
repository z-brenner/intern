/*
  What the person last did with the pointer - a mouse, a pen, a finger -
  and with the keys: whether the last input was a press or a key, and
  where the pointer was last seen.

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
  scroll, by a row above it closing, by a question opening under it. Only a
  pointer that moved, or a finger that touched down somewhere new, says
  where the person is aiming. A move to where the pointer already was is no
  move: browsers send one after a scroll or a change of layout to say what
  is now under a pointer that stayed put - WebKit, which Intern runs in on
  macOS and Linux, above all - and it is not counted.
*/

/** Where the pointer was seen, what kind it is, and whether it was seen by a press there rather than a move. */
export type PointerSeen = { x: number; y: number; type: string; press: boolean };

let pressed = false;
let pointer: PointerSeen | undefined;
const watchers = new Set<(seen: PointerSeen, before: PointerSeen | undefined) => void>();

const see = (event: PointerEvent, press: boolean) => {
  if (pointer && event.clientX === pointer.x && event.clientY === pointer.y) return;
  const before = pointer;
  pointer = { x: event.clientX, y: event.clientY, type: event.pointerType, press };
  for (const watch of watchers) watch(pointer, before);
};

if (typeof window !== 'undefined') {
  window.addEventListener('pointermove', (event) => see(event, false), { capture: true, passive: true });
  window.addEventListener('pointerdown', (event) => { pressed = true; see(event, true); }, { capture: true, passive: true });
  window.addEventListener('keydown', () => { pressed = false; }, { capture: true, passive: true });
}

/** True from a press of the pointer until the next key is pressed. */
export const afterAPress = (): boolean => pressed;

/** Where the pointer was last seen, if it has been. */
export const lastPointer = (): PointerSeen | undefined => pointer;

/**
 * Calls `watch` each time the pointer is seen somewhere new - a move, or a
 * press where the pointer had not been seen - with where it was before.
 * Returns the call that stops it.
 */
export function watchPointer(watch: (seen: PointerSeen, before: PointerSeen | undefined) => void): () => void {
  watchers.add(watch);
  return () => { watchers.delete(watch); };
}
