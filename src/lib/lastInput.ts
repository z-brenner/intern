/*
  Whether the person's last input was a press of the pointer - a mouse
  button, a pen, a finger - rather than a key.

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
*/

let pressed = false;

if (typeof window !== 'undefined') {
  window.addEventListener('pointerdown', () => { pressed = true; }, { capture: true, passive: true });
  window.addEventListener('keydown', () => { pressed = false; }, { capture: true, passive: true });
}

/** True from a press of the pointer until the next key is pressed. */
export const afterAPress = (): boolean => pressed;
