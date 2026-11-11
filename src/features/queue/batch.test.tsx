import { act, render, renderHook, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { TIMINGS_KEY, estimateLeft, useBatchProgress } from './batch';
import type { BatchProgress } from './batch';
import { QueueProgress, runFraction } from '../../components/QueueProgress';
import type { QueueItem } from '../../types';

const doc = (id: string, status: QueueItem['status']): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, ...(status === 'processing' ? { stage: 'naming' as const } : {}) });

describe('the queue run', () => {
  beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(0); });
  afterEach(() => { vi.useRealTimers(); });

  it('counts a run from its first document to its last, and estimates only once two are timed', () => {
    const finished = vi.fn();
    const { result, rerender } = renderHook(({ items }) => useBatchProgress(items, finished), {
      initialProps: { items: [doc('a', 'processing'), doc('b', 'waiting'), doc('c', 'waiting')] },
    });
    expect(result.current).toMatchObject({ total: 3, done: 0, current: { id: 'a' } });
    expect(result.current!.perDocument).toBeUndefined();

    act(() => { vi.setSystemTime(30_000); });
    rerender({ items: [doc('a', 'ready'), doc('b', 'processing'), doc('c', 'waiting')] });
    expect(result.current).toMatchObject({ total: 3, done: 1, current: { id: 'b' } });
    expect(result.current!.perDocument).toBeUndefined();

    act(() => { vi.setSystemTime(90_000); });
    rerender({ items: [doc('a', 'ready'), doc('b', 'review'), doc('c', 'processing')] });
    expect(result.current).toMatchObject({ total: 3, done: 2, current: { id: 'c' }, startedAt: 90_000 });
    // The median of 30s and 60s, as the sorted middle: 60s.
    expect(result.current!.perDocument).toBe(60_000);
    // One left, 10s in: (60 - 10) x 1.2 = 60s, rounded up.
    expect(estimateLeft(result.current!, 100_000)).toBe('about a minute left');
    // Ten left: (50 + 9 x 60) x 1.2 = 708s, rounded up to 12 minutes.
    expect(estimateLeft({ ...result.current!, total: 12 }, 100_000)).toBe('about 12 min left');

    expect(finished).not.toHaveBeenCalled();
    act(() => { vi.setSystemTime(150_000); });
    rerender({ items: [doc('a', 'ready'), doc('b', 'review'), doc('c', 'ready')] });
    expect(result.current).toBeUndefined();
    expect(finished).toHaveBeenCalledOnce();
    expect(finished).toHaveBeenCalledWith(3);
  });

  it('has the start of a document that begins on its own, with nothing else changing', () => {
    vi.setSystemTime(4_000);
    const { result } = renderHook(() => useBatchProgress([doc('a', 'processing'), doc('b', 'waiting')]));
    expect(result.current).toMatchObject({ current: { id: 'a' }, startedAt: 4_000 });
  });

  // A document takes about a minute, so a run of one or two never reached
  // an estimate, and a longer one said nothing for its first two minutes.
  it('estimates from the first document of a run when earlier runs were timed, and keeps the newest times', () => {
    window.localStorage.setItem(TIMINGS_KEY, JSON.stringify([40_000, 60_000, 80_000]));
    const { result, rerender } = renderHook(({ items }) => useBatchProgress(items), {
      initialProps: { items: [doc('a', 'processing'), doc('b', 'waiting'), doc('c', 'waiting')] },
    });
    // Nothing has finished in this run, and there is already a time to go by: the median of the three.
    expect(result.current).toMatchObject({ total: 3, done: 0, current: { id: 'a' }, startedAt: 0, perDocument: 60_000 });
    // Three to read, 15s into the first: (45 + 2 x 60) x 1.2 = 198s, rounded up.
    expect(estimateLeft(result.current!, 15_000)).toBe('about 4 min left');

    // This run's own times join them, for this run and the next launch.
    act(() => { vi.setSystemTime(100_000); });
    rerender({ items: [doc('a', 'ready'), doc('b', 'processing'), doc('c', 'waiting')] });
    expect(JSON.parse(window.localStorage.getItem(TIMINGS_KEY)!)).toEqual([40_000, 60_000, 80_000, 100_000]);
    expect(result.current).toMatchObject({ done: 1, perDocument: 80_000 });

    // Only the last eight are kept.
    window.localStorage.setItem(TIMINGS_KEY, JSON.stringify([1_000, 2_000, 3_000, 4_000, 5_000, 6_000, 7_000, 8_000, 9_000, 10_000]));
    const later = renderHook(({ items }) => useBatchProgress(items), { initialProps: { items: [doc('x', 'processing'), doc('y', 'waiting')] } });
    // Of 3s to 10s, the sorted middle.
    expect(later.result.current!.perDocument).toBe(7_000);
    act(() => { vi.setSystemTime(111_000); });
    later.rerender({ items: [doc('x', 'ready'), doc('y', 'processing')] });
    expect(JSON.parse(window.localStorage.getItem(TIMINGS_KEY)!)).toEqual([4_000, 5_000, 6_000, 7_000, 8_000, 9_000, 10_000, 11_000]);
  });

  it('takes one stored time for an anecdote, and anything else in storage for nothing', () => {
    const first = () => renderHook(() => useBatchProgress([doc('a', 'processing'), doc('b', 'waiting')])).result.current!;
    window.localStorage.setItem(TIMINGS_KEY, JSON.stringify([60_000]));
    expect(first().perDocument).toBeUndefined();
    // Not a list; not numbers; a time no document takes (a computer left asleep); nothing at all.
    for (const stored of ['{"a":1}', '"60000"', 'not json', JSON.stringify(['60000', null, -5, 0, Number.MAX_VALUE, 9 * 60 * 60 * 1000]), '']) {
      window.localStorage.setItem(TIMINGS_KEY, stored);
      expect(first().perDocument, stored).toBeUndefined();
    }
    // Two good ones among the rest are two samples.
    window.localStorage.setItem(TIMINGS_KEY, JSON.stringify(['x', 30_000, null, 50_000, 9 * 60 * 60 * 1000]));
    expect(first().perDocument).toBe(50_000);
  });

  it('still times a run when storage refuses to be read or written', () => {
    const getItem = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('Storage is disabled.'); });
    try {
      const { result, rerender } = renderHook(({ items }) => useBatchProgress(items), {
        initialProps: { items: [doc('a', 'processing'), doc('b', 'waiting'), doc('c', 'waiting')] },
      });
      expect(result.current!.perDocument).toBeUndefined();
      act(() => { vi.setSystemTime(30_000); });
      rerender({ items: [doc('a', 'ready'), doc('b', 'processing'), doc('c', 'waiting')] });
      act(() => { vi.setSystemTime(90_000); });
      rerender({ items: [doc('a', 'ready'), doc('b', 'review'), doc('c', 'processing')] });
      expect(result.current).toMatchObject({ done: 2, perDocument: 60_000 });
    } finally {
      getItem.mockRestore();
      setItem.mockRestore();
    }
  });

  it('does not time a document sent back to waiting, or count it done', () => {
    const { result, rerender } = renderHook(({ items }) => useBatchProgress(items), { initialProps: { items: [doc('a', 'processing'), doc('b', 'waiting')] } });
    act(() => { vi.setSystemTime(5_000); });
    rerender({ items: [doc('a', 'waiting'), doc('b', 'waiting')] });
    expect(result.current).toMatchObject({ total: 2, done: 0 });
  });
});

