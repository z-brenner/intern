import { ChevronDown } from 'lucide-react';
import { useEffect, useId, useRef, useState } from 'react';
import { Icon } from './Icon';

export interface PrivacyFacts {
  /** Whether the local model's files are all in place. */
  modelDownloaded: boolean;
  /** Whether Intern checks for updates on its own (Settings can switch it off). */
  automaticUpdateChecks: boolean;
  /** When the last update check answered, this session. */
  lastUpdateCheck?: Date;
}

/**
 * The privacy badge, as a button that says what it means. "On this device"
 * on its own is a claim; opened, it is the short list of everything Intern
 * does send - the same list as the README's "What leaves your machine" -
 * with each one's state on this computer, so the claim can be checked
 * rather than taken on trust.
 *
 * Said once, here, and not repeated in toasts or banners: a promise made on
 * every screen becomes wallpaper.
 */
export function PrivacyBadge({ facts, onOpenSettings, onOpenGuide }: { facts: PrivacyFacts; onOpenSettings(trigger: HTMLButtonElement): void; onOpenGuide(): void }) {
  const [open, setOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const panelId = useId();
  useEffect(() => {
    if (!open) return;
    headingRef.current?.focus();
    const onPointer = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!panelRef.current?.contains(target) && !buttonRef.current?.contains(target)) setOpen(false);
    };
    document.addEventListener('pointerdown', onPointer);
    return () => document.removeEventListener('pointerdown', onPointer);
  }, [open]);
  const close = () => { setOpen(false); buttonRef.current?.focus(); };
  const label = 'Private · On this device';
  return <>
    <button ref={buttonRef} type="button" className="private" aria-expanded={open} aria-controls={open ? panelId : undefined}
      title="What Intern sends, and what it keeps on this computer"
      onClick={() => open ? close() : setOpen(true)}><i /><span className="private-label">{label}</span><Icon icon={ChevronDown} /></button>
    {open && <div ref={panelRef} id={panelId} className="privacy-popover" role="dialog" aria-label="What Intern sends"
      onKeyDown={(event) => { if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); close(); } }}>
      <h2 ref={headingRef} tabIndex={-1}>Your documents stay on this computer</h2>
      <p>Intern reads documents locally. If you use Ask Claude, you choose what to share in your browser.</p>
      <dl>
        <div><dt>Model download</dt><dd>{facts.modelDownloaded ? 'Done' : 'Not finished'}</dd>
          <dd className="privacy-detail">Once, at setup. Fetches the model's files and carries nothing about your documents.</dd></div>
        <div><dt>Update checks</dt><dd>{facts.automaticUpdateChecks ? (facts.lastUpdateCheck ? `Last at ${timeOf(facts.lastUpdateCheck)}` : 'Automatic') : 'Off'}</dd>
          <dd className="privacy-detail">Asks GitHub whether a newer signed build exists. Carries nothing about your documents.</dd></div>
      </dl>
      <p><button type="button" className="link-button" onClick={() => { setOpen(false); if (buttonRef.current) onOpenSettings(buttonRef.current); }}>Change in Settings</button>{' · '}
        <button type="button" className="link-button" onClick={() => { setOpen(false); onOpenGuide(); }}>How to check this yourself</button></p>
    </div>}
  </>;
}

function timeOf(date: Date) {
  return date.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
}
