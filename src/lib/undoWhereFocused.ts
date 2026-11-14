import { useEffect } from 'react';

/**
 * An undo or a redo changes only the field with the focus. The browser's
 * undo and redo go to the field edited last in the page, wherever the focus
 * is, and take the focus there. In the review panel, Ctrl+Z in the
 * Description, in the queue's filter or after a press on the panel's text
 * undid the Filename's last edit and put the caret in it, where the next
 * Enter - meant for a new line in the Description - filed the document
 * under the name half undone. In Settings, Ctrl+Z on a button undid an
 * unsaved edit to this machine's name, and Save stored it. Refused, the undo
 * leaves every field, and the focus, where they were; in the field with the
 * focus it is the browser's, as ever.
 */
export function useUndoOnlyWhereFocused() {
  useEffect(() => {
    const elsewhere = (event: InputEvent) => {
      if (event.inputType.startsWith('history') && event.target !== document.activeElement) event.preventDefault();
    };
    document.addEventListener('beforeinput', elsewhere, true);
    return () => document.removeEventListener('beforeinput', elsewhere, true);
  }, []);
}
