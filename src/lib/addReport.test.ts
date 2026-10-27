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

  it('promises Microsoft only for the reason it is waiting on Microsoft', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [], copied: 1, held: 1, heldReason: 'UPLOADER_UNVERIFIED', inputFolder: 'D:\\Scans' }))
      .toBe('Copied 1 file into Scans. 1 waits in Scans until Intern can confirm who uploaded it.');
  });

  it('says a copy Microsoft reports as another account\'s upload is left in the Inbox', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [], copied: 1, held: 1, heldReason: 'UPLOADER_OTHER', inputFolder: 'C:\\Users\\pat\\Contoso\\Files\\Inbox' }))
      .toBe('Copied 1 file into Inbox. 1 was left in Inbox: Microsoft reports another account uploaded a file under that name, so Intern does not process it here. To process yours, rename it and add it again.');
  });

  it('says a copy that could not be taken back stays in the input folder', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [], copied: 1, held: 1, heldReason: 'LEFT_IN_INPUT_FOLDER', inputFolder: 'D:\\Input' }))
      .toBe('Copied 1 file into Input. 1 stays in Input and is picked up from there.');
  });

  it('says the shared folder\'s own refusals in words', () => {
    expect(describeAddReport({ added: 0, alreadyQueued: 0, skipped: [{ name: 'a.pdf', code: 'SYNC_CONFLICT_COPY' }, { name: 'b.pdf', code: 'CLAIM_IO_FAILED' }] }))
      .toBe('Skipped 2: a.pdf (a OneDrive conflict copy; resolve the conflict first), b.pdf (Intern could not note in the shared folder that it is working on it).');
  });
});
