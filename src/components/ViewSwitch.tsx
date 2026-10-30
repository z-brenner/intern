import type { QueueView } from '../types';

const entries = [
  { view: 'queue' as const, label: 'Queue' },
  { view: 'completed' as const, label: 'Completed' },
];

/**
 * Queue and Completed, as a switch at the head of the table they choose
 * between. They were a 230px sidebar of their own, two entries tall, which
 * beside the Details panel left the table too narrow for its status labels.
 */
export function ViewSwitch({ active, items, onChange }: { active: QueueView; items: { status: string }[]; onChange(view: QueueView): void }) {
  // Items needing review are counted in the queue's own toolbar, where they
  // are worked through; they had a page of their own here once.
  const count = (view: QueueView) => items.filter((item) => view === 'queue' ? item.status !== 'completed' : item.status === 'completed').length;
  return <nav className="view-switch" aria-label="Queue navigation">
    {/*
      The count is part of the name: a bare aria-label of "Queue" replaced the
      visible "Queue 5", so a screen reader never heard how much was waiting.
      `data-view` is what code looks a view's button up by, since the name
      changes with the queue.
    */}
    {entries.map(({ view, label }) => {
      const total = count(view);
      return <button type="button" key={view} data-view={view} onClick={() => onChange(view)} aria-current={active === view ? 'page' : undefined} aria-label={`${label}, ${total}`}>
        <span>{label}</span><b>{total}</b>
      </button>;
    })}
  </nav>;
}
