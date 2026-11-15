import { describe, expect, it } from 'vitest';
import { checkCell, checkText, itemFlaggedFields, itemNote, itemReasonCodes, nameParts, partyGiven, withParty } from './check';
import { describeNotes, humanizeReason, reasonCodes, shortReason } from '../../lib/reasons';
import type { QueueItem } from '../../types';

const review = (extra: Partial<QueueItem>): QueueItem => ({ id: 'r', originalFilename: 'Scan.pdf', status: 'review', proposedFilename: '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', ...extra });

describe('the Check column', () => {
  it('reads a list of codes, and nothing from free text', () => {
    expect(reasonCodes('DATE_AMBIGUOUS, LOW_CONFIDENCE')).toEqual(['DATE_AMBIGUOUS', 'LOW_CONFIDENCE']);
    // Free text with commas is not a code list.
    expect(reasonCodes('Duplicate of Contoso Worldwide, Inc.pdf')).toEqual([]);
    expect(reasonCodes('')).toEqual([]);
  });

  it('leads with the reason about a part of the name, and counts the rest', () => {
    expect(shortReason(['LOW_CONFIDENCE', 'PARTY_UNSUPPORTED'])).toEqual({ text: 'Party uncertain', more: 1 });
    expect(shortReason(['LOW_CONFIDENCE'])).toEqual({ text: 'Model unsure of its reading', more: 0 });
    expect(shortReason(['NOT_A_CODE'])).toBeUndefined();
  });

  it('has a few words for every review code the engine raises (ReviewReason::code)', () => {
    const codes = [
      'DATE_MISSING', 'DATE_UNSUPPORTED', 'TYPE_MISSING', 'TYPE_UNSUPPORTED', 'TYPE_INFERRED',
      'PARTY_UNSUPPORTED', 'PARTY_MISSING', 'DESCRIPTION_UNSUPPORTED', 'DESCRIPTION_INVALID',
      'LOW_CONFIDENCE', 'MODEL_REQUESTED_REVIEW', 'PARSER_WARNING', 'DATE_IMPLAUSIBLE',
      'DATE_IS_DEADLINE', 'DATE_AMBIGUOUS', 'DATE_IS_PERIOD', 'DATE_IS_FORMULA', 'FAST_MODEL_READING',
    ];
    for (const code of codes) expect(shortReason([code]), code).toBeDefined();
    expect(shortReason(['PARTY_MISSING'])).toEqual({ text: 'Party missing', more: 0 });
    expect(itemFlaggedFields(review({ reasonCodes: ['PARTY_MISSING'] }))).toEqual(['parties']);
    // A date that is only one end of a period leads over the model's doubt, and marks the date.
    expect(shortReason(['LOW_CONFIDENCE', 'DATE_IS_PERIOD'])).toEqual({ text: 'Date is a period\'s start or end', more: 1 });
    expect(itemFlaggedFields(review({ reasonCodes: ['DATE_IS_PERIOD'] }))).toEqual(['date']);
    // So does a date that only a TODAY or NOW formula shows.
    expect(shortReason(['LOW_CONFIDENCE', 'DATE_IS_FORMULA'])).toEqual({ text: 'Date comes from a formula', more: 1 });
    expect(itemFlaggedFields(review({ reasonCodes: ['DATE_IS_FORMULA'] }))).toEqual(['date']);
  });

  it('says what to check for a review, and that the checks passed for a ready row', () => {
    const ambiguous = review({ reasonCodes: ['DATE_AMBIGUOUS'], reason: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.' });
    expect(checkCell(ambiguous)).toEqual({ text: 'Date could be day- or month-first', more: 0, tone: 'review', title: ambiguous.reason });
    expect(checkCell({ ...ambiguous, status: 'ready', reasonCodes: undefined, reason: undefined })).toMatchObject({ text: 'All checks passed', tone: 'ready' });
    // A code sent only as the error code still counts.
    expect(itemReasonCodes(review({ errorCode: 'PASSWORD_PROTECTED' }))).toEqual(['PASSWORD_PROTECTED']);
    expect(checkText(review({ errorCode: 'PASSWORD_PROTECTED' }))).toBe('Password-protected');
    // Free text is its own short form.
    expect(checkText(review({ reason: 'Canceled.' }))).toBe('Canceled.');
    // Nothing yet: waiting, or being read.
    expect(checkCell({ ...ambiguous, status: 'waiting' })).toBeUndefined();
    expect(checkCell({ ...ambiguous, status: 'processing' })).toBeUndefined();
  });

  it('flags only the parts of a name in review', () => {
    expect(itemFlaggedFields(review({ reasonCodes: ['DATE_AMBIGUOUS', 'PARTY_UNSUPPORTED', 'LOW_CONFIDENCE'] }))).toEqual(['date', 'parties']);
    expect(itemFlaggedFields({ ...review({ reasonCodes: ['DATE_AMBIGUOUS'] }), status: 'ready' })).toEqual([]);
  });
});

/*
  At the Balanced review level a reading can be ready while it carries review
  codes, and a document Intern then renames by itself is filed with them. On
  such a row they are notes: it reads as ready, or completed, with a note -
  never as a review, and never as having passed every check.
*/
describe('a note on a row that is not waiting', () => {
  const ready = (extra: Partial<QueueItem>): QueueItem => ({ ...review({}), status: 'ready', ...extra });
  const renamed = (extra: Partial<QueueItem>): QueueItem => ({ ...review({}), status: 'completed', filedName: '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', undoable: true, keptOriginal: false, ...extra });
  const codes = ['TYPE_INFERRED', 'LOW_CONFIDENCE'];
  const sent = { reasonCodes: codes, reason: humanizeReason(codes.join(', ')) };

  it('is what a ready row with codes says in place of "All checks passed"', () => {
    const item = ready(sent);
    expect(itemNote(item)).toEqual({ codes, text: describeNotes(codes) });
    expect(checkCell(item)).toEqual({ text: 'Note: Type supplied by Intern', more: 1, tone: 'noted', title: describeNotes(codes) });
    // Copied, the cell still says the row was not waiting.
    expect(checkText(item)).toBe('Note: Type supplied by Intern (+1 more)');
    // The tooltip is the note, not the review's instructions, and not the claim that every check passed.
    expect(checkCell(item)?.title).not.toMatch(/Check |each found in the document/);
    // Nothing of its name is marked for checking: that is a review's.
    expect(itemFlaggedFields(item)).toEqual([]);
  });

  it('leaves a ready row with no reason exactly as it was', () => {
    expect(itemNote(ready({}))).toBeUndefined();
    expect(checkCell(ready({}))).toEqual({ text: 'All checks passed', more: 0, tone: 'ready', title: 'The date, type and parties in this name were each found in the document.' });
  });

  it('is not said of an approved name, of a copy, or of a row in review', () => {
    // An approved name is the person's own.
    expect(itemNote(ready({ ...sent, approved: true }))).toBeUndefined();
    expect(checkCell(ready({ ...sent, approved: true }))).toMatchObject({ text: 'Approved', tone: 'ready' });
    // A staged copy says what it is.
    const copy = ready({ copyOf: '2024-02-09 Invoice.pdf', approved: true, reason: humanizeReason('EXACT_COPY'), reasonCodes: ['EXACT_COPY'] });
    expect(itemNote(copy)).toBeUndefined();
    expect(itemNote({ ...copy, approved: false })).toBeUndefined();
    expect(checkCell({ ...copy, approved: false })).toMatchObject({ text: 'Exact copy', tone: 'quiet' });
    // In review, the same codes are what to check.
    expect(itemNote(review(sent))).toBeUndefined();
    expect(checkCell(review(sent))).toMatchObject({ text: 'Type supplied by Intern', more: 1, tone: 'review' });
    for (const status of ['waiting', 'processing', 'failed'] as const) expect(itemNote({ ...review(sent), status }), status).toBeUndefined();
  });

  it('is said in the words that came when a ready row gives a reason this build has no codes for', () => {
    const item = ready({ reason: 'TYPE_INFERRED, SOMETHING_NEW' });
    expect(itemNote(item)).toEqual({ codes: [], text: 'TYPE_INFERRED, SOMETHING_NEW' });
    expect(checkCell(item)).toEqual({ text: 'Note: TYPE_INFERRED, SOMETHING_NEW', more: 0, tone: 'noted', title: 'TYPE_INFERRED, SOMETHING_NEW' });
  });

  it('marks a document renamed while it carried codes, and not one that kept its name or was a copy', () => {
    const item = renamed(sent);
    expect(itemNote(item)).toEqual({ codes, text: describeNotes(codes) });
    expect(checkCell(item)).toEqual({ text: 'Note: Type supplied by Intern', more: 1, tone: 'noted', title: describeNotes(codes) });
    // Renamed with no codes: nothing to say, as before.
    expect(itemNote(renamed({}))).toBeUndefined();
    expect(checkCell(renamed({}))).toBeUndefined();
    // Kept its original name with a review's codes still on it: not renamed by them, and shown as it always was.
    const kept = renamed({ ...sent, keptOriginal: true, filedName: undefined, undoable: false });
    expect(itemNote(kept)).toBeUndefined();
    expect(checkCell(kept)).toEqual({ text: 'Type supplied by Intern', more: 1, tone: 'quiet', title: sent.reason });
    // What became of the file is no note about its reading.
    const named = renamed({ keptOriginal: true, filedName: undefined, undoable: false, reasonCodes: ['ALREADY_NAMED'], reason: humanizeReason('ALREADY_NAMED') });
    expect(itemNote(named)).toBeUndefined();
    expect(checkCell(named)).toMatchObject({ text: 'Already had this name', tone: 'quiet' });
    const copy = renamed({ copyOf: '2024-02-09 Invoice.pdf', reasonCodes: ['EXACT_COPY'], reason: humanizeReason('EXACT_COPY') });
    expect(itemNote(copy)).toBeUndefined();
    expect(checkCell(copy)).toMatchObject({ text: 'Exact copy', tone: 'quiet' });
    // A filed row gives its reason in codes or not at all: words alone mark nothing.
    expect(itemNote(renamed({ reason: 'Something in words.' }))).toBeUndefined();
  });
});

describe('a proposed name in parts', () => {
  it('splits a composed name into date, type and parties', () => {
    const parts = nameParts('2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', ['parties']);
    expect(parts.map((part) => part.text).join('')).toBe('2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf');
    expect(parts.filter((part) => part.flagged).map((part) => part.text)).toEqual(['Fabrikam Inc']);
    expect(parts.find((part) => part.field === 'type')?.text).toBe('Invoice INV-3301');
    expect(parts.find((part) => part.field === 'date')?.text).toBe('2024-02-09');
  });

  it('marks the date of a name with no parties, and nothing of a name with no date', () => {
    expect(nameParts('2024-02-09 Board Minutes.docx', ['date', 'parties']).filter((part) => part.flagged).map((part) => part.text)).toEqual(['2024-02-09']);
    const typed = nameParts('Lease for 12 Main St.pdf', ['date', 'type', 'parties']);
    expect(typed.some((part) => part.flagged)).toBe(false);
    expect(typed.map((part) => part.text).join('')).toBe('Lease for 12 Main St.pdf');
  });
});

describe('another party put in a name', () => {
  /** The offer for a proposed stem whose parties the backend read as `nameParties`, with `choices` beside it. */
  const offer = (proposed: string, nameParties: string[], choices: string[] = []) => ({ proposed, nameParties, choices });

  it('takes the place of the party the name gives, however its type reads', () => {
    const letter = offer('2012-06-19 Letter to U.S. Department of Justice', ['U.S. Department of Justice']);
    expect(withParty(letter.proposed, 'Morgan, Lewis & Bockius LLP', letter)).toBe('2012-06-19 Letter to Morgan, Lewis & Bockius LLP');
    // A type with a connecting word of its own keeps it: the backend said which words are the party.
    const application = offer('2007-04-24 Application for Certification for City of Victorville', ['City of Victorville']);
    expect(withParty(application.proposed, 'California Energy Commission', application))
      .toBe('2007-04-24 Application for Certification for California Energy Commission');
    // A name with no date yet is still a name with a party.
    const undated = offer('Letter to Carl Malamud', ['Carl Malamud']);
    expect(withParty(undated.proposed, 'Department of the Air Force', undated)).toBe('Letter to Department of the Air Force');
  });

  it('replaces the second side of a "between", and an earlier choice', () => {
    const notice = offer('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and WPP Group PLC', ['PA Advisors, LLC', 'WPP Group PLC'], ['Google Inc', 'Research In Motion Ltd']);
    const once = withParty(notice.proposed, 'Google Inc', notice);
    expect(once).toBe('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Google Inc');
    expect(withParty(once, 'Research In Motion Ltd', notice))
      .toBe('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Research In Motion Ltd');
    // Choosing the party already there changes nothing.
    expect(withParty(once, 'Google Inc', notice)).toBe(once);
  });

  it('leaves off a collision suffix, which the rename gives again if the new name needs one', () => {
    const application = offer('2007-04-24 Application for Certification for City of Victorville (2)', ['City of Victorville']);
    expect(withParty(application.proposed, 'California Energy Commission', application))
      .toBe('2007-04-24 Application for Certification for California Energy Commission');
    const agreement = offer('2024-01-05 Agreement between Acme Corporation and Beacon LLC (2)', ['Acme Corporation', 'Beacon LLC']);
    expect(withParty(agreement.proposed, 'Contoso Inc', agreement)).toBe('2024-01-05 Agreement between Acme Corporation and Contoso Inc');
  });

  it('adds a party to a name that gave none after its whole type, with no type word taken for one', () => {
    // A party left out for length, from a type holding "with" (real2-notice-12).
    const notice = offer('2022-03-17 Notice in Accordance with Section 57C of the Railways Act 1993', []);
    expect(withParty(notice.proposed, 'Department for Transport', notice))
      .toBe('2022-03-17 Notice in Accordance with Section 57C of the Railways Act 1993 - Department for Transport');
    // A party missing from the reading, from a type holding "for".
    const request = offer('2024-01-05 Request for Production', []);
    expect(withParty(request.proposed, 'Acme Corporation', request)).toBe('2024-01-05 Request for Production - Acme Corporation');
    const invoice = offer('2023-10-05 Requester Invoice', [], ['Sam Brownell', 'Litware Inc']);
    const added = withParty(invoice.proposed, 'Sam Brownell', invoice);
    expect(added).toBe('2023-10-05 Requester Invoice - Sam Brownell');
    // A second click replaces the first, and a connecting word the reviewer typed stays.
    expect(withParty(added, 'Litware Inc', invoice)).toBe('2023-10-05 Requester Invoice - Litware Inc');
    expect(withParty('2023-10-05 Requester Invoice from Sam', 'Litware Inc', invoice)).toBe('2023-10-05 Requester Invoice from Litware Inc');
    expect(withParty('', 'Sam Brownell', offer('', []))).toBe('Sam Brownell');
  });

  it('replaces a party typed over by hand after the proposed type, whatever its connecting word', () => {
    const invoice = offer('2024-02-09 Invoice INV-3301 from Fabrikam Inc.', ['Fabrikam Inc.'], ['Fabrikam Inc']);
    expect(withParty('2024-02-09 Invoice INV-3301 from Fabrikm', 'Fabrikam Inc', invoice)).toBe('2024-02-09 Invoice INV-3301 from Fabrikam Inc');
    const application = offer('2007-04-24 Application for Certification for City of Victorville', ['City of Victorville'], ['California Energy Commission']);
    // A date chosen and the connecting word changed are kept; the type's own "for" is no connecting word.
    expect(withParty('2007-05-01 Application for Certification to Cty of Victorvile', 'California Energy Commission', application))
      .toBe('2007-05-01 Application for Certification to California Energy Commission');
    // A name rewritten past the proposed type keeps every word typed.
    expect(withParty('2007-04-24 Petition for Review for City of Victorvile', 'California Energy Commission', application))
      .toBe('2007-04-24 Petition for Review for City of Victorvile - California Energy Commission');
  });

  it('keeps words typed after the party it replaces', () => {
    const letter = offer('2012-06-19 Request Letter to U.S. Department of Justice', ['U.S. Department of Justice'], ['Morgan, Lewis & Bockius LLP', 'STARS Alliance LLC']);
    const once = withParty('2012-06-19 Request Letter to U.S. Department of Justice re Merger', 'Morgan, Lewis & Bockius LLP', letter);
    expect(once).toBe('2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP re Merger');
    // After a party an earlier click put in, and with the connecting word changed.
    expect(withParty(once, 'STARS Alliance LLC', letter)).toBe('2012-06-19 Request Letter to STARS Alliance LLC re Merger');
    expect(withParty('2012-06-19 Request Letter from U.S. Department of Justice re Merger', 'STARS Alliance LLC', letter))
      .toBe('2012-06-19 Request Letter from STARS Alliance LLC re Merger');
    // After the second side of a "between".
    const notice = offer('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and WPP Group PLC', ['PA Advisors, LLC', 'WPP Group PLC'], ['Google Inc']);
    expect(withParty('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and WPP Group PLC (signed)', 'Google Inc', notice))
      .toBe('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Google Inc (signed)');
    // A party typed over by hand says nowhere where it ends: it goes, and what follows it.
    expect(withParty('2012-06-19 Request Letter to US Dept of Justice re Merger', 'STARS Alliance LLC', letter))
      .toBe('2012-06-19 Request Letter to STARS Alliance LLC');
  });

  /*
    A known party at the start of the party's words says where the party
    starts, not where it ends. "Smith" completed by hand to "Smith and Sons,
    Inc." is a party, and the words after "Smith" are the rest of it: kept,
    they gave "Letter to Jones LLP and Sons, Inc.", one Enter from filing,
    and a pressed chip for "Smith" that changed nothing when clicked.
  */
  it('replaces a party lengthened by hand whole, and keeps only a note typed after a known one', () => {
    const smith = offer('2020-01-02 Letter to Smith', ['Smith'], ['Jones LLP']);
    expect(withParty('2020-01-02 Letter to Smith and Sons, Inc.', 'Jones LLP', smith)).toBe('2020-01-02 Letter to Jones LLP');
    expect(withParty('2020-01-02 Letter to Smith & Sons re: Merger', 'Jones LLP', smith)).toBe('2020-01-02 Letter to Jones LLP');
    const court = offer('2020-01-02 Notice to Court', ['Court'], ['State of Ohio']);
    expect(withParty('2020-01-02 Notice to Court of Appeals', 'State of Ohio', court)).toBe('2020-01-02 Notice to State of Ohio');
    const notice = offer('2020-01-02 Notice between A Corp and B Inc', ['A Corp', 'B Inc'], ['C LLC']);
    expect(withParty('2020-01-02 Notice between A Corp and B Inc Holdings', 'C LLC', notice)).toBe('2020-01-02 Notice between A Corp and C LLC');
    // A party an earlier click put in, lengthened by hand.
    const invoice = offer('2023-10-05 Invoice', [], ['Acme', 'Beta LLC']);
    expect(withParty('2023-10-05 Invoice - Acme Corp', 'Beta LLC', invoice)).toBe('2023-10-05 Invoice - Beta LLC');
    // Only the longest known party is read: not "Smith" and a note " (UK) Ltd".
    const uk = offer('2020-01-02 Letter to Smith', ['Smith'], ['Smith (UK)', 'Jones LLP']);
    expect(withParty('2020-01-02 Letter to Smith (UK) Ltd', 'Jones LLP', uk)).toBe('2020-01-02 Letter to Jones LLP');
    expect(partyGiven('2020-01-02 Letter to Smith (UK) Ltd', uk)).toBeUndefined();
    // No chip is pressed for the known words the party starts with.
    expect(partyGiven('2020-01-02 Letter to Smith and Sons, Inc.', smith)).toBeUndefined();
    expect(partyGiven('2023-10-05 Invoice - Acme Corp', invoice)).toBeUndefined();
    // A note about the document after a known party still stays, and its chip is pressed.
    for (const note of [' re Merger', ' Re: Merger', ' regarding the Merger', ' about Merger', ' concerning Merger', ' - signed', ' (signed)']) {
      expect(withParty(`2020-01-02 Letter to Smith${note}`, 'Jones LLP', smith), note).toBe(`2020-01-02 Letter to Jones LLP${note}`);
      expect(partyGiven(`2020-01-02 Letter to Smith${note}`, smith), note).toBe('Smith');
    }
  });

  it('reads the party a name gives as withParty does, words typed after it or not', () => {
    const letter = offer('2012-06-19 Request Letter to U.S. Department of Justice', ['U.S. Department of Justice'], ['Morgan, Lewis & Bockius LLP']);
    expect(partyGiven('2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP (2)', letter)).toBe('Morgan, Lewis & Bockius LLP');
    expect(partyGiven('2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP re Merger', letter)).toBe('Morgan, Lewis & Bockius LLP');
    expect(partyGiven('2012-06-19 Request Letter to U.S. Department of Justice re Merger', letter)).toBe('U.S. Department of Justice');
    expect(partyGiven('2012-06-19 Request Letter to US Dept of Justice re Merger', letter)).toBeUndefined();
    expect(partyGiven('2012-06-19 Petition to Morgan, Lewis & Bockius LLP re Merger', letter)).toBeUndefined();
  });
});
