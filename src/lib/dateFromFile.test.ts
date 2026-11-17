import { describe, expect, it } from 'vitest';
import { humanizeReason, reasonCodes, reasonField } from './reasons';

// A reading the document gave no date is dated from the file itself - the
// original filename's date, or else the file's last-modified date - and its
// record says which in the place of DATE_MISSING (fallback_date.rs in the
// queue). The note is how a person finds such a name afterwards.
describe('a date taken from the file', () => {
  it('says where the date came from, in one short sentence', () => {
    expect(humanizeReason('DATE_FROM_FILENAME')).toBe('Date taken from the original filename.');
    expect(humanizeReason('DATE_FROM_FILE_MODIFIED')).toBe('Date taken from the file\'s last-modified date.');
  });

  it('is a code among the others, about the date', () => {
    expect(reasonCodes('DATE_FROM_FILE_MODIFIED, PARTY_UNSUPPORTED')).toEqual(['DATE_FROM_FILE_MODIFIED', 'PARTY_UNSUPPORTED']);
    expect(reasonField('DATE_FROM_FILENAME')).toBe('date');
    expect(reasonField('DATE_FROM_FILE_MODIFIED')).toBe('date');
  });
});
