import { describe, expect, it } from 'vitest';
import { describeAddReport } from './addReport';

describe('describeAddReport', () => {
  it('says what was added and names each file left out with a readable reason', () => {
    expect(describeAddReport({ added: 24, alreadyQueued: 0, skipped: [{ name: 'notes.zip', code: 'UNSUPPORTED_FORMAT' }] }))
      .toBe('Added 24 documents. Skipped 1: notes.zip (not a supported format).');
    expect(describeAddReport({ added: 1, alreadyQueued: 1, skipped: [] }))
      .toBe('Added 1 document. 1 was already in the queue.');
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [{ name: '~$nda.docx', code: 'TEMPORARY_FILE' }, { name: 'blank.pdf', code: 'EMPTY_FILE' }] }))
      .toBe('Skipped 2: ~$nda.docx (a temporary or hidden file), blank.pdf (the file is empty).');
  });

  it('counts the files it does not name, and shows a code it has no words for as it is', () => {
    const skipped = Array.from({ length: 7 }, (_, index) => ({ name: `scan-${index + 1}.pdf`, code: index === 0 ? 'SOMETHING_NEW' : 'SOURCE_LOCKED' }));
    expect(describeAddReport({ added: 193, alreadyQueued: 0, skipped }))
      .toBe('Added 193 documents. Skipped 7: scan-1.pdf (SOMETHING_NEW), scan-2.pdf (another program has it open), scan-3.pdf (another program has it open), scan-4.pdf (another program has it open), scan-5.pdf (another program has it open), and 2 more.');
  });

  it('says so when a folder held nothing to add', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [] })).toBe('No documents were found to add.');
  });

  it('says what was copied into the input folder, by the folder\'s name', () => {
    expect(describeAddReport({ added: 3, alreadyQueued: 0, skipped: [], copied: 3, inputFolder: 'C:\\Intern\\Input' }))
      .toBe('Copied 3 files into Input.');
    expect(describeAddReport({ added: 2, alreadyQueued: 1, skipped: [{ name: 'Filed.pdf', code: 'IN_OUTPUT_FOLDER' }], copied: 1, inPlace: 1, inputFolder: 'C:\\Intern\\Input' }))
      .toBe('Copied 1 file into Input. 1 already in Input was added to the queue. 1 was already in the queue. Skipped 1: Filed.pdf (it is already in your output folder).');
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [], copied: 0 })).toBe('No documents were found to copy.');
  });

  it('says when copies wait for Microsoft before they join the queue', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [], copied: 2, held: 2, inputFolder: 'C:\\Users\\pat\\Contoso\\Files\\Inbox' }))
      .toBe('Copied 2 files into Inbox. They join the queue once OneDrive has uploaded them and Microsoft confirms they are yours.');
  });
});
