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

/** What a party chosen in review is put in a name with. */
export interface PartyOffer {
  /** The stem Intern proposed, before any edit: the name `nameParties` were read from. */
  proposed: string;
  /**
   * The parties the proposed name gives, as it writes them - read by the
   * backend with the grammar that composed it, so a type holding "for" is
   * never taken for a party - and none when it gives none.
   */
  nameParties: readonly string[];
  /** The parties offered, one of which an earlier click may have put in the name. */
  choices: readonly string[];
}

/** A name's own collision suffix, " (2)": no part of what a reviewer names, and given again by the rename if still needed. */
const COLLISION = / \(\d+\)$/;
/** The connecting word a single party follows, as Intern composes it (PartyRelation, and "-" for none), at the start of what follows a type. */
const SINGLE_RELATION = /^ (with|from|for|to|-) /;
/** The same word at the end of the words before a party. */
const RELATION_BEFORE_PARTY = / (with|from|for|to|-) $/;

/**
 * A name's stem with `party` in place of the party it gives: the reviewer's
 * one-click fix for a name that gives the wrong party.
 *
 * The party replaced is the longest of the known ones the stem ends with:
 * the proposed name's own, or a choice an earlier click put there - so of
 * "between A and B" only B, and "between" keeps both its sides. Otherwise
 * the proposed name says where its party starts: after its type and the
 * connecting word that follows it, which the reviewer may have changed ("to"
 * for "from"), or - in a name that gave none - after its whole type, where
 * the party is added after " - ", as Intern names a party it gives no
 * relation to. A name rewritten past what the proposed one says keeps every
 * word the reviewer typed, with the party added after " - ". No type word is
 * taken for a party. A collision suffix is left off: the rename gives the
 * new name its own if it still needs one.
 */
export function withParty(stem: string, party: string, offer: PartyOffer): string {
  const body = withoutCollision(stem);
  const ending = partyEnding(body, [...offer.nameParties, ...offer.choices, party]);
  if (ending) return `${body.slice(0, body.length - ending.length)}${party}`;
  const date = LEADING_DATE.exec(body)?.[0] ?? '';
  const rest = body.slice(date.length).trimStart();
  const dated = (words: string) => (date ? `${date} ${words}` : words);
  const proposed = undated(withoutCollision(offer.proposed));
  const given = partyEnding(proposed, offer.nameParties);
  // The words before the proposed party - "Letter to ", "Notice between A
  // and " - or, with none, the whole name.
  const lead = given ? proposed.slice(0, proposed.length - given.length) : offer.nameParties.length ? '' : proposed;
  const single = given ? RELATION_BEFORE_PARTY.exec(lead) : null;
  const type = given ? (single ? lead.slice(0, single.index) : '') : lead;
  if (type) {
    if (rest === type) return dated(`${type} - ${party}`);
    const relation = rest.startsWith(type) ? SINGLE_RELATION.exec(rest.slice(type.length)) : null;
    if (relation) return dated(`${type}${relation[0]}${party}`);
  } else if (given && rest.startsWith(lead)) {
    return dated(`${lead}${party}`);
  }
  const kept = body.trimEnd();
  return kept ? `${kept} - ${party}` : party;
}

/** A stem without its leading date. */
function undated(stem: string): string {
  return stem.slice(LEADING_DATE.exec(stem)?.[0].length ?? 0).trimStart();
}

function withoutCollision(stem: string): string {
  return stem.trimEnd().replace(COLLISION, '').trimEnd();
}

/** The longest of `known` a name's stem ends with, as its last words, collision suffix aside: the party it now gives, of those. */
export function partyEnding(stem: string, known: readonly string[]): string | undefined {
  const body = withoutCollision(stem);
  return [...known].filter(Boolean).sort((a, b) => b.length - a.length).find((run) => body.endsWith(` ${run}`));
}
