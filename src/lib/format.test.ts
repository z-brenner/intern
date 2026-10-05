import { describe, expect, it } from 'vitest';
import { statusLabel } from './format';

describe('status labels', () => {
  // The backend never sends a percentage, so "Processing (0%)" was what every
  // document said for the whole of a long OCR run, and a figure that did
  // arrive would have printed every digit of a float.
  it('status label uses stage and rounds', () => {
    expect(statusLabel('processing', undefined, 'reading')).toBe('Reading document…');
    expect(statusLabel('processing', undefined, 'naming')).toBe('Proposing a name…');
    expect(statusLabel('processing', undefined, 'filing')).toBe('Renaming…');
    expect(statusLabel('processing', 33.3333, 'reading')).toBe('Reading document… (33%)');
    expect(statusLabel('processing', 99.6, 'naming')).toBe('Proposing a name… (100%)');
    expect(statusLabel('processing', Number.NaN, 'reading')).toBe('Reading document…');
    expect(statusLabel('processing', Number.POSITIVE_INFINITY, 'filing')).toBe('Renaming…');
    expect(statusLabel('processing')).toBe('Processing…');
    expect(statusLabel('processing', 0)).toBe('Processing… (0%)');
    for (const stage of ['reading', 'naming', 'filing'] as const) expect(statusLabel('processing', undefined, stage)).not.toContain('%');
    // A stage says nothing once the document has stopped being processed.
    expect(statusLabel('review', undefined, 'naming')).toBe('Needs review');
    expect(statusLabel('waiting', undefined, 'reading')).toBe('Waiting');
    expect(statusLabel('ready')).toBe('Ready');
  });
});
