import type { QueueStatus } from '../types';

export const confidence = (value?: number) => value === undefined ? '—' : `${Math.round(value * 100)}%`;
export const statusLabel = (status: QueueStatus, progress?: number) => status === 'review' ? 'Needs review' : status === 'processing' ? `Processing (${progress ?? 0}%)` : status[0].toUpperCase() + status.slice(1);
export const byteCount = (value: number) => new Intl.NumberFormat('en-US').format(value);

const MEBIBYTE = 1024 ** 2;
const GIBIBYTE = 1024 ** 3;
/// A download size a person can weigh a decision against, in the same binary
/// units the README and the release notes quote.
export const byteSize = (value: number) => value >= GIBIBYTE
  ? `${(value / GIBIBYTE).toFixed(2)} GiB`
  : `${Math.max(1, Math.round(value / MEBIBYTE))} MiB`;

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
