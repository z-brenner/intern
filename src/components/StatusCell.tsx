import { CheckCircle2, CircleAlert, Clock3, LoaderCircle, XCircle } from 'lucide-react';
import { Icon } from './Icon';
import { statusLabel } from '../lib/format';
import type { QueueItem } from '../types';

const glyphs = { ready: CheckCircle2, review: CircleAlert, waiting: Clock3, processing: LoaderCircle, completed: CheckCircle2, failed: XCircle };

/**
 * Two rows say more than their status alone. One waiting with a reason was
 * read before and will be read again - in doubt over whether the failure was
 * the machine's, or kept for its second attempt - and says why on hover. A
 * ready copy of a filed document waits for no one: the queue files it as
 * a copy when it runs.
 */
function label(item: QueueItem): string {
  if (item.status === 'waiting' && item.reason) return 'Waiting to read again';
  if (item.status === 'ready' && item.copyOf !== undefined) return 'Ready to file as a copy';
  return statusLabel(item.status, item.progress, item.stage);
}

export function StatusCell({ item }: { item: QueueItem }) {
  return <span className={`status ${item.status}`} title={item.status === 'waiting' ? item.reason : undefined}><Icon icon={glyphs[item.status]} />{label(item)}</span>;
}
