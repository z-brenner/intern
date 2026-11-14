import { fireEvent } from '@testing-library/react';

/*
  Test helpers for the field "Use on its own" asks the spelling to be typed
  into. jsdom types nothing: these send the events a browser sends when a
  person types into a field, composes in it with an input method, or puts
  text in it some other way.
*/

const inputType = (field: HTMLElement, type: string, data: string) =>
  fireEvent(field, new InputEvent('beforeinput', { inputType: type, data, bubbles: true, cancelable: true }));

/** `text` typed into `field` a key at a time: the key, the input it is about to make, and the input made, each only when the one before was not refused. */
export function typeInto(field: HTMLElement, text: string) {
  for (const data of text) {
    if (!fireEvent.keyDown(field, { key: data }) || !inputType(field, 'insertText', data)) continue;
    fireEvent.input(field, { target: { value: (field as HTMLInputElement).value + data }, inputType: 'insertText', data });
  }
}

/** Each of `steps` composed in `field` with an input method, the last one committed, as Chromium sends it. */
export function composeInto(field: HTMLElement, steps: string[]) {
  const before = (field as HTMLInputElement).value;
  fireEvent.compositionStart(field, { data: '' });
  for (const step of steps) {
    fireEvent.keyDown(field, { key: 'Process', keyCode: 229, isComposing: true });
    fireEvent(field, new InputEvent('beforeinput', { inputType: 'insertCompositionText', data: step, bubbles: true, cancelable: false }));
    fireEvent.input(field, { target: { value: before + step }, inputType: 'insertCompositionText', data: step, isComposing: true });
  }
}

/** The input method's Enter, which commits what was composed, and the end of the composition. */
export function commitComposition(field: HTMLElement) {
  fireEvent.keyDown(field, { key: 'Enter', isComposing: true });
  fireEvent.compositionEnd(field, { data: (field as HTMLInputElement).value });
}

/**
 * `text` put in `field` at once, outside a composition, after `keys` keys:
 * an input method that shows no composition commits so what its keys
 * spelled, and, with a key or two or none, an input method's or an
 * on-screen keyboard's clipboard, or an emoji keyboard, puts in a word.
 */
export function insertAtOnce(field: HTMLElement, text: string, keys: number) {
  for (let key = 0; key < keys; key += 1) fireEvent.keyDown(field, { key: 'Process', keyCode: 229 });
  if (!inputType(field, 'insertText', text)) return;
  fireEvent.input(field, { target: { value: (field as HTMLInputElement).value + text }, inputType: 'insertText', data: text });
}

/**
 * Every way but typing to put `text` in `field`, each tried on its own: a
 * paste and a drop, the input each would make - and a suggestion taken, a
 * yank, an undo, and a word put in at once with no key for it - and
 * autofill's input, which says no input type. Says each that put anything
 * in the field, or that the browser was not told to refuse.
 */
export function fillWithoutTyping(field: HTMLElement, text: string): string[] {
  const value = () => (field as HTMLInputElement).value;
  const through: string[] = [];
  const clipboardData = { getData: () => text, types: ['text/plain'] };
  if (fireEvent.paste(field, { clipboardData })) through.push('paste');
  if (fireEvent.drop(field, { dataTransfer: clipboardData })) through.push('drop');
  for (const type of ['insertText', 'insertFromPaste', 'insertFromPasteAsQuotation', 'insertFromDrop', 'insertReplacementText', 'insertFromYank', 'historyUndo', 'historyRedo']) {
    if (inputType(field, type, text)) through.push(`beforeinput ${type}`);
    const was = value();
    fireEvent.input(field, { target: { value: was + text }, inputType: type, data: text });
    if (value() !== was) through.push(`input ${type}`);
  }
  const was = value();
  fireEvent.change(field, { target: { value: was + text } });
  if (value() !== was) through.push('autofill');
  return through;
}

/** Everything in `field` deleted, as Ctrl+A and Backspace do. */
export function clearField(field: HTMLElement) {
  if ((field as HTMLInputElement).value === '') return;
  fireEvent(field, new InputEvent('beforeinput', { inputType: 'deleteContentBackward', bubbles: true, cancelable: true }));
  fireEvent.input(field, { target: { value: '' }, inputType: 'deleteContentBackward' });
}
