import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';
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
    expect(await bridge.getSettings()).toEqual(alpha9Settings);

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
    expect(await bridge.getSettings()).toEqual(alpha9Settings);
    expect(await bridge.listItems()).toEqual(alpha9Items);
  });
});
