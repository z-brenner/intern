import { LoaderCircle, Pause } from 'lucide-react';
import { useEffect, useState } from 'react';
import { estimateLeft } from '../features/queue/batch';
import type { BatchProgress } from '../features/queue/batch';
import { Icon } from './Icon';

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
  const fraction = progress.total > 0 ? progress.done / progress.total : 0;
  const counts = `${progress.done} of ${progress.total} done`;
  return <section className={`queue-progress ${paused ? 'paused' : 'working'}`} aria-label="Queue progress">
    <Icon icon={paused ? Pause : LoaderCircle} />
    <p>{paused
      ? <>Paused <span>· {left} {left === 1 ? 'document' : 'documents'} still to read · {counts}</span></>
      : progress.current
        ? <>Reading {progress.current.originalFilename} <span>({elapsed}) · {counts}{estimate ? ` · ${estimate}` : ''}</span></>
        : <>Starting the next document <span>· {counts}</span></>}</p>
    <span />
    <span className="meter" aria-hidden="true"><span className="meter-fill" style={{ width: `${Math.round(fraction * 100)}%` }} /></span>
  </section>;
}
