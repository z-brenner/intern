import { useEffect, useState } from 'react';
import { byteCount, eta, formatBytes } from '../lib/format';
import type { ProgressSample } from '../lib/format';
import type { SetupState } from '../types';

/** How far back the download rate is taken from. */
const RATE_WINDOW_MS = 30_000;

/**
 * The model download as a person reads it: "393 MiB of 1.19 GiB · 32% · about
 * 2 min left". It used to be "412,090,368 of 1,280,835,840 bytes", which
 * answers neither how far along it is nor how long to wait. The exact count
 * stays in the title, for anyone comparing it with a file on disk.
 */
export function DownloadProgress({ setup }: { setup: SetupState }) {
  const samples = useRateSamples(setup);
  const { downloadedBytes: done, totalBytes: total } = setup;
  const left = setup.state === 'downloading' ? eta(samples, total) : undefined;
  const percent = total > 0 ? Math.min(100, Math.max(0, Math.floor((done / total) * 100))) : undefined;
  const parts = percent === undefined
    ? [formatBytes(done)]
    : [`${formatBytes(done)} of ${formatBytes(total)}`, `${percent}%`, ...(left ? [left] : [])];
  // Before the manifest is read the total is 0, and "of 0 bytes" is not a size.
  const exact = percent === undefined ? `${byteCount(done)} bytes` : `${byteCount(done)} of ${byteCount(total)} bytes`;
  return <>
    <progress aria-label="Model setup progress" value={done} max={total} />
    <p aria-live="polite" title={exact}>{parts.join(' · ')}</p>
  </>;
}

/**
 * One sample per setup state received while downloading - every poll, not
 * only those that moved, so a stalled download's estimate grows instead of
 * freezing at the last good one.
 */
function useRateSamples(setup: SetupState): ProgressSample[] {
  const [samples, setSamples] = useState<ProgressSample[]>([]);
  useEffect(() => {
    if (setup.state !== 'downloading') {
      setSamples((current) => current.length ? [] : current);
      return;
    }
    const now = Date.now();
    const bytes = setup.downloadedBytes;
    // A count that went backwards is a restarted download; its old samples
    // describe a different transfer.
    setSamples((current) => [...current.filter((sample) => now - sample.at <= RATE_WINDOW_MS && sample.bytes <= bytes), { at: now, bytes }]);
  }, [setup]);
  return samples;
}
