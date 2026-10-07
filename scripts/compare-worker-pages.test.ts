import { describe, expect, it } from 'vitest';
import { compareDocuments, readWithWorker } from './compare-worker-pages.mjs';

interface Page {
  page_number: number;
  text: string;
  source: string;
  ocr_confidence: number | null;
  vision_escalated: boolean;
  layout?: { route: string };
}

function parsed(pages: Page[], warnings: string[] = []) {
  return {
    type: 'parsed',
    document: { pages, warnings, truncated: false, optional_image: null },
  };
}

function page(text: string, route?: string): Page {
  return {
    page_number: 1,
    text,
    source: 'native',
    ocr_confidence: null,
    vision_escalated: false,
    ...(route ? { layout: { route } } : {}),
  };
}

describe('compareDocuments', () => {
  it('accepts a fast-route page that is byte-for-byte what it was', () => {
    const result = compareDocuments('a.pdf', parsed([page('INVOICE\r\nTotal: $5')]), parsed([page('INVOICE\r\nTotal: $5', 'fast')]));

    expect(result.problems).toEqual([]);
    expect(result.identical).toBe(1);
    expect(result.routes).toEqual({ fast: 1 });
  });

  it('fails a fast-route page whose text changed by so much as a line ending', () => {
    const result = compareDocuments('a.pdf', parsed([page('INVOICE\r\nTotal: $5')]), parsed([page('INVOICE\nTotal: $5', 'fast')]));

    expect(result.problems).toEqual(['a.pdf page 1: fast-route text changed']);
    expect(result.identical).toBe(0);
  });

  it('only counts pages on the other routes, whose text is expected to change', () => {
    const result = compareDocuments(
      'lease.pdf',
      parsed([page('left right\r\nleft right')]),
      parsed([page('left\nleft\n\nright\nright', 'layout')]),
    );

    expect(result.problems).toEqual([]);
    expect(result.routes).toEqual({ layout: 1 });
  });

  it('fails a document whose pages, warnings, or outcome changed', () => {
    expect(compareDocuments('a.pdf', parsed([page('x')]), { type: 'error' }).problems).toEqual([
      'a.pdf: parsed before, error after',
    ]);
    expect(
      compareDocuments('a.pdf', parsed([page('x')]), parsed([page('x', 'fast'), { ...page('y', 'fast'), page_number: 2 }]))
        .problems,
    ).toEqual(['a.pdf: 1 pages before, 2 after']);
    expect(
      compareDocuments('a.pdf', parsed([page('x')]), parsed([page('x', 'fast')], ['LOW_OCR_CONFIDENCE'])).problems,
    ).toEqual(['a.pdf: warnings [] before, ["LOW_OCR_CONFIDENCE"] after']);
  });
});

describe('readWithWorker', () => {
  it('rejects instead of waiting on a worker that cannot run', async () => {
    await expect(readWithWorker('/nonexistent/intern-worker', ['one.pdf'])).rejects.toThrow(/could not run/);
  });
});
