import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { estimateLeft, useBatchProgress } from './batch';
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

  it('does not time a document sent back to waiting, or count it done', () => {
    const { result, rerender } = renderHook(({ items }) => useBatchProgress(items), { initialProps: { items: [doc('a', 'processing'), doc('b', 'waiting')] } });
    act(() => { vi.setSystemTime(5_000); });
    rerender({ items: [doc('a', 'waiting'), doc('b', 'waiting')] });
    expect(result.current).toMatchObject({ total: 2, done: 0 });
  });
});
