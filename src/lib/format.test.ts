import { describe, expect, it } from 'vitest';
import { eta, formatBytes, statusLabel } from './format';

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

describe('download sizes', () => {
  const MiB = 1024 ** 2;
  const total = 1_280_835_840;

  it('bytes and eta are human readable', () => {
    expect(formatBytes(total)).toBe('1.19 GiB');
    expect(formatBytes(393 * MiB)).toBe('393 MiB');
    expect(formatBytes(2048)).toBe('2 KiB');
    expect(formatBytes(300)).toBe('300 bytes');
    expect(formatBytes(1)).toBe('1 byte');
    expect(formatBytes(0)).toBe('0 bytes');

    // 30 MiB over 10 s is 3 MiB/s; 828 MiB left is about 4.6 minutes.
    const steady = [{ at: 0, bytes: 363 * MiB }, { at: 5_000, bytes: 378 * MiB }, { at: 10_000, bytes: 393 * MiB }];
    expect(eta(steady, total)).toBe('about 5 min left');
    expect(eta([{ at: 0, bytes: 0 }, { at: 10_000, bytes: total - 10 * MiB }], total)).toBe('less than a minute left');
    expect(eta([{ at: 0, bytes: 0 }, { at: 60_000, bytes: 4 * MiB }], total)).toBe('about 5 hours left');
    expect(eta([{ at: 0, bytes: 0 }, { at: 60_000, bytes: 10 * MiB }], total)).toBe('about 2 hours left');
  });

  it('gives no estimate until there is a rate worth quoting', () => {
    expect(eta([], total)).toBeUndefined();
    expect(eta([{ at: 0, bytes: 0 }], total)).toBeUndefined();
    // One poll to the next is too short a stretch to say anything.
    expect(eta([{ at: 0, bytes: 0 }, { at: 250, bytes: 50 * MiB }], total)).toBeUndefined();
    // Stalled, or already there.
    expect(eta([{ at: 0, bytes: 5 * MiB }, { at: 10_000, bytes: 5 * MiB }], total)).toBeUndefined();
    expect(eta([{ at: 0, bytes: total - MiB }, { at: 10_000, bytes: total }], total)).toBeUndefined();
  });
});
