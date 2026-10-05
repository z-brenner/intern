import { describe, expect, it } from 'vitest';
import snapshotSource from '../../crates/intern-core/src/snapshot.rs?raw';
import { FILE_KINDS, SUPPORTED_FORMATS_LABEL, fileKind } from './formats';

/** The extensions intern-core admits, read from the one list in its source. */
function admittedExtensions(): string[] {
  const list = /pub const SUPPORTED_EXTENSIONS: &\[&str\] = &\[([^\]]*)\];/.exec(snapshotSource);
  if (!list) throw new Error('SUPPORTED_EXTENSIONS was not found in intern-core');
  return [...list[1].matchAll(/"([a-z0-9]+)"/g)].map((match) => match[1]);
}

describe('supported formats', () => {
  it('names exactly the extensions the backend admits', () => {
    const admitted = admittedExtensions();
    expect(admitted).toContain('msg');
    expect(Object.keys(FILE_KINDS).sort()).toEqual([...admitted].sort());
  });

  it('tells people about every family of format the backend reads', () => {
    for (const named of ['PDF', 'Word', '.doc', '.rtf', '.odt', 'Excel', '.xls', '.ods', '.csv', 'PowerPoint', '.ppt', '.odp', 'Outlook .msg', '.eml', 'Markdown', 'TIFF']) {
      expect(SUPPORTED_FORMATS_LABEL).toContain(named);
    }
  });

  it('gives each file the kind of its extension, whatever its case', () => {
    expect(fileKind('Board deck.PPTX')).toBe('presentation');
    expect(fileKind('slides.odp')).toBe('presentation');
    expect(fileKind('Re: invoice.msg')).toBe('email');
    expect(fileKind('notice.eml')).toBe('email');
    expect(fileKind('letter.doc')).toBe('document');
    expect(fileKind('letter.odt')).toBe('document');
    expect(fileKind('ledger.xlsm')).toBe('spreadsheet');
    expect(fileKind('ledger.ods')).toBe('spreadsheet');
    expect(fileKind('statement.csv')).toBe('text');
    expect(fileKind('memo.rtf')).toBe('text');
    expect(fileKind('notes.markdown')).toBe('text');
    expect(fileKind('archive.zip')).toBeUndefined();
    expect(fileKind('README')).toBeUndefined();
    // Not an extension at all, only the name of something every object has.
    expect(fileKind('weird.constructor')).toBeUndefined();
  });
});
