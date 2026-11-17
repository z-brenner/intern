import { X } from 'lucide-react';
import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';
import { useEditsOnlyInside } from '../lib/dialogEdits';
import { tabWithin } from '../lib/dialogTab';
import { Icon } from './Icon';

interface Props {
  /** The question, as the dialog's heading and name: "Clear the queue?" */
  title: string;
  /** What the yes says it does: "Clear queue". */
  confirm: string;
  busy?: boolean;
  /** What it takes, what it leaves, and that it cannot be undone. */
  children: ReactNode;
  onConfirm(): void;
  onCancel(): void;
}

/**
 * The question before Clear queue or Clear completed: how many documents
 * go, that the files stay where they are, and that it cannot be undone.
 * Both clears ask it the same way, from the same place in their views.
 *
 * Cancel has the focus, so Enter or the space bar pressed on the way in
 * clears nothing; Escape is Cancel. The yes is a control that acts
 * (`data-acts`): drawn under a pointer that has not moved to it - the second
 * click of a double-click on the menu entry that opened this - a press on it
 * is not taken (src/lib/aimedPress.ts). Same backdrop and focus trap as
 * Rename history, and nothing behind it can take the focus or change.
 */
export function ClearDialog({ title, confirm, busy = false, children, onConfirm, onCancel }: Props) {
  const dialog = useRef<HTMLElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  const close = useRef(onCancel);
  useEffect(() => { close.current = onCancel; }, [onCancel]);
  useEditsOnlyInside(dialog);
  useEffect(() => {
    cancel.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { event.preventDefault(); close.current(); return; }
      tabWithin(event, Array.from(dialog.current?.querySelectorAll<HTMLElement>('button:not([disabled])') ?? []));
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, []);
  return <div className="dialog-backdrop" role="presentation"><section ref={dialog} className="settings-dialog clear-dialog" role="alertdialog" aria-modal="true" aria-labelledby="clear-dialog-title" aria-describedby="clear-dialog-body">
    <div className="dialog-head"><h2 id="clear-dialog-title">{title}</h2><button type="button" className="icon-button" onClick={onCancel} aria-label="Close"><Icon icon={X} /></button></div>
    <div id="clear-dialog-body" className="clear-dialog-body">{children}</div>
    <div className="history-actions">
      <button ref={cancel} type="button" onClick={onCancel}>Cancel</button>
      <button type="button" data-acts className="primary danger" disabled={busy} onClick={onConfirm}>{confirm}</button>
    </div>
  </section></div>;
}
