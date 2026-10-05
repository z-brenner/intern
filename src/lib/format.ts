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

const MEBIBYTE = 1024 ** 2;
const GIBIBYTE = 1024 ** 3;
/// A download size a person can weigh a decision against, in the same binary
/// units the README and the release notes quote.
export const byteSize = (value: number) => value >= GIBIBYTE
  ? `${(value / GIBIBYTE).toFixed(2)} GiB`
  : `${Math.max(1, Math.round(value / MEBIBYTE))} MiB`;
