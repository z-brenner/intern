import { useEffect, useRef, useState } from 'react';
import type { QueueItem } from '../../types';

/** How many recent documents the time per document is taken from. */
const SAMPLE_SIZE = 8;
/** Documents timed before an estimate is offered: one is an anecdote. */
const MIN_SAMPLES = 2;
/** Estimates are padded: finishing early pleases, finishing late erodes trust. */
const PADDING = 1.2;
/** Where the last few documents' times are kept between launches, so a run's first document already has an estimate. */
export const TIMINGS_KEY = 'intern.documentTimings';
/** A stored time longer than this was a computer asleep, not a document being read. */
const LONGEST_SAMPLE_MS = 60 * 60 * 1000;

/**
 * The times kept from earlier runs: the last few, each a plausible number
 * of milliseconds. Storage that is missing, refuses, or holds something
 * else is no samples, as it was before any were kept.
 */
function storedTimings(): number[] {
  try {
    const stored: unknown = JSON.parse(window.localStorage.getItem(TIMINGS_KEY) ?? '[]');
    if (!Array.isArray(stored)) return [];
    return stored.filter((value): value is number => typeof value === 'number' && Number.isFinite(value) && value > 0 && value <= LONGEST_SAMPLE_MS).slice(-SAMPLE_SIZE);
  } catch { return []; }
}

function storeTimings(samples: number[]) {
  try { window.localStorage.setItem(TIMINGS_KEY, JSON.stringify(samples.filter((value) => value <= LONGEST_SAMPLE_MS))); } catch { /* Not kept; this run still has them. */ }
}

export interface BatchProgress {
  /** Documents in this run: read already, being read, and waiting. */
  total: number;
  /** Read already, in this run. */
  done: number;
  /** The document being read now. */
  current?: QueueItem;
  /** When the window first saw the current document being read. */
  startedAt?: number;
  /** The median time a document has taken, once enough have been timed. */
  perDocument?: number;
}

/**
 * "about 4 min left": the median time per document over what is left, less
 * what the current one has already taken, padded and rounded up. Undefined
 * until two documents have been timed, in this run or the ones before it.
 */
export function estimateLeft(progress: BatchProgress, now: number): string | undefined {
  if (progress.perDocument === undefined) return undefined;
  const left = progress.total - progress.done;
  const elapsed = progress.startedAt === undefined ? 0 : now - progress.startedAt;
  const ms = (Math.max(0, progress.perDocument - elapsed) + Math.max(0, left - 1) * progress.perDocument) * PADDING;
  const minutes = Math.ceil(ms / 60_000);
  return minutes <= 1 ? 'about a minute left' : `about ${minutes} min left`;
}

/** Being read, as opposed to being renamed: renames are quick and not what anyone waits on. */
const reading = (item: QueueItem) => item.status === 'processing' && item.stage !== 'filing';

/**
 * The queue's current run, as a person waits on it: how many documents it
 * has, how many are done, and roughly how long the rest will take.
 *
 * A run starts when there is something to read and nothing was being read,
 * and ends when nothing is left. Each document's time is measured here, from
 * when the window first saw it being read to when it stopped, and the
 * estimate (estimateLeft) is the median of the last few, padded and
 * rounded up - per Nielsen Norman, an estimate that runs over is worse than
 * none - and only offered once two documents have been timed.
 *
 * The last few times are kept between launches. A document takes about a
 * minute on the accurate model, so a run of one or two never reached an
 * estimate, and a longer one said nothing for its first two minutes; with
 * the times of the documents read before, the first document of a run has
 * one. They are the same computer and, nearly always, the same model, and
 * the newest times push the oldest out as the run goes on.
 *
 * `onFinished` is called once when a run with more than one document ends.
 */
export function useBatchProgress(items: QueueItem[], onFinished?: (done: number) => void): BatchProgress | undefined {
  const since = useRef(new Map<string, number>());
  const durations = useRef<number[] | undefined>(undefined);
  // Read once, not at every render.
  durations.current ??= storedTimings();
  const samples = durations.current;
  const run = useRef<{ done: Set<string>; seen: Set<string> } | undefined>(undefined);
  const finished = useRef(onFinished);
  useEffect(() => { finished.current = onFinished; });
  // The run is kept in refs and settled after each render; this re-renders
  // once it has changed, so the counts shown are never a step behind.
  const [, settled] = useState(0);

  const remaining = items.filter((item) => reading(item) || item.status === 'waiting');
  const active = remaining.length > 0;

  useEffect(() => {
    const at = Date.now();
    let changed = false;
    const current = run.current ?? (active ? { done: new Set<string>(), seen: new Set<string>() } : undefined);
    for (const item of items) {
      if (reading(item)) {
        // A start is a change too: the strip's clock counts from it, and
        // nothing else may render again while one long reading runs.
        if (!since.current.has(item.id)) { since.current.set(item.id, at); changed = true; }
        current?.seen.add(item.id);
      } else {
        const started = since.current.get(item.id);
        if (started !== undefined) {
          since.current.delete(item.id);
          // Back to waiting is a pause or a retry, not a reading finished.
          if (item.status !== 'waiting') {
            durations.current = [...(durations.current ?? []), at - started].slice(-SAMPLE_SIZE);
            storeTimings(durations.current);
            current?.done.add(item.id);
            changed = true;
          }
        }
        if (item.status === 'waiting') current?.seen.add(item.id);
      }
    }
    // An item removed from the queue mid-read is not a reading to time.
    for (const id of [...since.current.keys()]) if (!items.some((item) => item.id === id)) since.current.delete(id);
    if (active) {
      run.current = current;
    } else if (run.current) {
      const done = run.current.done.size;
      run.current = undefined;
      if (done > 1) finished.current?.(done);
    }
    if (changed) settled((count) => count + 1);
  }, [items, active]);

  if (!active) return undefined;
  const done = run.current?.done.size ?? 0;
  const total = done + remaining.length;
  const current = items.find(reading);
  const sorted = [...samples].sort((a, b) => a - b);
  return {
    total,
    done,
    current,
    startedAt: current ? since.current.get(current.id) : undefined,
    perDocument: sorted.length >= MIN_SAMPLES ? sorted[Math.floor(sorted.length / 2)] : undefined,
  };
}
