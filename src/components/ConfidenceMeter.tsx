import { confidence, confidenceTone } from '../lib/format';
import type { QueueStatus } from '../types';

/**
 * Confidence, kept exact and made comparable.
 *
 * The number is the claim and it never leaves: a person deciding whether to
 * trust a proposed filename is owed the figure, not an adjective. It is how
 * sure Intern is of the name, worked out in one place in the backend
 * (`name_confidence` in review_level.rs), and a document in review never
 * shows as surer than one that is ready. The track beside it exists only so
 * a column of forty rows can be scanned without reading every number.
 *
 * Both take their color from the row's status, never from the figure
 * (`confidenceTone`): green for ready or renamed, amber for review. The
 * meter carries that tone itself, so wherever it is shown its color says
 * what the status says and nothing else.
 *
 * An absent confidence stays an em dash rather than an empty track, because
 * "not measured" and "zero" are different things.
 */
export function ConfidenceMeter({ value, status, variant = 'cell' }: { value?: number; status: QueueStatus; variant?: 'cell' | 'panel' }) {
  if (value === undefined || !Number.isFinite(value)) return <>—</>;
  const percent = Math.max(0, Math.min(100, Math.round(value * 100)));
  const tone = confidenceTone(status);
  return <span className={`confidence-meter confidence-meter--${variant}${tone ? ` confidence ${tone}` : ''}`}>
    <span className="confidence-value">{confidence(value)}</span>
    <span className="meter" aria-hidden="true"><span className="meter-fill" style={{ width: `${percent}%` }} /></span>
  </span>;
}
