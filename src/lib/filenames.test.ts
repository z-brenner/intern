import { describe, expect, it } from 'vitest';
import { filenameExtension, joinFilename, leadingDate, splitFilename, validateFilename, withLeadingDate } from './filenames';

describe('leadingDate', () => {
  it('finds a real date standing at the start of the name', () => {
    expect(leadingDate('2026-03-02 Invoice from Acme.pdf')).toBe('2026-03-02');
    expect(leadingDate('  2026-03-02.pdf')).toBe('2026-03-02');
    expect(leadingDate('2026-03-02-invoice.pdf')).toBe('2026-03-02');
  });

  it('refuses a missing, buried, run-on, or impossible date', () => {
    expect(leadingDate('Invoice from Acme.pdf')).toBeUndefined();
    expect(leadingDate('Invoice 2026-03-02 from Acme.pdf')).toBeUndefined();
    expect(leadingDate('2026-03-021 Invoice.pdf')).toBeUndefined();
    expect(leadingDate('2026-02-30 Invoice.pdf')).toBeUndefined();
    expect(leadingDate('')).toBeUndefined();
  });
});

describe('withLeadingDate', () => {
  it('prepends a date, replacing one already there', () => {
    expect(withLeadingDate('Invoice from Acme.pdf', '2026-03-02')).toBe('2026-03-02 Invoice from Acme.pdf');
    expect(withLeadingDate('2025-01-01 Invoice from Acme.pdf', '2026-03-02')).toBe('2026-03-02 Invoice from Acme.pdf');
    expect(withLeadingDate('2025-01-01 - Invoice.pdf', '2026-03-02')).toBe('2026-03-02 Invoice.pdf');
    expect(withLeadingDate('   ', '2026-03-02')).toBe('2026-03-02');
  });
});

// pipeline.rs validate_leaf_filename and the extension rule in
// Pipeline::approve. Every name refused here is one the backend refuses with
// "filename must be one nonblank path component" or "approved filename must
// preserve the source extension"; the sentence here names what is wrong.
describe('validateFilename', () => {
  const valid = '2024-03-01 Lease Agreement with Acme Corp.pdf';

  it('validate_filename_mirrors_backend', () => {
    expect(validateFilename(valid, 'pdf')).toBeUndefined();
    // Each character Windows reserves, and both separators, by name.
    for (const character of ['<', '>', ':', '"', '|', '?', '*']) {
      expect(validateFilename(`2024-03-01 Lease ${character} Acme.pdf`, 'pdf')).toBe(`\u201c${character}\u201d cannot be used in a Windows filename.`);
    }
    for (const character of ['/', '\\']) {
      expect(validateFilename(`2024-03-01 Smith${character}Jones.pdf`, 'pdf')).toBe(`\u201c${character}\u201d cannot be used in a filename; it separates folders.`);
    }
    // Control characters, which Rust's is_control means as category Cc.
    expect(validateFilename('2024-03-01 Lease\tAcme.pdf', 'pdf')).toBe('A tab cannot be used in a filename.');
    expect(validateFilename('2024-03-01 Lease\nAcme.pdf', 'pdf')).toBe('A line break cannot be used in a filename.');
    expect(validateFilename('2024-03-01 Lease\u0007Acme.pdf', 'pdf')).toBe('The control character U+0007 cannot be used in a filename.');
    expect(validateFilename('2024-03-01 Lease\u0085Acme.pdf', 'pdf')).toMatch(/U\+0085/);
    // Every bidi control the backend lists, and the code points either side of
    // each range, which it allows.
    for (const code of [0x061c, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066, 0x2067, 0x2068, 0x2069]) {
      expect(validateFilename(`2024-03-01 Lease${String.fromCodePoint(code)}.pdf`, 'pdf'), code.toString(16)).toMatch(/invisible text-direction mark/);
    }
    for (const code of [0x061b, 0x200d, 0x2010, 0x2029, 0x202f, 0x2065, 0x206a]) {
      expect(validateFilename(`2024-03-01 Lease${String.fromCodePoint(code)}.pdf`, 'pdf'), code.toString(16)).toBeUndefined();
    }
    // A trailing dot or space, which Windows drops; "." and "..".
    expect(validateFilename('2024-03-01 Lease.pdf.', 'pdf')).toBe('A filename cannot end with a dot; Windows would drop it.');
    expect(validateFilename('2024-03-01 Lease.pdf ', 'pdf')).toBe('A filename cannot end with a space; Windows would drop it.');
    expect(validateFilename('.', 'pdf')).toBe('\u201c.\u201d is not a filename.');
    expect(validateFilename('..', 'pdf')).toBe('\u201c..\u201d is not a filename.');
    expect(validateFilename('', 'pdf')).toBe('Filename is required');
    // 512 UTF-8 bytes, not characters: "é" is two.
    const stem = (length: number) => `2024-03-01 ${'a'.repeat(length - 11)}`;
    expect(new TextEncoder().encode(`${stem(508)}.pdf`).length).toBe(512);
    expect(validateFilename(`${stem(508)}.pdf`, 'pdf')).toBeUndefined();
    expect(validateFilename(`${stem(509)}.pdf`, 'pdf')).toBe('This filename is too long. Shorten it by about 1 character.');
    expect(validateFilename(`${stem(507)}é.pdf`, 'pdf')).toMatch(/too long/);
    expect(validateFilename(`${stem(600)}.pdf`, 'pdf')).toBe('This filename is too long. Shorten it by about 92 characters.');
    // The extension: kept, compared ASCII-case-insensitively; changed,
    // removed, or added to a file that never had one, refused.
    expect(validateFilename('2024-03-01 Lease.PDF', 'pdf')).toBeUndefined();
    expect(validateFilename('2024-03-01 Lease.docx', 'pdf')).toBe('The filename must end with \u201c.pdf\u201d, the file\'s own extension.');
    expect(validateFilename('2024-03-01 Lease', 'pdf')).toMatch(/must end with/);
    expect(validateFilename('2024-03-01 Lease.pdf', undefined)).toBe('This file has no extension, so Intern cannot rename it.');
  });

  it('reads an extension the way Path::extension does', () => {
    expect(filenameExtension('Lease.pdf')).toBe('pdf');
    expect(filenameExtension('archive.tar.gz')).toBe('gz');
    expect(filenameExtension('Lease.')).toBe('');
    expect(filenameExtension('Lease')).toBeUndefined();
    expect(filenameExtension('.pdf')).toBeUndefined();
    expect(filenameExtension('..pdf')).toBe('pdf');
  });

  it('splits a proposal into the part that can change and the extension that cannot', () => {
    expect(splitFilename('2024-03-01 Lease.pdf', 'pdf')).toEqual({ stem: '2024-03-01 Lease', extension: 'pdf' });
    expect(splitFilename('2024-03-01 Lease.PDF', 'pdf')).toEqual({ stem: '2024-03-01 Lease', extension: 'PDF' });
    // A proposal without the source's extension still gets it, locked.
    expect(splitFilename('2024-03-01 Lease', 'pdf')).toEqual({ stem: '2024-03-01 Lease', extension: 'pdf' });
    expect(splitFilename('', 'docx')).toEqual({ stem: '', extension: 'docx' });
    expect(splitFilename('README', undefined)).toEqual({ stem: 'README' });
    expect(joinFilename('  2024-03-01 Lease  ', 'pdf')).toBe('2024-03-01 Lease.pdf');
    expect(joinFilename('README', undefined)).toBe('README');
  });
});
