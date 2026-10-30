import { flaggedFields, humanizeReason, reasonCodes, shortReason } from '../../lib/reasons';
import type { ReasonField } from '../../lib/reasons';
import type { QueueItem } from '../../types';

/**
 * The backend's review codes for an item: the list it sent as its reason,
 * or else its one error code. The humanized `reason` is for people; these
 * say which part of a proposal is in doubt.
 */
export function itemReasonCodes(item: QueueItem): string[] {
  if (item.reasonCodes?.length) return item.reasonCodes;
  return item.errorCode ? reasonCodes(item.errorCode) : [];
}

/** The parts of an item's proposed name its review is about. */
export function itemFlaggedFields(item: QueueItem): ReasonField[] {
  return item.status === 'review' ? flaggedFields(itemReasonCodes(item)) : [];
}

export type CheckTone = 'review' | 'failed' | 'ready' | 'quiet';

export interface CheckCell {
  /** A few words: "Date could be day- or month-first". */
  text: string;
  /** Further reasons, counted rather than listed. */
  more: number;
  tone: CheckTone;
  /** The full sentence, for the tooltip. */
  title?: string;
}

/**
 * What the Check column says for a row. It replaced the model's own
 * confidence percentage, which said how sure a two-billion-parameter model
 * felt (a figure small models overstate) and not what was wrong. Review is
 * routed by specific checks - a date that reads either way round, a party
 * the document never names - and this column says which one failed.
 *
 * Undefined for a row with nothing to say yet (waiting, or being read).
 */
export function checkCell(item: QueueItem): CheckCell | undefined {
  const title = item.reason ? humanizeReason(item.reason) : undefined;
  if (item.status === 'ready') {
    if (item.approved) return { text: 'Approved', more: 0, tone: 'ready', title };
    if (item.copyOf !== undefined) return { text: 'Exact copy', more: 0, tone: 'quiet', title };
    return { text: 'All checks passed', more: 0, tone: 'ready', title: 'The date, type and parties in this name were each found in the document.' };
  }
  if (item.status === 'review' || item.status === 'failed') {
    const tone: CheckTone = item.status === 'failed' ? 'failed' : 'review';
    const short = shortReason(itemReasonCodes(item));
    if (short) return { ...short, tone, title };
    // Free text - "Duplicate of ...", "Canceled." - is its own short form.
    if (title) return { text: title, more: 0, tone, title };
    return { text: item.status === 'failed' ? 'Could not be read' : 'Needs a decision', more: 0, tone };
  }
  // A filed document's note - "already had this name", "exact copy" - in a word.
  if (item.status === 'completed') {
    const short = shortReason(itemReasonCodes(item));
    return short ? { ...short, tone: 'quiet', title } : undefined;
  }
  return undefined;
}

/** The Check cell as plain text, as it is copied. */
export function checkText(item: QueueItem): string {
  const cell = checkCell(item);
  if (!cell) return '';
  return cell.more ? `${cell.text} (+${cell.more} more)` : cell.text;
}

/** One run of a proposed name, marked when it is a part the review is about. */
export interface NamePart { text: string; field?: ReasonField; flagged: boolean }

const LEADING_DATE = /^\d{4}-\d{2}-\d{2}/;
/** The words Intern composes between a name's type and its parties (PartyRelation). */
const RELATION = / (between|with|from|for|to) /;

/**
 * A composed name split into the parts Intern built it from - "2024-02-09",
 * " Invoice INV-3301", " from ", "Fabrikam Inc", ".pdf" - with the parts in
 * doubt flagged. A name that does not read that way (one typed by hand) is
 * one unflagged run, except for a leading date, which every name has.
 */
export function nameParts(name: string, flagged: readonly ReasonField[]): NamePart[] {
  const extension = /\.[A-Za-z0-9]{1,6}$/.exec(name)?.[0] ?? '';
  const stem = extension ? name.slice(0, -extension.length) : name;
  const parts: NamePart[] = [];
  const push = (text: string, field?: ReasonField) => {
    if (text) parts.push({ text, field, flagged: field !== undefined && flagged.includes(field) });
  };
  const date = LEADING_DATE.exec(stem)?.[0] ?? '';
  const rest = stem.slice(date.length);
  push(date, 'date');
  const relation = RELATION.exec(rest);
  if (date && relation) {
    const space = rest.startsWith(' ') ? ' ' : '';
    push(space);
    push(rest.slice(space.length, relation.index), 'type');
    push(relation[0]);
    push(rest.slice(relation.index + relation[0].length), 'parties');
  } else if (date) {
    const space = rest.startsWith(' ') ? ' ' : '';
    push(space);
    push(rest.slice(space.length), 'type');
  } else {
    push(rest);
  }
  push(extension);
  return parts;
}
