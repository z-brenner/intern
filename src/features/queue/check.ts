import { describeNotes, flaggedFields, humanizeReason, noteCodes, outcomeFirst, reasonCodes, shortNote, shortOutcome, shortReason } from '../../lib/reasons';
import type { ReasonField } from '../../lib/reasons';
import type { QueueItem } from '../../types';
import { keptOriginal } from '../review/actions';

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

/** What Intern noted about a document that is not waiting: the codes, and the sentences for them. */
export interface ItemNote {
  /** The codes noted; none when the reason came as words this build has no codes for. */
  codes: string[];
  text: string;
}

/**
 * What Intern noted about the reading of a document that is not waiting
 * for a person, if anything. At the Balanced review level a reading can be
 * ready while it carries review codes - a type Intern supplied, a model
 * unsure of its reading - and a document renamed while it carried them, by
 * Intern itself, is filed with them still on it. They are notes there, not
 * what holds the document: the row is ready, or filed, and says so, with
 * the note beside it.
 *
 * Not every reason on such a row is a note about its reading. An approved
 * name is the person's own, and a copy says what it is in its own words. A
 * filed document that kept its original name was not renamed by what the
 * codes are about, and "already had this name" and "exact copy" say what
 * became of the file. A ready row whose reason is not a list of codes this
 * build knows still has a note, in the words that came: it is never shown
 * as having passed every check.
 */
export function itemNote(item: QueueItem): ItemNote | undefined {
  if (item.copyOf !== undefined) return undefined;
  if (item.status === 'ready' && !item.approved) {
    const codes = itemReasonCodes(item);
    const noted = noteCodes(codes);
    if (noted.length) return { codes: noted, text: describeNotes(noted) };
    return item.reason && !codes.length ? { codes: [], text: item.reason } : undefined;
  }
  if (item.status === 'completed' && !keptOriginal(item)) {
    const noted = noteCodes(itemReasonCodes(item));
    return noted.length ? { codes: noted, text: describeNotes(noted) } : undefined;
  }
  return undefined;
}

/**
 * The notes a document carries through two states in which it is neither
 * ready nor renamed, said as notes; undefined for every other row, whose
 * reason stands as it came.
 *
 * While Intern renames a ready document that had notes, the row is being
 * renamed with the reasons still on it (a person's approval clears them, so
 * a rename a person asked for carries none). And a document that already
 * had the name Intern would give it is completed under that name with
 * ALREADY_NAMED added to the reasons it had - beside other codes only when
 * Intern completed it by itself, for the same reason. Neither waits for a
 * person, and the reason both arrive with is the review's sentences, which
 * tell the person to check the name. A completed one says first what
 * became of the file.
 *
 * Only a reason whose codes this build knows is said again: the codes are
 * what the sentences were made from. A document a person kept under its own
 * name from review has no ALREADY_NAMED and keeps the review's words, as it
 * always has.
 */
export function carriedNotes(item: QueueItem): string | undefined {
  const codes = item.reasonCodes ?? [];
  if (!item.reason || !codes.length) return undefined;
  if (item.status === 'processing' && item.stage === 'filing') return describeNotes(codes);
  if (item.status === 'completed' && codes.includes('ALREADY_NAMED')) return describeNotes(outcomeFirst(codes));
  return undefined;
}

/** How a Check cell is drawn. `noted`: a note on a row that is not waiting (not `note`, which the stylesheet uses for a boxed notice). */
export type CheckTone = 'review' | 'failed' | 'ready' | 'noted' | 'quiet';

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
    // Ready with something noted is not "all checks passed": it says what was noted.
    const note = itemNote(item);
    if (note) return noteCell(note);
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
    // Renamed with something noted: the mark that finds it again among the filed.
    const note = itemNote(item);
    if (note) return noteCell(note);
    // Not renamed with a note: what became of the file leads, and anything it carried is said as a note.
    const short = shortOutcome(itemReasonCodes(item));
    return short ? { ...short, tone: 'quiet', title: carriedNotes(item) ?? title } : undefined;
  }
  return undefined;
}

/**
 * The Check cell of a row with a note: "Note:" and the few words for it -
 * a review's for that code, unless those would be false of a row that is
 * not waiting - so a copied cell still says the row was not waiting, with
 * the note's sentences as the tooltip.
 */
