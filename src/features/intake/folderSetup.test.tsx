import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SettingsDialog } from '../../components/SettingsDialog';
import type { DesktopBridge, SelectionBoundary } from '../../lib/bridge';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from '../../lib/inMemoryBridge';
import type { AppSettings, IntakeStatus } from '../../types';
import { filedBeside, folderLabel, rootName } from './folderNames';
import { HEALTH_COPY, intakeHealth } from './IntakeHealth';

const library = 'C:\\Users\\pat\\Contoso\\Legal - Documents';
const roots = [
  { provider: 'sharepoint' as const, displayName: 'Contoso', path: library },
  { provider: 'onedrive_business' as const, displayName: 'OneDrive – Contoso', path: 'C:\\Users\\pat\\OneDrive - Contoso' },
];

function firstRun(options: InMemoryBridgeOptions = {}) {
  return createInMemoryBridge({ completedOnboardingVersion: 0, ...options });
}

function selectionPicking(path: string): SelectionBoundary {
  return {
    pickFiles: async () => [],
    pickFolder: async () => ({ path, displayName: path.split('\\').pop()! }),
    pickExistingModelFiles: async () => undefined,
    resolveDrop: async () => ({}),
  };
}

const status = (overrides: Partial<IntakeStatus> = {}): IntakeStatus => ({
  enabled: true, watching: true, folder: `${library}\\Inbox`, machineId: 'm', machineName: 'PC',
  cloud: { provider: 'sharepoint', displayName: 'Contoso' }, machines: [], heldForOthers: 0, syncConflicts: 0,
  awaitingHydration: 0, unreadableFolders: 0, claimedByOthers: 0, processedHere: 0, lastScanAt: null, error: null,
  oneDriveRunning: true, ...overrides,
});

describe('folder names', () => {
  it('names synced locations the way a person would', () => {
    expect(rootName(roots[0])).toBe('Legal - Documents (Contoso SharePoint)');
    expect(rootName(roots[1])).toBe('OneDrive – Contoso');
    expect(folderLabel(roots, `${library}\\Inbox`)).toBe('Legal - Documents (Contoso SharePoint) › Inbox');
    expect(folderLabel(roots, 'D:\\Scans')).toBe('D:\\Scans');
    expect(filedBeside(`${library}\\Inbox`)).toBe(`${library}\\Filed`);
  });
});

describe('folder health', () => {
  it('reads synced, waiting, stopped, and no longer synced from the intake status', () => {
    expect(intakeHealth(status(), true)).toBe('synced');
    expect(intakeHealth(status({ awaitingHydration: 2 }), true)).toBe('waiting');
    expect(HEALTH_COPY.waiting.message(2)).toBe('2 documents are still downloading from OneDrive.');
    expect(intakeHealth(status({ oneDriveRunning: false, awaitingHydration: 2 }), true)).toBe('onedrive_stopped');
    expect(intakeHealth(status({ cloud: null }), true)).toBe('not_synced');
    // A plain local folder has no OneDrive to report on.
    expect(intakeHealth(status({ cloud: null }), false)).toBeUndefined();
    expect(intakeHealth(status({ enabled: false }), true)).toBeUndefined();
  });

  it('offers the one action each state needs in Settings, without support codes', async () => {
    const base = createInMemoryBridge();
    const openOneDrive = vi.fn(async () => {});
    const onChooseFolder = vi.fn();
    const settings: AppSettings = { ...(await base.getSettings()), intakeEnabled: true, intakeFolder: `${library}\\Inbox`, intakeMyFolder: true };
    const bridge: DesktopBridge = { ...base, openOneDrive, intakeStatus: async () => status({ awaitingHydration: 3 }) };
    const view = render(<SettingsDialog settings={settings} bridge={bridge} onSave={async () => {}} onClose={() => {}} onCheckForUpdate={async () => ({ state: 'unsupported' })} onInstallUpdate={async () => {}} onChooseFolder={onChooseFolder} />);

    expect(await screen.findByRole('status', { name: 'Folder health' })).toHaveTextContent('Waiting for OneDrive 3 documents are still downloading from OneDrive.');
    fireEvent.click(screen.getByRole('button', { name: 'Open OneDrive' }));
    expect(openOneDrive).toHaveBeenCalledOnce();
    view.unmount();

    const gone: DesktopBridge = { ...base, intakeStatus: async () => status({ cloud: null }) };
    render(<SettingsDialog settings={settings} bridge={gone} onSave={async () => {}} onClose={() => {}} onCheckForUpdate={async () => ({ state: 'unsupported' })} onInstallUpdate={async () => {}} onChooseFolder={onChooseFolder} />);
    expect(await screen.findByRole('status', { name: 'Folder health' })).toHaveTextContent('This folder isn\'t synced by OneDrive anymore.');
    fireEvent.click(screen.getByRole('button', { name: 'Choose folder again' }));
    expect(onChooseFolder).toHaveBeenCalledOnce();
  });
});

