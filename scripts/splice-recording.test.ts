import { execFile } from 'node:child_process';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { describe, expect, it } from 'vitest';
import { spliceRecording } from './splice-recording.mjs';

const exec = promisify(execFile);

interface Entry {
  file: string;
  text: string;
  prompt: string;
  date: string;
}

/**
 * A recording laid out the way `intern-evaluate` writes one: serde's pretty
 * printer, two-space indent, and a float that serde writes as `1.0` where
 * JSON.stringify would write `1`.
 */
function recording(entries: Entry[], note = 'recorded on Linux') {
  const blocks = entries.map((entry) => [
    '    {',
    `      "file": "${entry.file}",`,
    `      "sha256": "sha-${entry.file}",`,
    '      "extraction": {',
    '        "outcome": "parsed",',
    '        "source": {',
    '          "pages": [',
    '            {',
    '              "page_number": 1,',
    `              "text": ${JSON.stringify(entry.text)},`,
    '              "origin": "ocr",',
    '              "ocr_confidence": 80',
    '            }',
    '          ],',
    '          "parser_warnings": [],',
    '          "page_image": null',
    '        }',
    '      },',
    `      "prompt_sha256": "${entry.prompt}",`,
    '      "reply": {',
    '        "outcome": "proposed",',
    '        "proposal": {',
    `          "document_date": "${entry.date}",`,
    '          "confidence": 1.0',
    '        }',
    '      }',
    '    }',
  ].join('\n'));
  return [
    '{',
    '  "schema_version": 1,',
    '  "model_id": "intern-local",',
    '  "budget_characters": 12000,',
    '  "recorded_at_unix": 1790000000,',
    `  "note": ${JSON.stringify(note)},`,
    '  "fixtures": [',
    blocks.join(',\n'),
    '  ]',
    '}',
    '',
  ].join('\n');
}

const lease = { file: 'scanned-lease.pdf', text: 'LEASE AGREEMENT EFFECTIWE', prompt: 'p-lease-old', date: '2024-09-01' };
const invoice = { file: 'vendor-invoice.pdf', text: 'INVOICE\nContoso Worldwide', prompt: 'p-invoice', date: '2026-01-05' };
const receipt = { file: 'document-image.jpg', text: 'PACKING SLIP PS-311 DATE', prompt: 'p-slip-old', date: '2025-07-15' };

const committed = recording([lease, invoice, receipt]);

describe('splicing re-recorded fixtures into the committed recording', () => {
  it('replaces only the named entries and leaves every other byte as it was', () => {
    const relined = { ...lease, text: 'LEASE AGREEMENT\n\nEFFECTIWE', prompt: 'p-lease-new' };
    const reslipped = { ...receipt, text: 'PACKING SLIP PS-311\nDATE', prompt: 'p-slip-new' };
    const live = recording([relined, invoice, reslipped], 'live scratch run');

    const { text, differing } = spliceRecording(committed, live, {
      files: ['scanned-lease.pdf', 'document-image.jpg'],
      note: 'scans re-recorded',
    });

    expect(text).toBe(recording([relined, invoice, reslipped], 'scans re-recorded'));
    // serde's 1.0 survives; a parse-and-stringify would have written 1.
    expect(text).toContain('"confidence": 1.0');
    expect(differing).toEqual([]);
  });

  it('keeps the committed entry for a fixture whose reply alone differs, and says so', () => {
    const renamedReply = { ...invoice, date: '2026-01-06' };
    const live = recording([{ ...lease, prompt: 'p-lease-new' }, renamedReply, receipt]);

    const { text, differing } = spliceRecording(committed, live, {
      files: ['scanned-lease.pdf'],
      note: 'lease re-recorded',
    });

    expect(JSON.parse(text).fixtures[1].reply.proposal.document_date).toBe('2026-01-05');
    expect(differing).toEqual([{ file: 'vendor-invoice.pdf', extraction: false, prompt: false, reply: true }]);
  });

  it('refuses when the live run read a fixture it was not asked to splice differently', () => {
    const misread = { ...invoice, text: 'INVOICE Contoso Worldwide' };
    const live = recording([lease, misread, receipt]);

    expect(() => spliceRecording(committed, live, { files: ['scanned-lease.pdf'], note: 'lease' }))
      .toThrow(/vendor-invoice\.pdf/);
  });

  it('refuses recordings made for a different model or budget', () => {
    const live = recording([lease, invoice, receipt]).replace('"budget_characters": 12000', '"budget_characters": 8000');

    expect(() => spliceRecording(committed, live, { files: ['scanned-lease.pdf'], note: 'lease' }))
      .toThrow(/budget_characters/);
  });

  it('refuses a fixture recorded from different bytes, which needs the whole corpus re-recorded', () => {
    const live = recording([lease, invoice, receipt]).replace('"sha-scanned-lease.pdf"', '"sha-regenerated"');

    expect(() => spliceRecording(committed, live, { files: ['scanned-lease.pdf'], note: 'lease' }))
      .toThrow(/different bytes/);
  });

  it('refuses a splice without a note saying what was re-recorded', () => {
    expect(() => spliceRecording(committed, committed, { files: ['scanned-lease.pdf'], note: '' }))
      .toThrow(/note/);
  });

  it('writes the committed file in place from the command line', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'intern-splice-'));
    const committedPath = join(directory, 'corpus-recording.json');
    const livePath = join(directory, 'live.json');
    const relined = { ...lease, text: 'LEASE AGREEMENT\n\nEFFECTIWE', prompt: 'p-lease-new' };
    await writeFile(committedPath, committed);
    await writeFile(livePath, recording([relined, invoice, receipt]));

    const { stdout } = await exec(process.execPath, [
      'scripts/splice-recording.mjs', committedPath, livePath, '--note', 'lease re-recorded', 'scanned-lease.pdf',
    ]);

    expect(JSON.parse(stdout)).toEqual({ spliced: ['scanned-lease.pdf'], differing_elsewhere: [] });
    expect(await readFile(committedPath, 'utf8')).toBe(recording([relined, invoice, receipt], 'lease re-recorded'));
  });
});
