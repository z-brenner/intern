import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

describe('show only ready', () => {
  it('narrows the queue to what Rename all ready would rename, and back', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const toggle = await screen.findByRole('button', { name: '3 ready' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getAllByRole('row')).toHaveLength(9);

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows).toHaveLength(3);
    for (const row of rows) expect(row).toHaveTextContent('Ready');
    expect(screen.getByText('3 of 8 items')).toBeInTheDocument();

    fireEvent.click(toggle);
    expect(screen.getAllByRole('row')).toHaveLength(9);
  });
});
