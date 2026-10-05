import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App, UPDATE_POLL_INTERVAL_MS } from './App';
import { createInMemoryBridge } from './lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from './lib/inMemoryBridge';
import type { AppSettings, QueueItem } from './types';

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

/*
  The automatic check is the one request Intern makes that nobody asked for at
  that moment. Some offices allow none, so Settings can switch it off, and off
  has to mean off from the first instant: not one check at launch before the
  settings file has been read, and none when the six-hour timer comes round.
*/
describe('automatic update checks', () => {
  afterEach(() => { vi.useRealTimers(); });

  const current = { state: 'current' as const, currentVersion: '0.1.0-alpha.10' };

  it('no_update_check_when_disabled: neither at launch nor on the timer, while the button still checks', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = createInMemoryBridge({ settings: { skipUpdateChecks: true } });
    const checkForUpdate = vi.spyOn(bridge, 'checkForUpdate').mockResolvedValue(current);
    render(<App bridge={bridge} />);

    // Settings has the saved choice, so the app has read it by now.
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(within(dialog).getByLabelText(/^Check for updates automatically/)).not.toBeChecked();
    await act(async () => { await vi.advanceTimersByTimeAsync(UPDATE_POLL_INTERVAL_MS * 2 + 1000); });
    expect(checkForUpdate).not.toHaveBeenCalled();

    fireEvent.click(within(dialog).getByRole('button', { name: 'Check for updates' }));
    await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(1));
    expect(await within(dialog).findByRole('status', { name: 'Update status' })).toHaveTextContent('0.1.0-alpha.10 is the latest release');
  });

  it('check_runs_at_launch_and_on_timer_when_enabled', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const checkForUpdate = vi.fn(async () => current);
    render(<App bridge={{ ...createInMemoryBridge(), checkForUpdate }} />);

    await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(1));
    await act(async () => { await vi.advanceTimersByTimeAsync(UPDATE_POLL_INTERVAL_MS - 60_000); });
    expect(checkForUpdate).toHaveBeenCalledTimes(1);
    await act(async () => { await vi.advanceTimersByTimeAsync(60_000); });
    await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(2));
  });

  it('stops when switched off in Settings, and checks again as soon as it is switched back on', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = createInMemoryBridge();
    const checkForUpdate = vi.spyOn(bridge, 'checkForUpdate').mockResolvedValue(current);
    render(<App bridge={bridge} />);
    await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(1));

    const toggle = async () => {
      fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      fireEvent.click(within(dialog).getByLabelText(/^Check for updates automatically/));
      fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
      await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    };

    await toggle();
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(true);
    await act(async () => { await vi.advanceTimersByTimeAsync(UPDATE_POLL_INTERVAL_MS * 2); });
    expect(checkForUpdate).toHaveBeenCalledTimes(1);

    await toggle();
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(false);
    await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(2));
  });
});

/*
  An install updated from 0.1.0-alpha.9: no onboarding has ever been recorded
  (completedVersion 0), the person chose a manual intake folder and destination
  in Settings, the queue holds finished and pending work, and the local model is
  already downloaded.
*/
const alpha9Settings: AppSettings = {
  destination: 'D:\\Filed by Intern', destinationLayout: 'year', startMinimized: false, automaticRename: true,
  intakeFolder: 'D:\\Scans to file', intakeEnabled: true, intakeLocalOnly: false, processOthersUploads: false, machineLabel: 'Front desk',
  runInBackground: false, startAtLogin: false, recordDescriptions: true,
  modelSource: 'local', hostedProvider: 'openai_compatible', hostedBaseUrl: 'https://models.example.test/v1', hostedModel: 'example-model',
};
const alpha9Items: QueueItem[] = [
  { id: 'alpha9-filed', originalFilename: 'scan-0001.pdf', status: 'completed', proposedFilename: '2025-03-04 Invoice from Northwind.pdf', confidence: 0.94, undoable: true },
  { id: 'alpha9-review', originalFilename: 'scan-0002.pdf', status: 'review', proposedFilename: '2025-03-05 Lease Amendment.pdf', confidence: 0.61 },
  { id: 'alpha9-waiting', originalFilename: 'scan-0003.pdf', status: 'waiting' },
];

/*
  The same file as this build reads it back: a field added since alpha.9 takes
  its default, and the update switch's default is checks on.
*/
const alpha9SettingsAsRead: AppSettings = { ...alpha9Settings, skipUpdateChecks: false };

function alpha9Bridge(options: InMemoryBridgeOptions = {}) {
  return createInMemoryBridge({ settings: alpha9Settings, items: alpha9Items, completedOnboardingVersion: 0, ...options });
}

describe('updating from alpha.9', () => {
  it('shows onboarding once with the deployment enabled, and activation keeps the queue, history, model, and unrelated settings', async () => {
    const bridge = alpha9Bridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'ready_to_activate' } });
    const history = await bridge.historyList();
    const first = render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Set up Intern' }));
    // The model is already downloaded, so setup goes straight to Microsoft.
    fireEvent.click(await screen.findByRole('button', { name: 'Yes, this is my account' }));
    // Nothing is overwritten before the person turns filing on.
    await screen.findByRole('heading', { name: 'Turn on filing' });
    expect(await bridge.getSettings()).toEqual(alpha9SettingsAsRead);

    fireEvent.click(screen.getByRole('button', { name: 'Turn on filing' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Open Intern' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(await screen.findByRole('row', { name: /scan-0002.pdf/ })).toBeInTheDocument();

    expect(await bridge.listItems()).toEqual(alpha9Items);
    expect(await bridge.historyList()).toEqual(history);
    expect(await bridge.getSetup()).toMatchObject({ state: 'ready' });
    const settings = await bridge.getSettings();
    expect(settings).toMatchObject({
      destinationLayout: 'year', automaticRename: true, recordDescriptions: true, machineLabel: 'Front desk',
      modelSource: 'local', hostedProvider: 'openai_compatible', hostedBaseUrl: 'https://models.example.test/v1', hostedModel: 'example-model',
    });
    // The manual folders give way to the managed library only on activation.
    expect(settings).toMatchObject({ intakeEnabled: true, processOthersUploads: false, intakeLocalOnly: false, runInBackground: true, startAtLogin: true, startMinimized: true });
    expect(settings.intakeFolder).toMatch(/InternTestSite - Files\\Inbox$/);
    expect(settings.destination).toMatch(/InternTestSite - Files\\Filed$/);
    first.unmount();

    // Completion survives a remount: the next launch opens the app directly.
    render(<App bridge={bridge} />);
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('opens straight into the app for someone who already completed onboarding', async () => {
    const bridge = alpha9Bridge({ sharePoint: 'fake', completedOnboardingVersion: 1, sharePointFake: { connected: true, phase: 'active' } });
    render(<App bridge={bridge} />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(await screen.findByRole('row', { name: /scan-0002.pdf/ })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('shows no onboarding when the deployment is disabled, and leaves their settings alone', async () => {
    const bridge = alpha9Bridge();
    render(<App bridge={bridge} />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(await screen.findByRole('row', { name: /scan-0002.pdf/ })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
    expect(screen.queryByRole('list', { name: 'Setup steps' })).not.toBeInTheDocument();
    expect(await bridge.getSettings()).toEqual(alpha9SettingsAsRead);
    expect(await bridge.listItems()).toEqual(alpha9Items);
  });
});
