import { useEffect } from 'react';
import type { RefObject } from 'react';

/**
 * While the dialog `dialog` holds is open, no field outside it changes: an
 * input aimed outside it is refused before the browser makes it. The
 * browser's undo and redo go to the field edited last in the page, wherever
 * the focus is, and take the focus there. With Settings open over the review
 * window, Ctrl+Z in a field of Settings with nothing of its own to undo
 * undid the review panel's Filename behind the dialog and put the caret in
 * it, where the next Enter filed the document. Refused, an undo leaves the
 * field and the focus where they were.
 */
export function useEditsOnlyInside(dialog: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const outside = (event: InputEvent) => {
      const inside = dialog.current;
      if (inside && event.target instanceof Node && !inside.contains(event.target)) event.preventDefault();
    };
    document.addEventListener('beforeinput', outside, true);
    return () => document.removeEventListener('beforeinput', outside, true);
  }, [dialog]);
}
