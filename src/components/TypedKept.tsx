import { useId, useRef } from 'react';
import type { KeyboardEvent, MouseEvent, ReactNode } from 'react';
import { nameThenDescription } from '../features/review/drafts';
import type { KeptText } from '../features/review/drafts';

/**
 * A name and a description as a person had them, kept on screen where
 * typing changes nothing.
 *
 * The fields a person types a name and a description in can go by
 * themselves: the document is read again, or filed from another machine, or
 * leaves the queue, or the panel closes as the window changes width. The
 * caret went with them, and what the person typed next was no longer text
 * but review's keys - J and K move the selection, Enter opens a document at
 * its name and files it, Ctrl+Enter approves - so the rest of a sentence
 * filed documents nobody had looked at. What they had typed went too.
 *
 * So it stays, here: read-only, with the caret when the caret was in the
 * field that went. A read-only field is still a field - a letter typed in
 * it is nothing, and no review key acts from it (useReviewShortcuts) - and
 * it goes only when the person says so, with the button.
 *
 * Tab does not take the caret out of it. A person in a name presses Tab to
 * go on to the description: here that would put the focus on the button,
 * where a space typed next presses it - what was kept, thrown away - and
 * where J, K, Ctrl+Enter and Alt+K are review's keys again. Escape is the
 * way out from the keyboard, to the button; a press anywhere is the other.
 */
export function TypedKept({ kept, label, className, dismiss, onDismiss, children }: {
  kept: KeptText;
  /** The group's name: whose text this is. */
  label: string;
  className?: string;
  /** The button that takes it off the screen, and what it does. */
  dismiss: string;
  onDismiss(): void;
  /** Why it is here, in a sentence. */
  children: ReactNode;
}) {
  const noteId = useId();
  const button = useRef<HTMLButtonElement>(null);
  const held = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.nativeEvent.isComposing) return;
    if (event.key === 'Tab' && !event.ctrlKey && !event.metaKey && !event.altKey) { event.preventDefault(); event.stopPropagation(); return; }
    if (event.key !== 'Escape') return;
    // Its own Escape: in the drawer, one that reached the panel would close it.
    event.preventDefault();
    event.stopPropagation();
    button.current?.focus();
  };
  // Only a press of its own takes it away: the second click of a
  // double-click lands on whatever has moved under the pointer by then.
  const press = (event: MouseEvent<HTMLButtonElement>) => { if (event.detail <= 1) onDismiss(); };
  return <div className={`typed-kept${className ? ` ${className}` : ''}`} role="group" aria-label={label}>
    <p id={noteId}>{children} Typing here changes nothing, and Tab stays here: press Escape, or click, to leave it.</p>
    <textarea readOnly data-typed-kept={kept.key} aria-label="What you had typed" aria-describedby={noteId} spellCheck={false} rows={3} value={kept.text} onKeyDown={held} />
    <button ref={button} type="button" onClick={press}>{dismiss}</button>
  </div>;
}

/**
 * The name or description the caret is in - a field of the review panel, or
 * what review keeps of one (TypedKept) - and the text to keep if that field
 * goes: both fields of the panel, the name and the description under it.
 */
export function typingInReview(): { field: HTMLTextAreaElement; text: string; kept: boolean } | undefined {
  const field = document.activeElement;
  if (!(field instanceof HTMLTextAreaElement)) return undefined;
  if (field.dataset.typedKept !== undefined) return { field, text: field.value, kept: true };
  if (field.dataset.reviewField === undefined) return undefined;
  const fields = field.closest('.proposal');
  const name = fields?.querySelector<HTMLTextAreaElement>('[data-review-field="name"]')?.value ?? '';
  const description = fields?.querySelector<HTMLTextAreaElement>('[data-review-field="description"]')?.value ?? '';
  return { field, text: nameThenDescription(name, description), kept: false };
}
