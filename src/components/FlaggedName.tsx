import { nameParts } from '../features/queue/check';
import type { ReasonField } from '../lib/reasons';

const FIELD_NAMES: Record<ReasonField, string> = { date: 'date', type: 'document type', parties: 'parties', description: 'description' };

/**
 * A proposed filename with the parts its review is about marked where they
 * are - the date of a date that reads either way round, the parties of a
 * party the document never names - so the eye goes to what needs checking
 * instead of re-reading the whole name. Unflagged, it is the plain name.
 *
 * The mark is not the only signal: the Check column and the review panel
 * say the same in words.
 */
export function FlaggedName({ name, flagged }: { name: string; flagged: readonly ReasonField[] }) {
  if (!flagged.length) return <>{name}</>;
  const parts = nameParts(name, flagged);
  if (!parts.some((part) => part.flagged)) return <>{name}</>;
  return <>{parts.map((part, index) => part.flagged && part.field
    ? <mark key={index} className="flagged" title={`Check the ${FIELD_NAMES[part.field]}`}>{part.text}</mark>
    : <span key={index}>{part.text}</span>)}</>;
}
