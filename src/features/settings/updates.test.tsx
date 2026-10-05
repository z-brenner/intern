import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import type { UpdateProgressListener } from '../../lib/bridge';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';

const available = { state: 'available' as const, currentVersion: '0.1.0-alpha.10', version: '0.1.0-alpha.11' };

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  return screen.findByRole('dialog', { name: 'Settings' });
}

/** The atomic apply stage: the one stage of a rename that cannot be canceled. */
const applying: QueueItem = { id: 'applying', originalFilename: 'applying.pdf', status: 'processing', progress: 90, cancelable: false };

describe('the update settings', () => {
  it('toggle_round_trips: automatic checks are on by default, and the choice is saved either way', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);

    let dialog = await openSettings();
    const toggle = () => within(dialog).getByLabelText('Check for updates automatically (when Intern starts and every 6 hours)');
    expect(toggle()).toBeChecked();
    fireEvent.click(toggle());
    expect(within(dialog).getByText(/Intern asks only when you press Check for updates/)).toBeVisible();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(true);

    dialog = await openSettings();
    expect(toggle()).not.toBeChecked();
    fireEvent.click(toggle());
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(false);
  });

  // Installing closes Intern for the installer. A rename part-way through its
  // move cannot be canceled, so Install waits for it rather than cutting it off.
  it('install_disabled_while_applying: in the banner and in Settings, with the reason shown', async () => {
    const bridge = createInMemoryBridge({ update: available, items: [applying] });
    const installUpdate = vi.spyOn(bridge, 'installUpdate').mockResolvedValue();
    render(<App bridge={bridge} />);

    const banner = await screen.findByRole('status', { name: 'Update available' });
    expect(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' })).toBeDisabled();
    expect(within(banner).getByText('Waiting for a rename to finish')).toBeVisible();

    const dialog = await openSettings();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Check for updates' }));
    expect(await within(dialog).findByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' })).toBeDisabled();
    expect(within(dialog).getByText('Waiting for a rename to finish')).toBeVisible();
    expect(installUpdate).not.toHaveBeenCalled();
  });

  it('allows installing while other work is only processing, since that can be canceled and picked up again', async () => {
    const analyzing: QueueItem = { id: 'analyzing', originalFilename: 'analyzing.pdf', status: 'processing', progress: 40, cancelable: true };
    const bridge = createInMemoryBridge({ update: available, items: [analyzing] });
    const installUpdate = vi.spyOn(bridge, 'installUpdate').mockResolvedValue();
    render(<App bridge={bridge} />);

    const banner = await screen.findByRole('status', { name: 'Update available' });
    expect(within(banner).queryByText('Waiting for a rename to finish')).not.toBeInTheDocument();
    fireEvent.click(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    await waitFor(() => expect(installUpdate).toHaveBeenCalledTimes(1));
  });

  it('download_progress_rendered: the banner and Settings say how much has arrived', async () => {
    let report: UpdateProgressListener | undefined;
    const bridge = createInMemoryBridge({ update: available });
    // Never resolves: on success the installer closes Intern, so the label is
    // all a person sees until then.
    vi.spyOn(bridge, 'installUpdate').mockImplementation((onProgress) => { report = onProgress; return new Promise<void>(() => {}); });
    const view = render(<App bridge={bridge} />);

    const banner = await screen.findByRole('status', { name: 'Update available' });
    fireEvent.click(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    expect(await within(banner).findByRole('button', { name: 'Installing…' })).toBeDisabled();
    report!(0.42);
    expect(await within(banner).findByRole('button', { name: 'Downloading 42%…' })).toBeDisabled();
    // A server that sent no size gets no invented percentage.
    report!(undefined);
    expect(await within(banner).findByRole('button', { name: 'Downloading…' })).toBeDisabled();
    // Every byte is in; what remains is the installer's own work.
    report!(1);
    expect(await within(banner).findByRole('button', { name: 'Installing…' })).toBeDisabled();
    view.unmount();

    // The same from Settings' own Install button.
    render(<App bridge={bridge} />);
    const dialog = await openSettings();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Check for updates' }));
    fireEvent.click(await within(dialog).findByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    report!(0.996);
    // Held at 99% rather than rounded up to a "100%" that is still arriving.
    expect(await within(dialog).findByRole('button', { name: 'Downloading 99%…' })).toBeDisabled();
  });
});
