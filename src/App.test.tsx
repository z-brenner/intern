import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { createInMemoryBridge } from './lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from './lib/inMemoryBridge';
import { createTauriSelectionBoundary } from './lib/tauriBridge';
import type { TauriEvent, TauriTransport } from './lib/tauriBridge';
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
  The desktop window, end to end from the runtime's own events: Tauri takes
  every drop (dragDropEnabled), so these raw window events are the only way a
  dragged file reaches the queue - and every one used to be discarded.
*/
describe('desktop drag and drop', () => {
  function windowEvents() {
    const handlers = new Map<string, Array<(event: TauriEvent<unknown>) => void>>();
    const transport: TauriTransport = {
      invoke: async <T,>() => undefined as T,
      listen: async <T,>(event: string, handler: (event: TauriEvent<T>) => void) => {
        const list = handlers.get(event) ?? [];
        list.push(handler as (event: TauriEvent<unknown>) => void);
        handlers.set(event, list);
        return () => { handlers.set(event, (handlers.get(event) ?? []).filter((entry) => entry !== handler)); };
      },
    };
    const emit = (event: string, payload: unknown) => act(() => { for (const handler of handlers.get(event) ?? []) handler({ event, id: 1, payload }); });
    const listening = (event: string) => (handlers.get(event)?.length ?? 0) > 0;
    return { transport, emit, listening };
  }

  it('queues a file from the raw drop payload and shows how many are coming while dragging', async () => {
    const desktop = windowEvents();
    const bridge = createInMemoryBridge({ items: [] });
    const addFiles = vi.spyOn(bridge, 'addFiles');
    render(<App bridge={bridge} selection={createTauriSelectionBoundary(desktop.transport)} />);
    await screen.findByRole('main', { name: 'Intern' });
    await waitFor(() => expect(desktop.listening('tauri://drag-enter') && desktop.listening('tauri://drag-drop')).toBe(true));

    desktop.emit('tauri://drag-enter', { paths: ['C:/Docs/a.pdf'], position: { x: 1, y: 2 } });
    expect(screen.getByText('Drop to add 1 file')).toBeVisible();
    desktop.emit('tauri://drag-leave', null);
    expect(screen.queryByText(/Drop to add/)).not.toBeInTheDocument();

    desktop.emit('tauri://drag-enter', { paths: ['C:/Docs/a.pdf', 'C:/Docs/Scans'], position: { x: 1, y: 2 } });
    expect(screen.getByText('Drop to add 2 files')).toBeVisible();
    desktop.emit('tauri://drag-leave', null);

    desktop.emit('tauri://drag-enter', { paths: ['C:/Docs/a.pdf'], position: { x: 1, y: 2 } });
    desktop.emit('tauri://drag-drop', { paths: ['C:/Docs/a.pdf'], position: { x: 1, y: 2 } });

    expect(screen.queryByText(/Drop to add/)).not.toBeInTheDocument();
    expect(await screen.findByRole('row', { name: /a\.pdf/ })).toBeVisible();
    expect(addFiles).toHaveBeenCalledWith([{ path: 'C:/Docs/a.pdf', displayName: 'a.pdf' }]);
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
