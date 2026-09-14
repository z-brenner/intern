import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { createInMemoryBridge } from './lib/inMemoryBridge';

describe('App', () => {
  it('exposes the Intern application landmark', () => {
    render(<App />);

    expect(screen.getByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });

  it('exposes one unambiguous Settings action', async () => {
    render(<App />);

    expect(await screen.findAllByRole('button', { name: 'Settings' })).toHaveLength(1);
  });

  it('checks for an update on its own, with nobody opening Settings', async () => {
    const bridge = createInMemoryBridge({
      update: { state: 'available', currentVersion: '0.1.0-alpha.9', version: '0.1.0-alpha.10' },
    });
    const checkForUpdate = vi.spyOn(bridge, 'checkForUpdate');

    render(<App bridge={bridge} />);

    await waitFor(() => expect(checkForUpdate).toHaveBeenCalled());
    expect(await screen.findByRole('status', { name: 'Update available' })).toHaveTextContent('0.1.0-alpha.10');
  });

  it('installs the update that was found', async () => {
    const bridge = createInMemoryBridge({
      update: { state: 'available', currentVersion: '0.1.0-alpha.9', version: '0.1.0-alpha.10' },
    });
    const installUpdate = vi.spyOn(bridge, 'installUpdate').mockResolvedValue();

    render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: /^Install 0\.1\.0-alpha\.10/ }));
    await waitFor(() => expect(installUpdate).toHaveBeenCalled());
  });

  it('can be dismissed instead', async () => {
    const bridge = createInMemoryBridge({
      update: { state: 'available', currentVersion: '0.1.0-alpha.9', version: '0.1.0-alpha.10' },
    });

    render(<App bridge={bridge} />);

    const banner = await screen.findByRole('status', { name: 'Update available' });
    fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
    expect(banner).not.toBeInTheDocument();
  });
});
