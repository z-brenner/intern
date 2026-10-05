import type { ProcessingStage, QueueStatus } from '../types';

export const confidence = (value?: number) => value === undefined ? '—' : `${Math.round(value * 100)}%`;

const STAGE_LABELS: Record<ProcessingStage, string> = {
  reading: 'Reading document…',
  naming: 'Proposing a name…',
  filing: 'Renaming…',
};

/**
 * A row's status in words. A processing row says what it is doing, and adds a
 * percentage only when the backend actually sent one: it never does today, and
 * the "Processing (0%)" every document used to show read as stalled at nothing
 * for the whole of a long OCR run.
 */
export function statusLabel(status: QueueStatus, progress?: number, stage?: ProcessingStage): string {
  if (status === 'review') return 'Needs review';
  if (status !== 'processing') return status[0].toUpperCase() + status.slice(1);
  const label = stage ? STAGE_LABELS[stage] : 'Processing…';
  return typeof progress === 'number' && Number.isFinite(progress) ? `${label} (${Math.round(progress)}%)` : label;
}

export const byteCount = (value: number) => new Intl.NumberFormat('en-US').format(value);

const KIBIBYTE = 1024;
const MEBIBYTE = 1024 ** 2;
const GIBIBYTE = 1024 ** 3;
/// A size a person can weigh a decision against, in the same binary units the
/// README and the release notes quote. Below a mebibyte - the first moments of
/// a download - the unit steps down rather than rounding to a size that is not
/// true.
export const formatBytes = (value: number) => value >= GIBIBYTE
  ? `${(value / GIBIBYTE).toFixed(2)} GiB`
  : value >= MEBIBYTE
    ? `${Math.round(value / MEBIBYTE)} MiB`
    : value >= KIBIBYTE
      ? `${Math.round(value / KIBIBYTE)} KiB`
      : `${value} ${value === 1 ? 'byte' : 'bytes'}`;

/** How far a download had got at one moment, in milliseconds since the epoch. */
export interface ProgressSample { at: number; bytes: number }

/** The shortest stretch of samples a rate is taken from; one poll to the next is all noise. */
const ETA_MIN_SPAN_MS = 3_000;

/**
 * Time left on a download, from the rate across a window of recent samples
 * (oldest first), or undefined while there is too little to go on or nothing
 * is moving. The window is the caller's: the rate over the last half minute
 * follows a connection that speeds up or stalls, where the rate since the
 * start would not.
 */
export function eta(samples: ProgressSample[], totalBytes: number): string | undefined {
  const first = samples[0];
  const last = samples.at(-1);
  if (!first || !last || last.at - first.at < ETA_MIN_SPAN_MS) return undefined;
  const perMs = (last.bytes - first.bytes) / (last.at - first.at);
  const remaining = totalBytes - last.bytes;
  if (!(perMs > 0) || remaining <= 0) return undefined;
  const seconds = remaining / perMs / 1000;
  if (seconds < 60) return 'less than a minute left';
  const minutes = Math.round(seconds / 60);
  if (minutes < 90) return `about ${minutes} min left`;
  const hours = Math.round(minutes / 60);
  return `about ${hours} ${hours === 1 ? 'hour' : 'hours'} left`;
}