describe('folder setup', () => {
  it('watches a synced folder in three clicks and asks once about documents already there', async () => {
    const bridge = firstRun({ existingDocuments: 4 });
    const addFolder = vi.spyOn(bridge, 'addFolder');
    const filed = vi.spyOn(bridge, 'createFiledFolder');
    render(<App bridge={bridge} selection={selectionPicking(`${library}\\Inbox`)} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    const choices = await screen.findByRole('list', { name: 'Synced folders' });
    expect(within(choices).getByText('Legal - Documents (Contoso SharePoint)')).toBeVisible();
    expect(within(choices).getAllByText('Synced by OneDrive')).toHaveLength(2);
    // The library's own top folder has nothing synced beside it, so this
    // person browses to the Inbox inside it instead.
    fireEvent.click(screen.getByRole('button', { name: 'Browse for another folder…' }));

    await screen.findByRole('heading', { name: 'Where should renamed documents go?' });
    expect(screen.queryAllByRole('textbox')).toHaveLength(0);
    fireEvent.click(screen.getByRole('button', { name: 'Create a “Filed” folder here' }));

    expect(await screen.findByText('There are 4 documents already in this folder. Rename them too?')).toBeVisible();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Only new documents' })).toHaveFocus());
    fireEvent.click(screen.getByRole('button', { name: 'Rename them' }));

    expect(await screen.findByRole('heading', { name: 'Watching Legal - Documents (Contoso SharePoint) › Inbox.' })).toBeVisible();
    expect(filed).toHaveBeenCalledWith(`${library}\\Inbox`);
    expect(addFolder).toHaveBeenCalledWith({ path: `${library}\\Inbox`, displayName: 'Inbox' });
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: `${library}\\Inbox`, destination: `${library}\\Filed`, intakeEnabled: true, intakeMyFolder: true, intakeLocalOnly: false, processOthersUploads: false });

    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect((await bridge.getOnboarding()).completedVersion).toBe(1);
  });

  it('leaves existing documents alone by default and skips the question when there are none', async () => {
    const bridge = firstRun();
    const addFolder = vi.spyOn(bridge, 'addFolder');
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Browse for another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create a “Filed” folder here' }));

    expect(await screen.findByRole('heading', { name: 'Watching D:\\Scans\\Inbox.' })).toBeVisible();
    expect(addFolder).not.toHaveBeenCalled();
    // Not synced, so it is a private local intake rather than "my folder".
    expect(await bridge.getSettings()).toMatchObject({ intakeMyFolder: false, intakeLocalOnly: true, destination: 'D:\\Scans\\Filed' });
  });

  it('does not offer a Filed folder that would sit outside the synced location', async () => {
    render(<App bridge={firstRun()} selection={selectionPicking('D:\\Elsewhere')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Use Legal - Documents (Contoso SharePoint)' }));

    expect(await screen.findByText(/would not be synced/)).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Create a “Filed” folder here' })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Choose another folder…' })).toHaveClass('primary');
  });

  it('explains how to sync a folder when none is found, and checks again', async () => {
    const base = firstRun();
    const cloudRoots = vi.fn(async () => []);
    render(<App bridge={{ ...base, cloudRoots }} selection={selectionPicking('D:\\Scans')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    expect(await screen.findByText(/We didn't find any SharePoint folders on this computer/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Browse…' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Check again' }));
    await waitFor(() => expect(cloudRoots).toHaveBeenCalledTimes(2));
  });

  it('can be skipped once, and is not offered again', async () => {
    const bridge = firstRun();
    const first = render(<App bridge={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Not now' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    first.unmount();

    render(<App bridge={bridge} />);
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Choose a folder' })).not.toBeInTheDocument();
  });

  it('is not offered to an install that already watches a folder', async () => {
    render(<App bridge={firstRun({ settings: { intakeEnabled: true, intakeFolder: 'D:\\Scans', destination: 'D:\\Filed' } })} />);
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });

  it('opens from Settings and returns to the app', async () => {
    render(<App bridge={createInMemoryBridge()} selection={selectionPicking('D:\\Scans\\Inbox')} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Settings' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Set up a folder step by step…' }));

    expect(await screen.findByRole('heading', { name: 'Choose a folder' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });
});