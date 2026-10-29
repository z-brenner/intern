import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import { createInMemoryBridge } from '../lib/inMemoryBridge';
import { ViewSwitch } from './ViewSwitch';

const items = [
  { status: 'review' }, { status: 'review' }, { status: 'review' }, { status: 'review' }, { status: 'review' },
  { status: 'waiting' }, { status: 'completed' },
];

describe('queue navigation', () => {
  // The aria-label replaced the visible "Queue 6", so a screen reader never
  // heard the count; the count is part of the name.
  it('names each view with its count', () => {
    render(<ViewSwitch active="queue" items={items} onChange={vi.fn()} />);
    const nav = screen.getByRole('navigation', { name: 'Queue navigation' });

    for (const [view, label, count] of [['queue', 'Queue', 6], ['completed', 'Completed', 1]] as const) {
      const button = within(nav).getByRole('button', { name: `${label}, ${count}` });
      expect(button).toHaveAttribute('data-view', view);
      expect(button).toHaveTextContent(`${label}${count}`);
    }
    // Items needing review are counted in the queue's toolbar, not on a page of their own.
    expect(within(nav).getAllByRole('button')).toHaveLength(2);
    expect(within(nav).queryByRole('button', { name: /Needs Review/i })).not.toBeInTheDocument();
  });

  it('follows the queue as counts change', () => {
    const view = render(<ViewSwitch active="queue" items={items} onChange={vi.fn()} />);
    view.rerender(<ViewSwitch active="queue" items={items.slice(1)} onChange={vi.fn()} />);

    const current = screen.getByRole('button', { name: 'Queue, 5' });
    expect(current).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('button', { name: 'Completed, 1' })).not.toHaveAttribute('aria-current');
  });

  // The view's button is found by data-view, not by its name: the name now
  // changes with the count, and the lookup by aria-label="Completed" stopped
  // finding anything, leaving focus nowhere after Clear history.
  it('returns focus to the Completed view after Clear history', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click(await screen.findByRole('button', { name: /^Completed, / }));
    fireEvent.click(screen.getByRole('button', { name: 'Clear history' }));

    await waitFor(() => expect(screen.getByRole('button', { name: /^Completed, / })).toHaveFocus());
  });
});
