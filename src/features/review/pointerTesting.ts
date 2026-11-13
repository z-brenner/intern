import { fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

/*
  Test helpers for the yes of "Use on its own", which takes a press of the
  pointer only once the pointer came onto it. jsdom draws nothing, so the
  yes is drawn here at a box a test can move, and the pointer is moved and
  pressed at points on it and off it.
*/

export type Point = { clientX: number; clientY: number };

/** Where the yes is drawn until a test moves it, and a point on it and one off it, above it, on its sentence. */
export const yesAt = new DOMRect(100, 200, 120, 30);
export const onTheYes: Point = { clientX: 150, clientY: 212 };
export const offTheYes: Point = { clientX: 150, clientY: 180 };

/** Draw every question's yes at `current` from now on; a test moves it by setting `current`. Restored with the other mocks. */
export function drawYes() {
  const box = { current: yesAt };
  const drawn = Element.prototype.getBoundingClientRect;
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    return this.matches('.rule-confirm button.primary') ? box.current : drawn.call(this);
  });
  return box;
}

/** The mouse moved to `to`. */
export const moveTo = (to: Point) => fireEvent.pointerMove(document.body, { ...to, pointerType: 'mouse' });

/** A press at `at` on `button`: it focuses the button, as it does in Chromium, and counts one press. */
export function pressAt(button: HTMLElement, at: Point, pointerType = 'mouse') {
  fireEvent.pointerDown(button, { ...at, pointerType });
  button.focus();
  fireEvent.click(button, { detail: 1 });
}
