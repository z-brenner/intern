import { CheckCircle2, CircleAlert, Clock3, LoaderCircle, XCircle } from 'lucide-react';
import { Icon } from './Icon';
import { itemNote } from '../features/queue/check';
import { statusLabel } from '../lib/format';
import type { QueueItem } from '../types';

const glyphs = { ready: CheckCircle2, review: CircleAlert, waiting: Clock3, processing: LoaderCircle, completed: CheckCircle2, failed: XCircle };

/**
 * Some rows say more than their status alone. One waiting with a reason was
 * read before and will be read again - in doubt over whether the failure was
 * the machine's, or kept for its second attempt - and says why on hover. A
 * ready copy of a filed document waits for no one: the queue files it as
 * a copy when it runs. And a document that is ready, or was renamed, while
 * Intern had something noted about its reading (the Balanced review level)
 * is still ready or completed, and says both: the status is what the
 * filename's cell is described by, so someone moving down the first column
 * hears of the note without going looking for it. The note itself is on
 * hover, in the Check column and in the Details panel.
 */
function label(item: QueueItem, noted: boolean): string {
  if (item.status === 'waiting' && item.reason) return 'Waiting to read again';
  if (item.status === 'ready' && item.copyOf !== undefined) return 'Ready to file as a copy';
  const status = statusLabel(item.status, item.progress, item.stage);
  // No comma: "Completed with a note" is as wide as the longest label the column was sized for.
  return noted ? `${status} with a note` : status;
}

export function StatusCell({ item }: { item: QueueItem }) {
  const note = itemNote(item);
  return <span className={`status ${item.status}`} title={item.status === 'waiting' ? item.reason : note?.text}><Icon icon={glyphs[item.status]} />{label(item, note !== undefined)}</span>;
}
