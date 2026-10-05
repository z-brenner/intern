import { X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { Icon } from './Icon';

/** How long a success stays on screen, counted while nobody is pointing at it or working in it. */
export const TOAST_TIMEOUT_MS = 10_000;

export type ToastTone = 'success' | 'progress' | 'error';

interface Props {
  tone: ToastTone;
  /** Its accessible name: "Action error" for a failure, as the banner it replaces was named. */
  label: string;
  children: ReactNode;
  /** The one thing to do about it - Undo. `name` is the button's accessible name when the label alone is too short to say what it undoes. */
  action?: { label: string; name?: string; disabled?: boolean; onClick(): void };
  /** Work still running cannot be waved away, so progress has no Dismiss. */
  onDismiss?(): void;
}

/**
 * The outcome of a queue action, at the foot of the queue.
 *
 * A success leaves on its own after ten seconds, but not while the pointer or
 * focus is on it, so Undo never disappears from under someone reaching for
 * it; the ten seconds start again when they leave. A failure stays until it
 * is dismissed. It used to stay until the next action, with no way to close
 * it, centred over whatever was beneath it - the drawer's own buttons
 * included.
 *
 * Only a failure is a live region: an alert, because it is created with its
 * sentence already in it, and a polite region that arrives complete is not
 * reliably spoken. A success or progress is announced once through the app's
 * "Action status" and is not read a second time from here.
 */
export function Toast({ tone, label, children, action, onDismiss }: Props) {
  const [held, setHeld] = useState(false);
  const dismiss = useRef(onDismiss);
  useEffect(() => { dismiss.current = onDismiss; });
  useEffect(() => {
    if (tone !== 'success' || held) return;
    const timer = window.setTimeout(() => dismiss.current?.(), TOAST_TIMEOUT_MS);
    return () => window.clearTimeout(timer);
  }, [tone, held]);
  return <div className={`toast toast--${tone}`} role={tone === 'error' ? 'alert' : 'group'} aria-label={label}
    onPointerEnter={() => setHeld(true)} onPointerLeave={() => setHeld(false)}
    onFocus={() => setHeld(true)} onBlur={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setHeld(false); }}>
    <p>{children}</p>
    {action && <button type="button" className="toast-action" disabled={action.disabled} aria-label={action.name} onClick={action.onClick}>{action.label}</button>}
    {onDismiss && <button type="button" className="icon-button" aria-label="Dismiss" onClick={onDismiss}><Icon icon={X} /></button>}
  </div>;
}
