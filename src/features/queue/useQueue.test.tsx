import { StrictMode } from 'react';
import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { DesktopBridge } from '../../lib/bridge';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueBridgeEvent } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';
import { shareUnchanged, useQueue } from './useQueue';

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

const item: QueueItem = { id: 'document', originalFilename: 'agreement.pdf', status: 'waiting' };

describe('queue synchronization', () => {
  it('ignores an older refresh that resolves after a newer one', async () => {
    const first = deferred<QueueItem[]>();
    const second = deferred<QueueItem[]>();
    const listItems = vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const bridge = { ...createInMemoryBridge(), listItems };
    const { result } = renderHook(() => useQueue(bridge));
    await act(async () => {
      const refresh = result.current.refresh();
      second.resolve([{ ...item, status: 'completed' }]);
      await refresh;
    });
    await act(async () => { first.resolve([item]); await first.promise; });
    expect(result.current.items[0].status).toBe('completed');
  });

  it('retains the last snapshot on failure and clears the error after retry', async () => {
    const base = createInMemoryBridge({ items: [item] });
    const listItems = vi.fn(base.listItems);
    const bridge = { ...base, listItems };
    const { result } = renderHook(() => useQueue(bridge));
    await waitFor(() => expect(result.current.items).toEqual([item]));
    const error = new Error('Queue database is busy.');
    listItems.mockRejectedValueOnce(error);
    await act(async () => { await expect(result.current.refresh()).rejects.toBe(error); });
    expect(result.current.items).toEqual([item]);
    expect(result.current.error).toEqual({ kind: 'snapshot', cause: error });
    await act(async () => { await result.current.refresh(); });
    expect(result.current.error).toBeNull();
  });

  it('does not hide a subscription failure when a snapshot succeeds', async () => {
    const error = new Error('Event connection failed.');
    const stop = vi.fn();
    const subscribeQueue = vi.fn().mockRejectedValueOnce(error).mockResolvedValue(stop);
    const bridge = { ...createInMemoryBridge({ items: [item] }), subscribeQueue };
    const { result, unmount } = renderHook(() => useQueue(bridge));
    await waitFor(() => expect(result.current.error?.kind).toBe('subscription'));
    await act(async () => { await result.current.refresh(); });
    expect(result.current.error?.cause).toBe(error);
    act(() => result.current.reconnect());
    await waitFor(() => expect(result.current.error).toBeNull());
    expect(subscribeQueue).toHaveBeenCalledTimes(2);
    unmount();
    expect(stop).toHaveBeenCalledOnce();
  });

  it('does not turn settled items back into processing on late progress', async () => {
    const statuses = ['waiting', 'processing', 'ready', 'review', 'completed', 'failed'] as const;
    const items = statuses.map((status) => ({ ...item, id: status, status }));
    let listener!: (event: QueueBridgeEvent) => void;
    const bridge = {
      ...createInMemoryBridge({ items }),
      subscribeQueue: async (next: typeof listener) => { listener = next; return () => {}; },
    };
    const { result } = renderHook(() => useQueue(bridge));
    await waitFor(() => expect(result.current.items).toHaveLength(statuses.length));
    act(() => {
      for (const status of statuses) listener({ type: 'progress', itemId: status, stage: 'analyzing', progress: 40 });
    });
    expect(result.current.items.map((entry) => entry.status)).toEqual(['processing', 'processing', 'ready', 'review', 'completed', 'failed']);
  });

  // The worker reports `extracting` while it reads; that used to be thrown
  // away and the row went on saying nothing about what it was doing.
  it('takes the stage from progress events and drops a figure from an earlier stage', async () => {
    let listener!: (event: QueueBridgeEvent) => void;
    const bridge = {
      ...createInMemoryBridge({ items: [item] }),
      subscribeQueue: async (next: typeof listener) => { listener = next; return () => {}; },
    };
    const { result } = renderHook(() => useQueue(bridge));
    await waitFor(() => expect(result.current.items).toHaveLength(1));
    const current = () => result.current.items[0];

    act(() => listener({ type: 'progress', itemId: item.id, stage: 'extracting' }));
    expect(current()).toMatchObject({ status: 'processing', stage: 'reading' });
    act(() => listener({ type: 'progress', itemId: item.id, stage: 'ocr', progress: 40 }));
    expect(current()).toMatchObject({ stage: 'reading', progress: 40 });
    // A stage that names no stage leaves the last one, and its figure, alone.
    act(() => listener({ type: 'progress', itemId: item.id, stage: 'cancel_requested' }));
    expect(current()).toMatchObject({ stage: 'reading', progress: 40 });
    // Reading's 40% says nothing about naming.
    act(() => listener({ type: 'progress', itemId: item.id, stage: 'analyzing' }));
    expect(current().stage).toBe('naming');
    expect(current()).not.toHaveProperty('progress');
  });

  it('ignores events and results from a replaced bridge', async () => {
    let oldListener!: (event: QueueBridgeEvent) => void;
    const pending = deferred<QueueItem[]>();
    const oldBridge = {
      ...createInMemoryBridge(),
      listItems: vi.fn(() => pending.promise),
      subscribeQueue: async (next: typeof oldListener) => { oldListener = next; return () => {}; },
    };
    const nextBridge = { ...createInMemoryBridge({ items: [{ ...item, status: 'completed' }] }), subscribeQueue: async () => () => {} };
    const { result, rerender } = renderHook<ReturnType<typeof useQueue>, { bridge: DesktopBridge }>(({ bridge }) => useQueue(bridge), { initialProps: { bridge: oldBridge } });
    await waitFor(() => expect(oldListener).toBeTypeOf('function'));
    rerender({ bridge: nextBridge });
    await waitFor(() => expect(result.current.items[0]?.status).toBe('completed'));
    const requests = oldBridge.listItems.mock.calls.length;
    await act(async () => {
      oldListener({ type: 'changed', paused: true });
      oldListener({ type: 'progress', itemId: item.id, stage: 'analyzing', progress: 20 });
      pending.resolve([item]);
      await pending.promise;
    });
    expect(oldBridge.listItems).toHaveBeenCalledTimes(requests);
    expect(result.current.items[0].status).toBe('completed');
    expect(result.current.paused).toBe(false);
  });

  // Every queue event rereads the whole list as new objects. The rows are
  // memoized on their item, so a read that keeps the unchanged ones is what
  // lets one document's change render one row instead of all of them.
  it('keeps the items a reread did not change, and the list itself when none did', async () => {
    const queue = (): QueueItem[] => [
      { id: 'a', originalFilename: 'a.pdf', status: 'review', proposedFilename: '2024-01-02 Lease.pdf', evidence: { date: 'January 2, 2024' }, datesInDocument: ['2024-01-02'], readBy: { tier: 'accurate', modelId: 'qwen', displayName: 'Qwen3.5 9B' } },
      { id: 'b', originalFilename: 'b.pdf', status: 'processing', stage: 'naming', cancelable: true },
      { id: 'c', originalFilename: 'c.pdf', status: 'waiting' },
    ];
    let listed = queue();
    let listener!: (event: QueueBridgeEvent) => void;
    const bridge = {
      ...createInMemoryBridge(),
      // New objects on every read, as the desktop bridge returns them.
      listItems: async () => listed.map((entry) => structuredClone(entry)),
      subscribeQueue: async (next: typeof listener) => { listener = next; return () => {}; },
    };
    const { result } = renderHook(() => useQueue(bridge));
    await waitFor(() => expect(result.current.items).toHaveLength(3));
    const first = result.current.items;

    // Nothing changed: the same list, so nothing that depends on it runs again.
    await act(async () => { listener({ type: 'changed' }); await result.current.refresh(); });
    expect(result.current.items).toBe(first);

    // One document settles: it is new, the others are the objects on screen.
    listed = queue().map((entry) => entry.id === 'b' ? { id: 'b', originalFilename: 'b.pdf', status: 'ready', proposedFilename: '2024-02-03 Invoice.pdf', confidence: 1 } : entry);
    await act(async () => { await result.current.refresh(); });
    const second = result.current.items;
    expect(second).not.toBe(first);
    expect(second[0]).toBe(first[0]);
    expect(second[1]).toMatchObject({ status: 'ready', proposedFilename: '2024-02-03 Invoice.pdf' });
    expect(second[1]).not.toBe(first[1]);
    expect(second[2]).toBe(first[2]);

    // A progress event's change is the window's own; the next read still decides.
    act(() => listener({ type: 'progress', itemId: 'c', stage: 'extracting', progress: 40 }));
    expect(result.current.items[2]).toMatchObject({ status: 'processing', stage: 'reading', progress: 40 });
    expect(result.current.items[0]).toBe(first[0]);
    await act(async () => { await result.current.refresh(); });
    expect(result.current.items[2]).toEqual({ id: 'c', originalFilename: 'c.pdf', status: 'waiting' });
    expect(result.current.items[0]).toBe(first[0]);
  });

  it('disposes late subscriptions across Strict Mode cleanup and unmount', async () => {
    const subscriptions: ReturnType<typeof deferred<() => void>>[] = [];
    const subscribeQueue = vi.fn(() => {
      const subscription = deferred<() => void>();
      subscriptions.push(subscription);
      return subscription.promise;
    });
    const bridge = { ...createInMemoryBridge({ items: [] }), subscribeQueue };
    const { unmount } = renderHook(() => useQueue(bridge), { wrapper: StrictMode });
    await waitFor(() => expect(subscriptions).toHaveLength(2));
    const firstStop = vi.fn();
    const secondStop = vi.fn();
    unmount();
    await act(async () => {
      subscriptions[0].resolve(firstStop);
      subscriptions[1].resolve(secondStop);
      await Promise.all(subscriptions.map((subscription) => subscription.promise));
    });
    expect(firstStop).toHaveBeenCalledOnce();
    expect(secondStop).toHaveBeenCalledOnce();
  });
});

