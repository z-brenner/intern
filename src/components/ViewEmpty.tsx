import { FileCheck2, Inbox } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { Icon } from './Icon';
import type { QueueView } from '../types';

/**
 * A filtered view with nothing in it, when the queue itself is not empty.
 *
 * Each one says what the view collects, so a person clicking "Completed"
 * and finding it blank learns what will appear there - rather than
 * wondering whether the click worked.
 */
const states: Record<QueueView, { icon: LucideIcon; title: string; body: string }> = {
  queue: {
    icon: Inbox,
    title: 'The queue is clear',
    body: 'Everything Intern has read has been dealt with. Drop more documents above to keep going.',
  },
  completed: {
    icon: FileCheck2,
    title: 'No renamed documents yet',
    body: 'Approve a proposal and the renamed file is listed here, with a way to undo it.',
  },
};

export function ViewEmpty({ view }: { view: QueueView }) {
  const { icon, title, body } = states[view];
  return <div className="view-empty">
    <span className="view-empty-mark"><Icon icon={icon} /></span>
    <p className="view-empty-title">{title}</p>
    <p>{body}</p>
  </div>;
}
