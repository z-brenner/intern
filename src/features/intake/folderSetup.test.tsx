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

  // A root has nothing beside it. "E:\" used to come out as "E/Filed", and
  // a share root as a path on the server rather than on any share.
  it('offers no Filed folder beside a drive, a share, or the file system root', () => {
    for (const root of ['E:\\', 'E:', 'e:/', '\\\\?\\E:\\', '\\\\server\\share', '\\\\server\\share\\', '\\\\?\\UNC\\server\\share', '/']) {
      expect(filedBeside(root), root).toBeUndefined();
    }
    expect(filedBeside('E:\\Scans')).toBe('E:\\Filed');
    expect(filedBeside('E:\\Scans\\Inbox\\')).toBe('E:\\Scans\\Filed');
    expect(filedBeside('\\\\server\\share\\Scans')).toBe('\\\\server\\share\\Filed');
    expect(filedBeside('\\\\?\\UNC\\server\\share\\Scans')).toBe('\\\\?\\UNC\\server\\share\\Filed');
    expect(filedBeside('/home/pat/Scans')).toBe('/home/pat/Filed');
    expect(filedBeside('/Scans')).toBe('/Filed');
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

  it('names the documents already there that could not be added, rather than saying all of them are', async () => {
    const bridge = firstRun({ existingDocuments: 3 });
    vi.spyOn(bridge, 'addFolder').mockResolvedValue({ added: 2, alreadyQueued: 0, skipped: [{ name: 'scan-3.pdf', code: 'SOURCE_LOCKED' }] });
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Browse for another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create a “Filed” folder here' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Rename them' }));

    expect(await screen.findByRole('heading', { name: 'Watching D:\\Scans\\Inbox.' })).toBeVisible();
    expect(screen.getByRole('note', { name: 'Files not added' })).toHaveTextContent('Added 2 documents. Skipped 1: scan-3.pdf (another program has it open). You can add them from the queue later.');
  });

  it('says so when the folder itself could not be read for its existing documents', async () => {
    const bridge = firstRun({ existingDocuments: 3 });
    vi.spyOn(bridge, 'addFolder').mockResolvedValue({ added: 0, alreadyQueued: 0, skipped: [{ name: 'Inbox', code: 'FOLDER_UNAVAILABLE' }] });
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Browse for another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create a “Filed” folder here' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Rename them' }));

    expect(await screen.findByRole('note', { name: 'Files not added' })).toHaveTextContent('Skipped 1: Inbox (the folder could not be read).');
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

  it('makes an Inbox and a Filed folder inside a synced location chosen whole, so both stay synced', async () => {
    const bridge = firstRun({ existingDocuments: 0 });
    render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Use Legal - Documents (Contoso SharePoint)' }));
    expect(screen.queryByRole('button', { name: 'Create a “Filed” folder here' })).not.toBeInTheDocument();
    fireEvent.click(await screen.findByRole('button', { name: 'Create Inbox and Filed folders' }));

    expect(await screen.findByRole('heading', { name: 'Watching Legal - Documents (Contoso SharePoint) › Inbox.' })).toBeVisible();
    expect(screen.getByText('Renamed documents go to Legal - Documents (Contoso SharePoint) › Filed.')).toBeVisible();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: `${library}\\Inbox`, destination: `${library}\\Filed`, intakeMyFolder: true });
  });

  // The backend refuses to make a Filed folder beside a drive's top folder,
  // so the button that asked it to could only fail, under a "Filed folder:"
  // line naming a path that cannot exist.
  it('drive root has no filed folder beside it', async () => {
    const bridge = firstRun();
    const createFiledFolder = vi.spyOn(bridge, 'createFiledFolder');
    const picks = ['E:\\', 'D:\\Filed by Intern'];
    const selection = { ...selectionPicking(''), pickFolder: async () => { const path = picks.shift()!; return { path, displayName: path }; } };
    render(<App bridge={bridge} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Browse for another folder…' }));

    await screen.findByRole('heading', { name: 'Where should renamed documents go?' });
    expect(screen.getByText('A drive\'s top folder has nothing beside it. Choose where renamed documents should go.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Create a “Filed” folder here' })).not.toBeInTheDocument();
    expect(screen.queryByText(/Filed folder:/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Choose another folder…' }));

    expect(await screen.findByRole('heading', { name: 'Watching E:\\.' })).toBeVisible();
    expect(createFiledFolder).not.toHaveBeenCalled();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: 'E:\\', destination: 'D:\\Filed by Intern' });
  });

  it('names the Filed folder it will make beside an ordinary folder', async () => {
    render(<App bridge={firstRun()} selection={selectionPicking('D:\\Scans\\Inbox')} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Choose a folder' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Browse for another folder…' }));

    expect(await screen.findByText('Filed folder: D:\\Scans\\Filed')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Create a “Filed” folder here' })).toBeVisible();
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