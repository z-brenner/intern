import { LoaderCircle, Pause } from 'lucide-react';
import { useEffect, useState } from 'react';
import { estimateLeft } from '../features/queue/batch';
import type { BatchProgress } from '../features/queue/batch';
import { Icon } from './Icon';

/**
 * How much of the current document the meter may claim before the document
 * is done: an estimate never fills its share, so the step when it finishes
 * is always forwards.
 */
const DOCUMENT_FILL_CAP = 0.95;

/**
 * How far along the run is, as a fraction. Documents done count whole. The
 * one being read counts for the part of a typical document's time it has
 * already taken, once there is a typical time to go by: without that the
 * meter stood still for the minute a document takes, and for a run of one
 * document stood at nothing until it was over.
 */
export function runFraction(progress: BatchProgress, now: number): number {
  if (progress.total <= 0) return 0;
  const within = progress.current && progress.perDocument && progress.startedAt !== undefined
    ? Math.min(DOCUMENT_FILL_CAP, Math.max(0, now - progress.startedAt) / progress.perDocument)
    : 0;
  return Math.min(1, (progress.done + within) / progress.total);
}

/**
 * The run as a whole, above the table: which document is being read, how
 * many are done, and - once two have been timed - roughly how long the rest
 * will take. A row only ever says "Proposing a name…"; on a computer that
 * takes half a minute a document, nothing said whether that was the last of
 * three or the fourth of two hundred. How long the current document has
 * taken is here too: a spinner looks the same at second two as at minute two.
 *
 * Not a live region: the queue's own status line already announces the
 * counts, and a sentence that changes with every document would talk over it.
 */
export function QueueProgress({ progress, paused }: { progress?: BatchProgress; paused: boolean }) {
  // Its own clock, so the elapsed time and the estimate tick here rather
  // than re-rendering the whole window every second.
  const [now, setNow] = useState(() => Date.now());
  const running = Boolean(progress) && !paused;
  useEffect(() => {
    if (!running) return;
    const timer = window.setInterval(() => setNow(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, [running]);
  if (!progress) return null;
  const estimate = estimateLeft(progress, now);
  const seconds = progress.startedAt === undefined ? 0 : Math.max(0, Math.floor((now - progress.startedAt) / 1_000));
  const elapsed = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
  const left = progress.total - progress.done;
  // Paused, the clock above stands still, and so does the meter: it does not step back.
  const fraction = runFraction(progress, now);
  const counts = `${progress.done} of ${progress.total} done`;
  return <section className={`queue-progress ${paused ? 'paused' : 'working'}`} aria-label="Queue progress">
    <Icon icon={paused ? Pause : LoaderCircle} />
    <p>{paused
      ? <>Paused <span>· {left} {left === 1 ? 'document' : 'documents'} still to read · {counts}</span></>
      : progress.current
        ? <>Reading {progress.current.originalFilename} <span>({elapsed}) · {counts}{estimate ? ` · ${estimate}` : ''}</span></>
        : <>Starting the next document <span>· {counts}</span></>}</p>
    <span />
    <span className="meter" aria-hidden="true"><span className="meter-fill" style={{ width: `${Math.round(fraction * 1000) / 10}%` }} /></span>
  </section>;
}
