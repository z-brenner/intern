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
    expect(within(dialog).getByText('Turn this off and Intern asks only when you press Check for updates.')).toBeVisible();
    fireEvent.click(toggle());
    expect(within(dialog).getByText('Off: Intern asks only when you press Check for updates.')).toBeVisible();
    expect(within(dialog).queryByText('Turn this off and Intern asks only when you press Check for updates.')).not.toBeInTheDocument();
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

  // The disabled button covers the click. The download that follows can take
  // minutes, and a rename that begins its move meanwhile would be cut off by
  // the installer closing Intern, so the install waits for it too, with the
  // queue paused so that no further rename starts.
  it('install_waits_for_a_rename_that_starts_during_the_download, with the queue paused', async () => {
    let queue: QueueItem[] = [];
    let installed = false;
    let downloaded: () => void = () => {};
    const download = new Promise<void>((resolve) => { downloaded = resolve; });
    const bridge = createInMemoryBridge({ update: available, items: [] });
    vi.spyOn(bridge, 'listItems').mockImplementation(async () => queue.map((item) => ({ ...item })));
    const pauseQueue = vi.spyOn(bridge, 'pauseQueue').mockResolvedValue();
    const resumeQueue = vi.spyOn(bridge, 'resumeQueue').mockResolvedValue();
    vi.spyOn(bridge, 'installUpdate').mockImplementation(async (onProgress, beforeInstall) => {
      queue = [applying];
      onProgress?.(1);
      downloaded();
      await beforeInstall?.();
      installed = true;
    });
    render(<App bridge={bridge} />);

    const banner = await screen.findByRole('status', { name: 'Update available' });
    fireEvent.click(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    await download;
    await waitFor(() => expect(pauseQueue).toHaveBeenCalledTimes(1));
    expect(await screen.findByRole('button', { name: 'Resume queue' })).toBeVisible();
    // Long enough for the queue to have been asked again more than once.
    await new Promise((resolve) => setTimeout(resolve, 600));
    expect(installed).toBe(false);

    queue = [{ ...applying, status: 'completed', progress: undefined, cancelable: undefined }];
    await waitFor(() => expect(installed).toBe(true));
    // Intern is about to close for the installer; nothing to resume.
    expect(resumeQueue).not.toHaveBeenCalled();
  });

  it('undoes its own pause when installing fails, and leaves a pause the person made', async () => {
    const bridge = createInMemoryBridge({ update: available, items: [] });
    // Spies that call through: the queue must really be paused and resumed.
    // A stub that only resolves leaves the in-memory queue running, and its
    // next state event then reports it unpaused - which reads to the app as
    // the person's own pause having gone away.
    const pauseQueue = vi.spyOn(bridge, 'pauseQueue');
    const resumeQueue = vi.spyOn(bridge, 'resumeQueue');
    vi.spyOn(bridge, 'installUpdate').mockImplementation(async (_onProgress, beforeInstall) => {
      await beforeInstall?.();
      throw new Error('The installer could not be started.');
    });
    render(<App bridge={bridge} />);

    let banner = await screen.findByRole('status', { name: 'Update available' });
    fireEvent.click(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    expect(await within(banner).findByRole('alert')).toHaveTextContent('The installer could not be started.');
    expect(pauseQueue).toHaveBeenCalledTimes(1);
    expect(resumeQueue).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: 'Pause queue' })).toBeVisible();

    fireEvent.click(screen.getByRole('button', { name: 'Pause queue' }));
    expect(await screen.findByRole('button', { name: 'Resume queue' })).toBeVisible();
    expect(pauseQueue).toHaveBeenCalledTimes(2);
    banner = screen.getByRole('status', { name: 'Update available' });
    fireEvent.click(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' }));
    await waitFor(() => expect(within(banner).getByRole('button', { name: 'Install 0.1.0-alpha.11 and restart' })).toBeEnabled());
    expect(pauseQueue).toHaveBeenCalledTimes(2);
    expect(resumeQueue).toHaveBeenCalledTimes(1);
    expect(screen.getByRole('button', { name: 'Resume queue' })).toBeVisible();
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
