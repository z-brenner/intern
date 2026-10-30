import { describe, expect, it } from 'vitest';
import { checkCell, checkText, itemFlaggedFields, itemReasonCodes, nameParts } from './check';
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
    expect(shortReason(['LOW_CONFIDENCE', 'PARTY_UNSUPPORTED'])).toEqual({ text: 'Party not found', more: 1 });
    expect(shortReason(['LOW_CONFIDENCE'])).toEqual({ text: 'Model unsure of its reading', more: 0 });
    expect(shortReason(['NOT_A_CODE'])).toBeUndefined();
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