function noteCell(note: ItemNote): CheckCell {
  const short = shortNote(note.codes);
  return short
    ? { text: `Note: ${short.text}`, more: short.more, tone: 'noted', title: note.text }
    : { text: `Note: ${note.text}`, more: 0, tone: 'noted', title: note.text };
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
/** A connecting word, or the " - " Intern writes before a party it gives no relation to. */
const SUBJECT_RELATION = / (between|with|from|for|to|-) /;
/** What stands between the organization a name leads with and its subject (own_names.rs). */
const ENTITY_SEPARATOR = ' - ';
/**
 * Words that name a kind of document (DOCUMENT_KINDS in own_names.rs, and
 * "Document", which a name says where the type should be when there is none).
 */
const DOCUMENT_KIND = /\b(agreement|contract|addendum|amendment|invoice|bill|receipt|statement|letter|notice|notification|memo|memorandum|minutes|report|proposal|quote|quotation|order|certificate|policy|lease|deed|resolution|application|form|return|email|correspondence|communications|offer|claim|complaint|judgment|motion|plan|schedule|sow|nda|msa|license|licence|permit|waiver|release|consent|confirmation|summons|subpoena|petition|bylaws|charter|budget|presentation|payslip|timesheet|document)\b/i;

/**
 * The organization a name's words after its date lead with, as Intern
 * composes `<entity> - <subject>` (NameView in own_names.rs), or nothing
 * for a name that does not lead with one.
 *
 * The words before the first " - " are the entity where what follows holds
 * the subject's connecting word ("Vistage International - Host Chair SOW
 * for John Smith") or a " - " of its own ("Acme - Notice - John Smith"),
 * but not where they hold a connecting word themselves ("Letter to Smith -
 * signed" is a note). Two parts and no connecting word read either way:
 * "Nimbus Orchard Supply - Invoice" leads with an organization, and
 * "Notice of Termination - John Smith", a document naming only people, is
 * the type and its party. The side that names a kind of document is the
 * type; where neither or both do, the entity, which names lead with far
 * more often.
 */
function entityOf(words: string): string | undefined {
  const at = words.indexOf(ENTITY_SEPARATOR);
  if (at <= 0) return undefined;
  const before = words.slice(0, at);
  const after = words.slice(at + ENTITY_SEPARATOR.length);
  if (!after.trim() || RELATION.test(` ${before} `)) return undefined;
  if (after.includes(ENTITY_SEPARATOR) || RELATION.test(` ${after} `)) return before;
  return DOCUMENT_KIND.test(before) && !DOCUMENT_KIND.test(after) ? undefined : before;
}

/**
 * A composed name split into the parts Intern built it from - "2024-02-09",
 * " ", "Fabrikam", " - ", "Invoice INV-3301", " for ", "Contoso Ltd", ".pdf"
 * - with the parts in doubt flagged. The organization a name leads with is
 * one of the document's parties, and is flagged with them. A name that does
 * not read that way (one typed by hand) is one unflagged run, except for a
 * leading date, which every name has.
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
  if (!date) {
    push(rest);
    push(extension);
    return parts;
  }
  const space = rest.startsWith(' ') ? ' ' : '';
  push(space);
  let subject = rest.slice(space.length);
  const entity = entityOf(subject);
  if (entity !== undefined) {
    push(entity, 'parties');
    push(ENTITY_SEPARATOR);
    subject = subject.slice(entity.length + ENTITY_SEPARATOR.length);
  }
  // The first connecting word, or the " - " before a party given no
  // relation: "Notice of Termination - John Smith".
  const relation = SUBJECT_RELATION.exec(subject);
  if (relation) {
    push(subject.slice(0, relation.index), 'type');
    push(relation[0]);
    push(subject.slice(relation.index + relation[0].length), 'parties');
  } else {
    push(subject, 'type');
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
 * The party is the other side's, after the organization the name leads
 * with ("Vistage International - Host Chair SOW for John Smith"): the
 * backend gives only the subject's parties as the name's (`nameParties`),
 * so the entity is read as part of the words before the type's, and stays.
 * A name that leads with an organization and gives no other party has the
 * party added after its type with " - ", as Intern composes one given no
 * relation: "Nimbus Orchard Supply - Invoice - Contoso Ltd".
 *
 * The party replaced is the longest of the known ones the stem ends with:
 * the proposed name's own, or a choice an earlier click put there - so of
 * "between A and B" only B, and "between" keeps both its sides. Otherwise
 * the proposed name says where its party starts: after its type and the
 * connecting word that follows it, which the reviewer may have changed ("to"
 * for "from"), or - in a name that gave none - after its whole type, where
 * the party is added after " - ", as Intern names a party it gives no
 * relation to. A note typed after a known party there - " re Merger" after
 * the proposed "to U.S. Department of Justice", or after a party an earlier
 * click put in - stays after the new one (`knownLead`). Any other words
 * there go with the party they follow: a party typed over by hand, and one
 * lengthened by hand ("Smith" to "Smith and Sons, Inc."), are replaced
 * whole, since no known party says where they end. A name rewritten past
 * what the proposed one says keeps every word the reviewer typed, with the
 * party added after " - ". No type word is taken for a party. A collision
 * suffix is left off: the rename gives the new name its own if it still
 * needs one.
 */
export function withParty(stem: string, party: string, offer: PartyOffer): string {
  const body = withoutCollision(stem);
  const known = [...offer.nameParties, ...offer.choices, party];
  const ending = partyEnding(body, known);
  if (ending) return `${body.slice(0, body.length - ending.length)}${party}`;
  const slot = partySlot(body, offer);
  if (slot) {
    const there = knownLead(slot.words, known);
    return `${slot.before}${party}${there ? slot.words.slice(there.length) : ''}`;
  }
  const kept = body.trimEnd();
  return kept ? `${kept} - ${party}` : party;
}

/**
 * Which of the known parties - the proposed name's own and those offered -
 * a stem gives, read as `withParty` reads it: the longest it ends with, or
 * the one its party starts with, before a note typed after it. The chip for
 * it is shown pressed.
 */
export function partyGiven(stem: string, offer: PartyOffer): string | undefined {
  const body = withoutCollision(stem);
  const known = [...offer.nameParties, ...offer.choices];
  const ending = partyEnding(body, known);
  if (ending) return ending;
  const slot = partySlot(body, offer);
  return slot ? knownLead(slot.words, known) : undefined;
}

/** Where a party goes in a stem: the words before it, from the date to the connecting word, and the words there now. */
interface PartySlot { before: string; words: string }

/**
 * Where the proposed name says a stem's party is: after its type and the
 * connecting word that follows it, which the reviewer may have changed, or
 * after the words its party follows when no one connecting word does
 * ("Notice between A and "); in a name that gave none and is still its
 * whole type, after " - ", with no words there yet. None for a stem
 * rewritten past what the proposed name says.
 */
function partySlot(body: string, offer: PartyOffer): PartySlot | undefined {
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
    if (rest === type) return { before: dated(`${type} - `), words: '' };
    const relation = rest.startsWith(type) ? SINGLE_RELATION.exec(rest.slice(type.length)) : null;
    if (relation) return { before: dated(`${type}${relation[0]}`), words: rest.slice(type.length + relation[0].length) };
  } else if (given && rest.startsWith(lead)) {
    return { before: dated(lead), words: rest.slice(lead.length) };
  }
  return undefined;
}

