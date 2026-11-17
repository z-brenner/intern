import type { ProcessingStage, QueueStatus } from '../types';

/**
 * A confidence figure as a percentage. The figure is how sure Intern is of
 * the name (`name_confidence` in the engine's review_level.rs): never the
 * model's own rating, which the local model does not give and used to read
 * 100% beside a document in review. One that is missing, or not a number,
 * is a dash: not measured is not zero.
 */
export const confidence = (value?: number) => value === undefined || !Number.isFinite(value) ? '—' : `${Math.round(value * 100)}%`;

/**
 * The tone a confidence figure is shown in: its row's status, never the
 * figure's size. A ready or renamed row's is green and a row in review's
 * amber, whatever the number, so the color can never say ready of a
 * document that waits, nor the reverse. Any other row's takes the color
 * around it.
 */
export function confidenceTone(status: QueueStatus): 'ready' | 'review' | undefined {
  if (status === 'ready' || status === 'completed') return 'ready';
  if (status === 'review') return 'review';
  return undefined;
}

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

/**
 * What an Install button says while an update installs. `progress` is
 * undefined until the download first reports, and its fraction is undefined
 * when the server sent no size. Never "100%" before the last byte: a
 * download that rounds up to done while it is still arriving reads as stuck.
 */
export const installingLabel = (progress?: { fraction: number | undefined }) => {
  if (!progress) return 'Installing…';
  if (progress.fraction === undefined) return 'Downloading…';
  if (progress.fraction >= 1) return 'Installing…';
  return `Downloading ${Math.min(99, Math.round(progress.fraction * 100))}%…`;
};