/*
  The meter stood still for the minute a document takes: it showed documents
  done out of the run, so one document's whole reading was an empty bar. With
  a typical time to go by, the document being read counts for the part of
  that time it has taken, short of finishing.
*/
describe('the run\'s meter', () => {
  beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(0); });
  afterEach(() => { vi.useRealTimers(); });
  const run = (overrides: Partial<BatchProgress> = {}): BatchProgress => ({ total: 4, done: 1, current: doc('b', 'processing'), startedAt: 0, perDocument: 60_000, ...overrides });
  const width = () => document.querySelector<HTMLElement>('.queue-progress .meter-fill')!.style.width;

  it('counts the part of a typical document\'s time the current one has taken, and never all of it', () => {
    expect(runFraction(run(), 0)).toBe(0.25);
    expect(runFraction(run(), 30_000)).toBe(0.375);
    // Past its typical time it waits just short of done, however long it takes.
    expect(runFraction(run(), 60_000)).toBeCloseTo((1 + 0.95) / 4);
    expect(runFraction(run(), 600_000)).toBeCloseTo((1 + 0.95) / 4);
    // Finishing it is a step forwards from there.
    expect(runFraction(run({ done: 2, startedAt: 600_000 }), 600_000)).toBeGreaterThan(runFraction(run(), 600_000));
    // A clock that reads earlier than the start claims nothing.
    expect(runFraction(run({ startedAt: 5_000 }), 0)).toBe(0.25);
    expect(runFraction(run({ total: 0, done: 0 }), 30_000)).toBe(0);
    expect(runFraction(run({ total: 1, done: 1, current: undefined }), 30_000)).toBe(1);
  });

  it('shows documents done and nothing more until there is a typical time, or with nothing being read', () => {
    expect(runFraction(run({ perDocument: undefined }), 45_000)).toBe(0.25);
    expect(runFraction(run({ current: undefined }), 45_000)).toBe(0.25);
    expect(runFraction(run({ startedAt: undefined }), 45_000)).toBe(0.25);
    // As the strip is drawn for a run nobody has timed: an empty meter, and no estimate.
    render(<QueueProgress progress={run({ done: 0, perDocument: undefined })} paused={false} />);
    act(() => { vi.advanceTimersByTime(30_000); });
    expect(width()).toBe('0%');
    expect(screen.getByLabelText('Queue progress')).toHaveTextContent('Reading b.pdf (0:30) · 0 of 4 done');
    expect(screen.getByLabelText('Queue progress')).not.toHaveTextContent('left');
  });

  it('moves with the clock while a document is read, and stands still while paused', () => {
    const view = render(<QueueProgress progress={run({ done: 0 })} paused={false} />);
    expect(width()).toBe('0%');
    act(() => { vi.advanceTimersByTime(15_000); });
    // A quarter of one document of four.
    expect(width()).toBe('6.3%');
    expect(screen.getByLabelText('Queue progress')).toHaveTextContent('Reading b.pdf (0:15) · 0 of 4 done · about 5 min left');
    act(() => { vi.advanceTimersByTime(15_000); });
    expect(width()).toBe('12.5%');

    view.rerender(<QueueProgress progress={run({ done: 0 })} paused />);
    act(() => { vi.advanceTimersByTime(60_000); });
    expect(width()).toBe('12.5%');
    expect(screen.getByLabelText('Queue progress')).toHaveTextContent('Paused · 4 documents still to read · 0 of 4 done');
  });
});
