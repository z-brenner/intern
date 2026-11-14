/*
  Whether the person's last input was a press of the pointer - a mouse
  button, a pen, a finger - rather than a key.

  A press is aimed at what is under the pointer, and the next press is
  aimed there too: the second of a double-click, however slow, or a press
  that follows the first because nothing seemed to happen. So nothing a
  press sets off scrolls what is under the pointer: not the focus the press
  itself moves - the review panel scrolls a control focused under its
  decision clear of it - and not the focus code moves in answer to it: the
  question "Use on its own" opening in Settings where its action was, with
  the focus in its field. A scroll then brings something else under the
  pointer for the next press; when the question was asked in the review
  panel, its scroll brought the question's yes there. Focus moved after a
  key is scrolled into sight as usual: whoever pressed the key is not
  aiming the pointer.
*/

let pressed = false;

if (typeof window !== 'undefined') {
  window.addEventListener('pointerdown', () => { pressed = true; }, { capture: true, passive: true });
  window.addEventListener('keydown', () => { pressed = false; }, { capture: true, passive: true });
}

/** True from a press of the pointer until the next key is pressed. */
export const afterAPress = (): boolean => pressed;
