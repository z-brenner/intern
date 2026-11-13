import { fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

/*
  Test helpers for the yes of "Use on its own", which takes a press of the
  pointer only from a pointer that came onto it. jsdom draws nothing, so the
  yes is drawn here at a box a test can move, a pointer is reported on the
  yes when it is in that box, as a browser finds what is under it, and the
  pointer is moved and pressed at points on it and off it.
*/

export type Point = { clientX: number; clientY: number };

/**
 * Where the yes is drawn until a test moves it, a point on it, and one off
 * it, above it on its sentence, away from it: more than its height off it,
 * as a pointer moved to it comes from.
 */
export const yesAt = new DOMRect(100, 200, 120, 30);
export const onTheYes: Point = { clientX: 150, clientY: 212 };
export const offTheYes: Point = { clientX: 150, clientY: 160 };

const YES = '.rule-confirm button.primary';

/** Draw every question's yes at `current` from now on; a test moves it by setting `current`. Restored with the other mocks. */
export function drawYes() {
  const box = { current: yesAt };
  const drawn = Element.prototype.getBoundingClientRect;
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function (this: Element) {
    return this.matches(YES) ? box.current : drawn.call(this);
  });
  return box;
}

/** What the browser finds under `at`: a yes drawn there, or else the page. */
export function under(at: Point): Element {
  return [...document.querySelectorAll(YES)].find((yes) => {
    const box = yes.getBoundingClientRect();
    return at.clientX >= box.left && at.clientX < box.right && at.clientY >= box.top && at.clientY < box.bottom;
  }) ?? document.body;
}

/** The mouse moved to `to`, reported on what is under it there, or on `on` when something else is drawn over it. */
export const moveTo = (to: Point, on: Element = under(to)) => fireEvent.pointerMove(on, { ...to, pointerType: 'mouse' });

/** The next animation frame. */
export const nextFrame = () => new Promise<void>((resolve) => { requestAnimationFrame(() => resolve()); });

/**
 * A press at `at` on `button`, as Chromium makes one: the pointer down, the
 * mouse down, which focuses the button unless that is prevented, the
 * pointer and the mouse up, and a click counting one press.
 */
export function pressAt(button: HTMLElement, at: Point, pointerType = 'mouse') {
  fireEvent.pointerDown(button, { ...at, pointerType, buttons: 1 });
  if (fireEvent.mouseDown(button, { ...at })) button.focus();
  fireEvent.pointerUp(button, { ...at, pointerType });
  fireEvent.mouseUp(button, { ...at });
  fireEvent.click(button, { ...at, detail: 1 });
}
