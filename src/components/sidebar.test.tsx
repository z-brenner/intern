import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import { createInMemoryBridge } from '../lib/inMemoryBridge';
import { Sidebar } from './Sidebar';

const items = [
  { status: 'review' }, { status: 'review' }, { status: 'review' }, { status: 'review' }, { status: 'review' },
  { status: 'waiting' }, { status: 'completed' },
];

describe('queue navigation', () => {
  // At 1100px and below the labels and counts were both hidden and the nav
  // buttons had no tooltip: three bare icons. And at any width the
  // aria-label replaced the visible "Needs Review 5", so a screen reader
  // never heard the count.
  it('collapsed nav has titles and counts', () => {
    render(<Sidebar active="queue" items={items} onChange={vi.fn()} onSettings={vi.fn()} onHelp={vi.fn()} />);
    const nav = screen.getByRole('navigation', { name: 'Queue navigation' });

    for (const [view, label, count] of [['queue', 'Queue', 6], ['review', 'Needs Review', 5], ['completed', 'Completed', 1]] as const) {
      const button = within(nav).getByRole('button', { name: `${label}, ${count}` });
      expect(button).toHaveAttribute('title', label);
      expect(button).toHaveAttribute('data-view', view);
      // The badge the collapsed sidebar shows on the icon, kept out of the
      // accessible name, which already carries the count.
      const badge = button.querySelector('.nav-icon .nav-badge');
      expect(badge).toHaveTextContent(String(count));
      expect(badge).toHaveAttribute('aria-hidden', 'true');
    }
  });

  it('follows the queue as counts change', () => {
    const view = render(<Sidebar active="review" items={items} onChange={vi.fn()} onSettings={vi.fn()} onHelp={vi.fn()} />);
    view.rerender(<Sidebar active="review" items={items.slice(1)} onChange={vi.fn()} onSettings={vi.fn()} onHelp={vi.fn()} />);

    const current = screen.getByRole('button', { name: 'Needs Review, 4' });
    expect(current).toHaveAttribute('aria-current', 'page');
    expect(current.querySelector('.nav-badge')).toHaveTextContent('4');
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
