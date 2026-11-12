import { describe, expect, it } from 'vitest';
import { checkCell, checkText, itemFlaggedFields, itemReasonCodes, nameParts, withParty } from './check';
import { reasonCodes, shortReason } from '../../lib/reasons';
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
      'DATE_IS_DEADLINE', 'DATE_AMBIGUOUS', 'FAST_MODEL_READING',
    ];
    for (const code of codes) expect(shortReason([code]), code).toBeDefined();
    expect(shortReason(['PARTY_MISSING'])).toEqual({ text: 'Party missing', more: 0 });
    expect(itemFlaggedFields(review({ reasonCodes: ['PARTY_MISSING'] }))).toEqual(['parties']);
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
});
