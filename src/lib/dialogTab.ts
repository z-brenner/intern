/**
 * Tab and Shift+Tab in a dialog: to the control after or before the focus
 * among the dialog's `controls`, round from the last to the first and back,
 * wherever the focus is. It used to move only from one of the controls
 * themselves; from anywhere else the browser moved it. After a press on the
 * dialog's heading or its backdrop the focus is on nothing, and Shift+Tab
 * walked out of Settings to the review panel behind it, where Enter filed
 * the document. From nothing, or from behind the dialog, Tab goes to its
 * first control and Shift+Tab to its last; from something in the dialog
 * that is not one of them - a row a press took the focus to - to the next
 * or the one before, as they come. Whether it moved the focus.
 */
export function tabWithin(event: KeyboardEvent, controls: HTMLElement[]): boolean {
  if (event.key !== 'Tab' || controls.length === 0) return false;
  const count = controls.length;
  const at = document.activeElement ?? document.body;
  const index = controls.indexOf(at as HTMLElement);
  const after = (control: HTMLElement) => Boolean(at.compareDocumentPosition(control) & Node.DOCUMENT_POSITION_FOLLOWING);
  const next = index !== -1
    ? controls[(index + (event.shiftKey ? count - 1 : 1)) % count]
    : event.shiftKey
      ? [...controls].reverse().find((control) => !after(control)) ?? controls[count - 1]
      : controls.find(after) ?? controls[0];
  event.preventDefault();
  next.focus();
  return true;
}