/**
 * The longest of `known` a party's words start with, followed by more
 * words: "U.S. Department of Justice" of "U.S. Department of Justice re
 * Merger". None when they start with no known party, as a party typed over
 * by hand does, and none when they are only a party, which `partyEnding`
 * finds.
 */
function partyStart(words: string, known: readonly string[]): string | undefined {
  return [...known].filter(Boolean).sort((a, b) => b.length - a.length).find((run) => words.startsWith(`${run} `));
}

/**
 * The start of a note about the document, after a party: " re Merger",
 * " re: the lease", " regarding", " about", " concerning", " - signed",
 * " (signed)". Words that do not start one may be more of the party.
 */
const NOTE_AFTER_PARTY = /^ (?:re:?|regarding|about|concerning|-) |^ \(/i;

/**
 * The known party a party's words start with, when what follows it is a
 * note (`NOTE_AFTER_PARTY`): "U.S. Department of Justice" of "U.S.
 * Department of Justice re Merger". None when the words after it may be
 * more of the party, as in "Smith and Sons, Inc." - "Smith" says where that
 * party starts, not where it ends.
 */
function knownLead(words: string, known: readonly string[]): string | undefined {
  const run = partyStart(words, known);
  return run && NOTE_AFTER_PARTY.test(words.slice(run.length)) ? run : undefined;
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