describe('sharing unchanged items between reads', () => {
  const review = (id: string, overrides: Partial<QueueItem> = {}): QueueItem => ({
    id, originalFilename: `${id}.pdf`, status: 'review', proposedFilename: `2024-01-02 ${id}.pdf`, confidence: 1,
    evidence: { date: 'January 2, 2024', type: 'LEASE', parties: 'Northwind Traders' }, reasonCodes: ['PARTY_UNSUPPORTED', 'LOW_CONFIDENCE'],
    houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }], ...overrides,
  });
  const reread = (items: QueueItem[]) => items.map((entry) => structuredClone(entry));

  it('takes the first read as it is', () => {
    const read = [review('a')];
    expect(shareUnchanged([], read)).toBe(read);
  });

  it('returns the list on screen when a read says the same', () => {
    const previous = [review('a'), review('b')];
    expect(shareUnchanged(previous, reread(previous))).toBe(previous);
  });

  it('replaces only what differs, at any depth', () => {
    const previous = [review('a'), review('b'), review('c'), review('d'), review('e')];
    const read = reread(previous);
    read[1] = review('b', { status: 'ready' });
    read[2] = review('c', { evidence: { date: 'January 2, 2024', type: 'LEASE', parties: 'Fabrikam Inc.' } });
    read[3] = review('d', { reasonCodes: ['PARTY_UNSUPPORTED'] });
    read[4] = review('e', { houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso Ltd' }] });
    const merged = shareUnchanged(previous, read);
    expect(merged).not.toBe(previous);
    expect(merged[0]).toBe(previous[0]);
    for (const index of [1, 2, 3, 4]) expect(merged[index]).toBe(read[index]);
  });

  it('notices a part that was added or taken away', () => {
    const previous = [review('a'), review('b', { suggestedDate: '2024-01-02' })];
    const read = [review('a', { nearDuplicateOf: '2024-01-02 Lease.pdf' }), review('b')];
    const merged = shareUnchanged(previous, read);
    expect(merged[0]).toBe(read[0]);
    expect(merged[1]).toBe(read[1]);
    expect(merged[1]).not.toHaveProperty('suggestedDate');
  });

  it('follows the read for order and for which items exist', () => {
    const previous = [review('a'), review('b'), review('c')];
    // b was filed and left; d arrived first in the list.
    const read = [review('d'), ...reread([previous[2], previous[0]])];
    const merged = shareUnchanged(previous, read);
    expect(merged.map((entry) => entry.id)).toEqual(['d', 'c', 'a']);
    expect(merged[0]).toBe(read[0]);
    expect(merged[1]).toBe(previous[2]);
    expect(merged[2]).toBe(previous[0]);
    // The same items in another order is a different list.
    const reordered = shareUnchanged(previous, reread([previous[1], previous[0], previous[2]]));
    expect(reordered).not.toBe(previous);
    expect(reordered).toEqual([previous[1], previous[0], previous[2]]);
    expect(reordered[0]).toBe(previous[1]);
    // And a shorter one, even when what is left reads the same.
    const shorter = shareUnchanged(previous, reread(previous.slice(0, 2)));
    expect(shorter).not.toBe(previous);
    expect(shorter).toHaveLength(2);
  });

  it('counts the same data with its keys in another order as unchanged', () => {
    // A progress event writes `stage` and `progress` after the keys the bridge wrote.
    const touched: QueueItem = { id: 'a', originalFilename: 'a.pdf', cancelable: true, status: 'processing', progress: 40, stage: 'reading' };
    const read: QueueItem[] = [{ id: 'a', originalFilename: 'a.pdf', status: 'processing', stage: 'reading', progress: 40, cancelable: true }];
    const previous = [touched];
    expect(shareUnchanged(previous, read)).toBe(previous);
  });

  it('does not take a missing part for one that reads as nothing', () => {
    const previous: QueueItem[] = [{ id: 'a', originalFilename: 'a.pdf', status: 'review', reason: '' }];
    const read: QueueItem[] = [{ id: 'a', originalFilename: 'a.pdf', status: 'review' }];
    expect(shareUnchanged(previous, read)[0]).toBe(read[0]);
    expect(shareUnchanged(read, [{ id: 'a', originalFilename: 'a.pdf', status: 'review', confidence: 0 }])[0]).toHaveProperty('confidence', 0);
    expect(shareUnchanged([{ ...read[0], datesInDocument: [] }], [{ ...read[0], datesInDocument: ['2024-01-02'] }])[0].datesInDocument).toEqual(['2024-01-02']);
  });
});
