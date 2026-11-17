import { ChevronDown } from 'lucide-react';
import { useId, useState } from 'react';
import { Icon } from '../../components/Icon';

/** One reason as the review panel shows it: a few words, and why. */
export interface Reason {
  code: string;
  /** "Party unclear". */
  label: string;
  /** The sentence behind it; empty for a code this build has none for. */
  why: string;
}

/**
 * A document's reasons as a row of chips, a few words each, for a person to
 * take in at a glance. They used to be one run of sentences - a paragraph
 * for some codes - and the panel read like a manual.
 *
 * Each chip with a sentence behind it is its own small disclosure: pressed,
 * the sentence shows under the row; hovered, it is the tooltip. Nothing a
 * person needs to act on is lost, it is only quiet until asked for. A chip
 * for a code with no sentence (an unmapped code) is the code as it came,
 * with nothing to open. What is open is the panel's for this document only:
 * the row is keyed by the document, so the next one starts closed.
 */
export function ReasonChips({ reasons, label }: { reasons: readonly Reason[]; label: string }) {
  const [open, setOpen] = useState<ReadonlySet<string>>(() => new Set());
  const id = useId();
  const toggle = (code: string) => setOpen((current) => {
    const next = new Set(current);
    if (!next.delete(code)) next.add(code);
    return next;
  });
  return <div className="reason-chips">
    <ul aria-label={label}>
      {reasons.map((reason, index) => <li key={reason.code}>
        {reason.why
          ? <button type="button" className="reason-chip" title={reason.why} aria-expanded={open.has(reason.code)} aria-controls={`${id}-${index}`} onClick={() => toggle(reason.code)}>
            {reason.label}<Icon icon={ChevronDown} />
          </button>
          : <span className="reason-chip">{reason.label}</span>}
      </li>)}
    </ul>
    {reasons.map((reason, index) => reason.why && <p key={reason.code} id={`${id}-${index}`} className="reason-why" hidden={!open.has(reason.code)}>
      <strong>{reason.label}:</strong> {reason.why}
    </p>)}
  </div>;
}
