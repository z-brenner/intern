import { describe, expect, it } from 'vitest';
import { carriedNotes, checkCell, checkText, itemFlaggedFields, itemNote, itemReasonCodes, nameParts, partyGiven, withParty } from './check';
import { describeNotes, humanizeReason, reasonCodes, reasonLines, shortReason } from '../../lib/reasons';
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
    expect(shortReason(['LOW_CONFIDENCE', 'PARTY_UNSUPPORTED'])).toEqual({ text: 'Party unclear', more: 1 });
    expect(shortReason(['LOW_CONFIDENCE'])).toEqual({ text: 'Model unsure', more: 0 });
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
    expect(shortReason(['LOW_CONFIDENCE', 'DATE_IS_PERIOD'])).toEqual({ text: 'Date is a period end', more: 1 });
    expect(itemFlaggedFields(review({ reasonCodes: ['DATE_IS_PERIOD'] }))).toEqual(['date']);
    // So does a date that only a TODAY or NOW formula shows.
    expect(shortReason(['LOW_CONFIDENCE', 'DATE_IS_FORMULA'])).toEqual({ text: 'Date from a formula', more: 1 });
    expect(itemFlaggedFields(review({ reasonCodes: ['DATE_IS_FORMULA'] }))).toEqual(['date']);
  });

  it('says what to check for a review, and that the checks passed for a ready row', () => {
    const ambiguous = review({ reasonCodes: ['DATE_AMBIGUOUS'], reason: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.' });
    // The tooltip names the reason in its few words, then says why.
    expect(checkCell(ambiguous)).toEqual({ text: 'Date order unclear', more: 0, tone: 'review', title: `Date order unclear: ${ambiguous.reason}` });
    // Every reason by name, those counted as "+1 more" too.
    expect(checkCell(review({ reasonCodes: ['LOW_CONFIDENCE', 'PARTY_UNSUPPORTED'], reason: humanizeReason('LOW_CONFIDENCE, PARTY_UNSUPPORTED') }))?.title).toBe(`Model unsure: ${humanizeReason('LOW_CONFIDENCE')}\nParty unclear: ${humanizeReason('PARTY_UNSUPPORTED')}`);
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
    expect(itemNote(item)).toEqual({ codes, text: reasonLines(codes, { note: true }) });
    expect(itemNote(item)?.text).toBe(`Type guessed: ${describeNotes(['TYPE_INFERRED'])}\nModel unsure: ${describeNotes(['LOW_CONFIDENCE'])}`);
    expect(checkCell(item)).toEqual({ text: 'Note: Type guessed', more: 1, tone: 'noted', title: reasonLines(codes, { note: true }) });
    // Copied, the cell still says the row was not waiting.
    expect(checkText(item)).toBe('Note: Type guessed (+1 more)');
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
    expect(checkCell(review(sent))).toMatchObject({ text: 'Type guessed', more: 1, tone: 'review' });
    for (const status of ['waiting', 'processing', 'failed'] as const) expect(itemNote({ ...review(sent), status }), status).toBeUndefined();
  });

  it('is said in the words that came when a ready row gives a reason this build has no codes for', () => {
    const item = ready({ reason: 'TYPE_INFERRED, SOMETHING_NEW' });
    expect(itemNote(item)).toEqual({ codes: [], text: 'TYPE_INFERRED, SOMETHING_NEW' });
    expect(checkCell(item)).toEqual({ text: 'Note: TYPE_INFERRED, SOMETHING_NEW', more: 0, tone: 'noted', title: 'TYPE_INFERRED, SOMETHING_NEW' });
  });

  it('marks a document renamed while it carried codes, and not one that kept its name or was a copy', () => {
    const item = renamed(sent);
    expect(itemNote(item)).toEqual({ codes, text: reasonLines(codes, { note: true }) });
    expect(itemNote(item)?.text).toBe(`Type guessed: ${describeNotes(['TYPE_INFERRED'])}\nModel unsure: ${describeNotes(['LOW_CONFIDENCE'])}`);
    expect(checkCell(item)).toEqual({ text: 'Note: Type guessed', more: 1, tone: 'noted', title: reasonLines(codes, { note: true }) });
    // Renamed with no codes: nothing to say, as before.
    expect(itemNote(renamed({}))).toBeUndefined();
    expect(checkCell(renamed({}))).toBeUndefined();
    // Kept its original name with a review's codes still on it: not renamed by them, and shown as it always was.
    const kept = renamed({ ...sent, keptOriginal: true, filedName: undefined, undoable: false });
    expect(itemNote(kept)).toBeUndefined();
    expect(checkCell(kept)).toEqual({ text: 'Type guessed', more: 1, tone: 'quiet', title: reasonLines(codes) });
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

  /*
    The engine raises TYPE_MISSING for a type that is a general word alone -
    "Letter", "Agreement", "Notice" - and the Balanced level makes such a
    reading ready only when its name has a type. "No document type" beside
    "Letter to Fabrikam Inc" said the name lacked what it had, and that
    Intern had made ready what Settings says waits.
  */
  it('says of a type that is a general word that it is not specific, never that the name has none', () => {
    const name = '2024-03-08 Letter to Fabrikam Inc.pdf';
    const general = { proposedFilename: name, reasonCodes: ['TYPE_MISSING'], reason: humanizeReason('TYPE_MISSING') };
    for (const item of [ready(general), renamed({ ...general, filedName: name })]) {
      expect(itemNote(item), item.status).toEqual({ codes: ['TYPE_MISSING'], text: 'No specific type: No specific document type was identified.' });
      expect(checkCell(item), item.status).toEqual({ text: 'Note: No specific type', more: 0, tone: 'noted', title: 'No specific type: No specific document type was identified.' });
      // As it is copied, and as Rename history marks it.
      expect(checkText(item), item.status).toBe('Note: No specific type');
    }
    // Leading a list, and counted behind another note about the type.
    expect(checkText(ready({ ...general, reasonCodes: ['TYPE_MISSING', 'LOW_CONFIDENCE'] }))).toBe('Note: No specific type (+1 more)');
    expect(checkText(ready({ ...general, reasonCodes: ['TYPE_INFERRED', 'TYPE_MISSING'] }))).toBe('Note: Type guessed (+1 more)');
    // No note says the name has no type, whatever it carries.
    for (const carried of [['TYPE_MISSING'], ['TYPE_MISSING', 'TYPE_INFERRED'], ['LOW_CONFIDENCE', 'TYPE_MISSING'], ['DESCRIPTION_UNSUPPORTED', 'TYPE_MISSING']]) {
      expect(checkText(ready({ ...general, reasonCodes: carried })), carried.join()).not.toMatch(/No document type/);
    }
    // In review the few words are as they were: there the name may well have none.
    expect(checkCell(review(general))).toEqual({ text: 'No document type', more: 0, tone: 'review', title: `No document type: ${general.reason}` });
    // And on a document a person kept under its own name.
    expect(checkCell(renamed({ ...general, keptOriginal: true, filedName: undefined, undoable: false }))).toEqual({ text: 'No document type', more: 0, tone: 'quiet', title: `No document type: ${general.reason}` });
  });
});

/*
  Two states a document with notes passes through without being ready or
  renamed. While Intern renames it, it is being renamed. And where it
  already had exactly the name Intern would give it, Intern completes it
  under that name: the backend adds ALREADY_NAMED to the reasons it had (an
  approval clears them first, so ALREADY_NAMED beside other codes is always
  Intern's own doing). Neither waits for anyone, so neither is told to check
  anything.
*/
describe('the notes a document carries while it is renamed, or completed under the name it had', () => {
  const codes = ['TYPE_INFERRED', 'LOW_CONFIDENCE'];
  const name = '2024-03-08 Letter to Fabrikam Inc.pdf';
  const filing: QueueItem = { id: 'f', originalFilename: 'Scan 3.pdf', status: 'processing', stage: 'filing', cancelable: false, proposedFilename: name, reasonCodes: codes, reason: humanizeReason(codes.join(', ')) };
  const named: QueueItem = { id: 'n', originalFilename: name, status: 'completed', proposedFilename: name, keptOriginal: true, undoable: false, reasonCodes: [...codes, 'ALREADY_NAMED'], reason: humanizeReason([...codes, 'ALREADY_NAMED'].join(', ')) };
  const alreadyNamed = 'This document already had this name, so nothing was renamed.';

  it('are said as notes, with no instruction', () => {
    expect(humanizeReason(codes.join(', '))).toMatch(/Check that it names the document\. .*Check the name against the document/);
    expect(carriedNotes(filing)).toBe(describeNotes(codes));
    expect(carriedNotes(filing)).not.toMatch(/(^|\. )Check\b/);
    // What became of the file first, then what Intern had noted.
    expect(carriedNotes(named)).toBe(`${alreadyNamed} ${describeNotes(codes)}`);
    expect(carriedNotes(named)).not.toMatch(/(^|\. )Check\b/);
  });

  it('leave a document being renamed without a Check cell, and lead a completed one\'s with what became of the file', () => {
    expect(checkCell(filing)).toBeUndefined();
    expect(itemNote(filing)).toBeUndefined();
    // Not renamed, so not among those renamed with a note; the notes are counted behind the outcome.
    expect(itemNote(named)).toBeUndefined();
    expect(checkCell(named)).toEqual({ text: 'Already had this name', more: 2, tone: 'quiet', title: `Already had this name: ${alreadyNamed}\n${reasonLines(codes, { note: true })}` });
    expect(checkText(named)).toBe('Already had this name (+2 more)');
  });

  it('change nothing where the reason stands as it came', () => {
    // Already named with nothing else, as an approval or the Strict level leaves it.
    const plain: QueueItem = { ...named, reasonCodes: ['ALREADY_NAMED'], reason: alreadyNamed };
    expect(carriedNotes(plain)).toBe(alreadyNamed);
    expect(checkCell(plain)).toEqual({ text: 'Already had this name', more: 0, tone: 'quiet', title: `Already had this name: ${alreadyNamed}` });
    // Kept under its own name by a person: the review's codes and words, as before.
    const kept: QueueItem = { ...named, reasonCodes: codes, reason: humanizeReason(codes.join(', ')) };
    expect(carriedNotes(kept)).toBeUndefined();
    expect(checkCell(kept)).toEqual({ text: 'Type guessed', more: 1, tone: 'quiet', title: reasonLines(codes) });
    // In review, ready, renamed, waiting, or being read again: said elsewhere, or as it came.
    for (const status of ['review', 'failed', 'ready', 'waiting'] as const) expect(carriedNotes({ ...filing, status, stage: undefined }), status).toBeUndefined();
    for (const stage of ['reading', 'naming'] as const) expect(carriedNotes({ ...filing, stage }), stage).toBeUndefined();
    expect(carriedNotes({ ...named, keptOriginal: false, filedName: name, reasonCodes: codes, reason: filing.reason })).toBeUndefined();
    // Words with no codes behind them are nobody's to reword.
    expect(carriedNotes({ ...filing, reasonCodes: undefined, reason: 'Something in words.' })).toBeUndefined();
    expect(carriedNotes({ ...filing, reasonCodes: undefined, reason: undefined })).toBeUndefined();
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

  /** The parts of `name` flagged when every field is, by field, in the order the name reads. */
  const fields = (name: string) => nameParts(name, ['date', 'type', 'parties'])
    .filter((part) => part.field)
    .map((part) => [part.field, part.text]);

  it('reads the organization a name leads with as one of its parties', () => {
    const name = '2026-10-16 Vistage International - Host Chair SOW for John Smith.pdf';
    expect(nameParts(name, []).map((part) => part.text).join('')).toBe(name);
    expect(fields(name)).toEqual([
      ['date', '2026-10-16'],
      ['parties', 'Vistage International'],
      ['type', 'Host Chair SOW'],
      ['parties', 'John Smith'],
    ]);
    expect(nameParts(name, ['parties']).filter((part) => part.flagged).map((part) => part.text))
      .toEqual(['Vistage International', 'John Smith']);
    expect(nameParts(name, ['type']).filter((part) => part.flagged).map((part) => part.text)).toEqual(['Host Chair SOW']);
    // No other side.
    expect(fields('2024-11-09 EGN Denmark - Incorporation Email Communications.pdf')).toEqual([
      ['date', '2024-11-09'],
      ['parties', 'EGN Denmark'],
      ['type', 'Incorporation Email Communications'],
    ]);
    expect(fields('2025-04-30 Nimbus Orchard Supply - Invoice.pdf')).toEqual([
      ['date', '2025-04-30'],
      ['parties', 'Nimbus Orchard Supply'],
      ['type', 'Invoice'],
    ]);
    // A party given no relation after it, and two sides of a "between".
    expect(fields('2024-04-12 Acme - Notice of Termination - John Smith.pdf')).toEqual([
      ['date', '2024-04-12'],
      ['parties', 'Acme'],
      ['type', 'Notice of Termination'],
      ['parties', 'John Smith'],
    ]);
    expect(fields('2024-04-12 Wingtip - Agreement between John Smith and Acme Corporation (2).pdf')).toEqual([
      ['date', '2024-04-12'],
      ['parties', 'Wingtip'],
      ['type', 'Agreement'],
      ['parties', 'John Smith and Acme Corporation (2)'],
    ]);
  });

  it('reads a name with no organization as the type and the party after it', () => {
    // A document naming only people: the side that names a kind of document is the type.
    expect(fields('2026-12-29 Notice of Termination - John Smith.pdf')).toEqual([
      ['date', '2026-12-29'],
      ['type', 'Notice of Termination'],
      ['parties', 'John Smith'],
    ]);
    expect(fields('2024-02-09 Letter to Smith - signed.pdf')).toEqual([
      ['date', '2024-02-09'],
      ['type', 'Letter'],
      ['parties', 'Smith - signed'],
    ]);
    expect(fields('2025-01-05 Settlement Agreement between John Smith and Jane Doe.pdf')).toEqual([
      ['date', '2025-01-05'],
      ['type', 'Settlement Agreement'],
      ['parties', 'John Smith and Jane Doe'],
    ]);
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

  it('takes the place of the other side after the organization the name leads with, which stays', () => {
    const sow = offer('2026-10-16 Vistage International - Host Chair SOW for John Smith', ['John Smith'], ['Jane Doe']);
    expect(withParty(sow.proposed, 'Jane Doe', sow)).toBe('2026-10-16 Vistage International - Host Chair SOW for Jane Doe');
    expect(partyGiven(sow.proposed, sow)).toBe('John Smith');
    // A note typed after the party stays, and so does the organization respelled by hand.
    expect(withParty('2026-10-16 Vistage International - Host Chair SOW for John Smith re Renewal', 'Jane Doe', sow))
      .toBe('2026-10-16 Vistage International - Host Chair SOW for Jane Doe re Renewal');
    expect(withParty('2026-10-16 Vistage Intl - Host Chair SOW for John Smith', 'Jane Doe', sow))
      .toBe('2026-10-16 Vistage Intl - Host Chair SOW for Jane Doe');
    // One that gives no other side is given one after its type, as a party with no relation.
    const invoice = offer('2025-04-30 Nimbus Orchard Supply - Invoice', [], ['Contoso Ltd']);
    const added = withParty(invoice.proposed, 'Contoso Ltd', invoice);
    expect(added).toBe('2025-04-30 Nimbus Orchard Supply - Invoice - Contoso Ltd');
    expect(partyGiven(added, invoice)).toBe('Contoso Ltd');
    expect(partyGiven(invoice.proposed, invoice)).toBeUndefined();
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
