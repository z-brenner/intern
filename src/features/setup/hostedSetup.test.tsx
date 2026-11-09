import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

describe('local model setup', () => {
  it('offers the local download without an API-key alternative', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0 } })} />);
    const setup = await screen.findByRole('main', { name: 'Intern setup' });
    expect(within(setup).getByRole('button', { name: 'Download model' })).toBeVisible();
    expect(within(setup).queryByRole('button', { name: 'Set up a hosted model' })).not.toBeInTheDocument();
    expect(setup).not.toHaveTextContent(/API key/i);
  });
});
