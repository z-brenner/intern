import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SettingsDialog } from '../../components/SettingsDialog';
import type { DesktopBridge, SelectionBoundary } from '../../lib/bridge';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from '../../lib/inMemoryBridge';
import type { AppSettings, IntakeStatus } from '../../types';
import { outputBeside, folderLabel, rootName } from './folderNames';
import { outputFolderProblem } from './folderProblems';
import { HEALTH_COPY, intakeHealth } from './IntakeHealth';

const library = 'C:\\Users\\pat\\Contoso\\Legal - Documents';
const roots = [
  { provider: 'sharepoint' as const, displayName: 'Contoso', path: library },
  { provider: 'onedrive_business' as const, displayName: 'OneDrive – Contoso', path: 'C:\\Users\\pat\\OneDrive - Contoso' },
];

/** A first run: no input or output folder chosen yet. */
function firstRun(options: InMemoryBridgeOptions = {}) {
  return createInMemoryBridge({ completedOnboardingVersion: 0, ...options, settings: { intakeFolder: '', intakeEnabled: false, intakeLocalOnly: false, destination: '', ...options.settings } });
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
    expect(folderLabel(roots, `${library}\\Input`)).toBe('Legal - Documents (Contoso SharePoint) › Input');
    expect(folderLabel(roots, 'D:\\Scans')).toBe('D:\\Scans');
    expect(outputBeside(`${library}\\Input`)).toBe(`${library}\\Output`);
  });

  // A root has nothing beside it. "E:\" used to come out as "E/Filed", and
  // a share root as a path on the server rather than on any share.
  it('offers no Output folder beside a drive, a share, or the file system root', () => {
    for (const root of ['E:\\', 'E:', 'e:/', '\\\\?\\E:\\', '\\\\server\\share', '\\\\server\\share\\', '\\\\?\\UNC\\server\\share', '/']) {
      expect(outputBeside(root), root).toBeUndefined();
    }
    expect(outputBeside('E:\\Scans')).toBe('E:\\Output');
    expect(outputBeside('E:\\Scans\\Inbox\\')).toBe('E:\\Scans\\Output');
    expect(outputBeside('\\\\server\\share\\Scans')).toBe('\\\\server\\share\\Output');
    expect(outputBeside('\\\\?\\UNC\\server\\share\\Scans')).toBe('\\\\?\\UNC\\server\\share\\Output');
    expect(outputBeside('/home/pat/Scans')).toBe('/home/pat/Output');
    expect(outputBeside('/Scans')).toBe('/Output');
  });

  it('says in plain words why a folder cannot be the output folder', () => {
    expect(outputFolderProblem('D:\\Scans', 'D:\\Scans\\Done')).toMatch(/inside your input folder/);
    expect(outputFolderProblem('D:\\Scans', 'd:\\scans\\')).toMatch(/same folder/);
    expect(outputFolderProblem('D:\\Scans', 'D:\\Output')).toBeUndefined();
    // The other way round is allowed: the input folder may sit inside the output folder.
    expect(outputFolderProblem('D:\\Output\\Input', 'D:\\Output')).toBeUndefined();
  });
});

describe('folder setup', () => {
  it('cannot be skipped: both folders are chosen, then one question about files already there', async () => {
    const bridge = firstRun({ existingDocuments: 4 });
    const queueInputFolder = vi.spyOn(bridge, 'queueInputFolder');
    const output = vi.spyOn(bridge, 'createOutputFolder');
    render(<App bridge={bridge} selection={selectionPicking(`${library}\\Inbox`)} />);

    expect(await screen.findByRole('heading', { name: 'Your documents, renamed and filed' })).toBeVisible();
    expect(screen.getByText('Put files here that you want Intern to process.')).toBeVisible();
    expect(screen.getByText('Processed files end up here, renamed.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Not now' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Choose folders' }));

    expect(await screen.findByRole('heading', { name: 'Choose your input folder' })).toBeVisible();
    const choices = await screen.findByRole('list', { name: 'Suggested folders' });
    expect(within(choices).getByText('Legal - Documents (Contoso SharePoint)')).toBeVisible();
    expect(within(choices).getAllByText('Synced by OneDrive')).toHaveLength(2);
    // The library's own top folder has nothing synced beside it, so this
    // person browses to the Inbox inside it instead.
    fireEvent.click(screen.getByRole('button', { name: 'Choose another folder…' }));

    await screen.findByRole('heading', { name: 'Choose your output folder' });
    expect(screen.queryAllByRole('textbox')).toHaveLength(0);
    fireEvent.click(screen.getByRole('button', { name: 'Create an “Output” folder next to it' }));

    expect(await screen.findByText('Your input folder already has 4 files in it. Should Intern process them too, or only files you add from now on?')).toBeVisible();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Only new files' })).toHaveFocus());
    fireEvent.click(screen.getByRole('button', { name: 'Process them too' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(screen.getByText('Legal - Documents (Contoso SharePoint) › Inbox')).toBeVisible();
    expect(screen.getByText('Legal - Documents (Contoso SharePoint) › Output')).toBeVisible();
    expect(output).toHaveBeenCalledWith(`${library}\\Inbox`);
    expect(queueInputFolder).toHaveBeenCalledOnce();
    // The output folder is in a SharePoint library, so description records
    // for its Description column are switched on with it.
    expect(screen.getByText(/leaves a short description of each document there for SharePoint’s Description column/)).toBeVisible();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: `${library}\\Inbox`, destination: `${library}\\Output`, intakeEnabled: true, intakeMyFolder: true, intakeLocalOnly: false, processOthersUploads: false, recordDescriptions: true });

    fireEvent.click(screen.getByRole('button', { name: 'Done' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect((await bridge.getOnboarding()).completedVersion).toBe(1);
  });

  it('names the documents already there that could not be queued, rather than saying all of them were', async () => {
    const bridge = firstRun({ existingDocuments: 3 });
    vi.spyOn(bridge, 'queueInputFolder').mockResolvedValue({ added: 2, alreadyQueued: 0, skipped: [{ name: 'scan-3.pdf', code: 'SOURCE_LOCKED' }] });
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create an “Output” folder next to it' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Process them too' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(screen.getByRole('note', { name: 'Files not added' })).toHaveTextContent('Added 2 documents. Skipped 1: scan-3.pdf (another program has it open). They stay in your input folder.');
  });

  it('says so when the folder itself could not be read for its existing documents', async () => {
    const bridge = firstRun({ existingDocuments: 3 });
    vi.spyOn(bridge, 'queueInputFolder').mockResolvedValue({ added: 0, alreadyQueued: 0, skipped: [{ name: 'Inbox', code: 'FOLDER_UNAVAILABLE' }] });
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create an “Output” folder next to it' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Process them too' }));

    expect(await screen.findByRole('note', { name: 'Files not added' })).toHaveTextContent('Skipped 1: Inbox (the folder could not be read).');
  });

  it('leaves existing documents alone by default and skips the question when there are none', async () => {
    const bridge = firstRun();
    const queueInputFolder = vi.spyOn(bridge, 'queueInputFolder');
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans\\Inbox')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Create an “Output” folder next to it' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(queueInputFolder).not.toHaveBeenCalled();
    // Not synced, so it is a private local intake rather than "my folder",
    // and its output folder gets no records for a SharePoint column.
    expect(await bridge.getSettings()).toMatchObject({ intakeMyFolder: false, intakeLocalOnly: true, destination: 'D:\\Scans\\Output', recordDescriptions: false });
    expect(screen.queryByText(/SharePoint’s Description column/)).not.toBeInTheDocument();
  });

  it('makes an Input and an Output folder inside a synced location chosen whole, so both stay synced', async () => {
    const bridge = firstRun({ existingDocuments: 0 });
    render(<App bridge={bridge} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Use Legal - Documents (Contoso SharePoint)' }));
    expect(screen.queryByRole('button', { name: 'Create an “Output” folder next to it' })).not.toBeInTheDocument();
    fireEvent.click(await screen.findByRole('button', { name: 'Create Input and Output folders' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(screen.getByText('Legal - Documents (Contoso SharePoint) › Input')).toBeVisible();
    expect(screen.getByText('Legal - Documents (Contoso SharePoint) › Output')).toBeVisible();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: `${library}\\Input`, destination: `${library}\\Output`, intakeMyFolder: true });
  });

  // The backend refuses to make an Output folder beside a drive's top
  // folder, so the button that asked it to could only fail, under a line
  // naming a path that cannot exist.
  it('drive root has no output folder beside it', async () => {
    const bridge = firstRun();
    const createOutputFolder = vi.spyOn(bridge, 'createOutputFolder');
    const picks = ['E:\\', 'D:\\Filed by Intern'];
    const selection = { ...selectionPicking(''), pickFolder: async () => { const path = picks.shift()!; return { path, displayName: path }; } };
    render(<App bridge={bridge} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));

    await screen.findByRole('heading', { name: 'Choose your output folder' });
    expect(screen.getByText('A drive’s top folder has nothing beside it. Choose a folder for processed files.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Create an “Output” folder next to it' })).not.toBeInTheDocument();
    expect(screen.queryByText(/New Output folder:/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Choose another folder…' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(createOutputFolder).not.toHaveBeenCalled();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: 'E:\\', destination: 'D:\\Filed by Intern' });
  });

  it('refuses an output folder inside the input folder, in plain words', async () => {
    const bridge = firstRun();
    const picks = ['D:\\Scans', 'D:\\Scans\\Done'];
    const selection = { ...selectionPicking(''), pickFolder: async () => { const path = picks.shift()!; return { path, displayName: path }; } };
    render(<App bridge={bridge} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));
    await screen.findByRole('heading', { name: 'Choose your output folder' });
    fireEvent.click(screen.getByRole('button', { name: 'Choose another folder…' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('That folder is inside your input folder, so processed files would be picked up and processed again. Choose a folder outside it.');
    expect(screen.getByRole('heading', { name: 'Choose your output folder' })).toBeVisible();
    expect((await bridge.getSettings()).intakeEnabled).toBe(false);
  });

  it('names the Output folder it will make beside an ordinary folder', async () => {
    render(<App bridge={firstRun()} selection={selectionPicking('D:\\Scans\\Inbox')} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));

    expect(await screen.findByText('New Output folder: D:\\Scans\\Output')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Create an “Output” folder next to it' })).toBeVisible();
  });

  it('offers any folder when no synced folder is found, and looks again', async () => {
    const base = firstRun();
    const cloudRoots = vi.fn(async () => []);
    render(<App bridge={{ ...base, cloudRoots }} selection={selectionPicking('D:\\Scans')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    expect(await screen.findByText(/Any folder on this computer works too/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Choose a folder…' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Look for OneDrive folders again' }));
    await waitFor(() => expect(cloudRoots).toHaveBeenCalledTimes(2));
  });

  it('is asked again on every launch until both folders are chosen', async () => {
    const bridge = firstRun();
    const first = render(<App bridge={bridge} />);
    expect(await screen.findByRole('main', { name: 'Intern setup' })).toBeInTheDocument();
    first.unmount();
    // Even with onboarding recorded as done, as an earlier release's "Not
    // now" left it: no input folder, no app.
    await bridge.completeOnboarding();
    render(<App bridge={bridge} />);
    expect(await screen.findByRole('main', { name: 'Intern setup' })).toBeInTheDocument();
    expect(screen.queryByRole('main', { name: 'Intern' })).not.toBeInTheDocument();
  });

  it('offers the output folder an existing install already had', async () => {
    const bridge = firstRun({ settings: { intakeFolder: '', intakeEnabled: false, destination: 'D:\\Filed' } });
    render(<App bridge={bridge} selection={selectionPicking('D:\\Scans')} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose folders' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Choose another folder…' }));
    expect(await screen.findByText('D:\\Filed')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Keep using this folder' }));

    expect(await screen.findByRole('heading', { name: 'You’re all set' })).toBeVisible();
    expect(await bridge.getSettings()).toMatchObject({ intakeFolder: 'D:\\Scans', destination: 'D:\\Filed', intakeEnabled: true });
  });

  it('is not offered to an install that already has both folders', async () => {
    render(<App bridge={firstRun({ settings: { intakeEnabled: true, intakeFolder: 'D:\\Scans', destination: 'D:\\Filed' } })} />);
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });

  it('opens from Settings and returns to the app', async () => {
    render(<App bridge={createInMemoryBridge()} selection={selectionPicking('D:\\Scans\\Inbox')} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Settings' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Set up folders step by step…' }));

    expect(await screen.findByRole('heading', { name: 'Choose your input folder' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });
});

describe('folders in Settings', () => {
  const openSettings = (bridge: DesktopBridge, settings: AppSettings, pickFolder: () => Promise<{ path: string; displayName: string }>, onSave = vi.fn(async (_next: AppSettings) => {})) => {
    render(<SettingsDialog settings={settings} bridge={bridge} selection={{ ...selectionPicking(''), pickFolder }} onSave={onSave} onClose={() => {}} onCheckForUpdate={async () => ({ state: 'unsupported' })} onInstallUpdate={async () => {}} />);
    return onSave;
  };

  it('shows both folders at the top with what each is for, and changes them with a picker', async () => {
    const bridge = createInMemoryBridge({ existingDocuments: 2 });
    const settings = await bridge.getSettings();
    const picks = ['D:\\Scans', 'C:\\Users\\pat\\OneDrive - Contoso\\Filed'];
    const queueInputFolder = vi.spyOn(bridge, 'queueInputFolder');
    const onSave = openSettings(bridge, settings, async () => { const path = picks.shift()!; return { path, displayName: path }; }, vi.fn(async (next: AppSettings) => { await bridge.saveSettings(next); }));

    const folders = await screen.findByRole('group', { name: 'Folders' });
    expect(within(folders).getByText('C:\\Intern\\Input')).toBeVisible();
    expect(within(folders).getByText('Put files here that you want Intern to process.')).toBeVisible();
    expect(within(folders).getByText('C:\\Intern\\Output')).toBeVisible();
    expect(within(folders).getByText('Processed files end up here, renamed.')).toBeVisible();
    expect(screen.queryByRole('checkbox', { name: 'Watch a folder for new documents' })).not.toBeInTheDocument();
    expect(within(folders).queryAllByRole('textbox')).toHaveLength(0);

    fireEvent.click(within(folders).getByRole('button', { name: 'Change input folder' }));
    expect(await within(folders).findByText('D:\\Scans')).toBeVisible();
    // The new folder already has documents: setup's question, only new files unless changed.
    const question = await within(folders).findByRole('group', { name: 'This folder already has 2 files in it. Process them too?' });
    expect(within(question).getByRole('radio', { name: 'Only new files' })).toBeChecked();
    fireEvent.click(within(question).getByRole('radio', { name: 'Process them too' }));

    fireEvent.click(within(folders).getByRole('button', { name: 'Change output folder' }));
    expect(await within(folders).findByText('C:\\Users\\pat\\OneDrive - Contoso\\Filed')).toBeVisible();
    // An output folder OneDrive syncs gets records for SharePoint's Description column.
    await waitFor(() => expect(screen.getByRole('checkbox', { name: 'Write a description record for each filed document' })).toBeChecked());

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(queueInputFolder).toHaveBeenCalledOnce());
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ intakeFolder: 'D:\\Scans', intakeEnabled: true, intakeLocalOnly: true, intakeMyFolder: false, destination: 'C:\\Users\\pat\\OneDrive - Contoso\\Filed', recordDescriptions: true }));
  });

  it('refuses an output folder inside the input folder, and an input folder around the output folder', async () => {
    const bridge = createInMemoryBridge();
    const settings = await bridge.getSettings();
    const picks = ['C:\\Intern\\Input\\Done', 'C:\\Intern'];
    const onSave = openSettings(bridge, settings, async () => { const path = picks.shift()!; return { path, displayName: path }; });
    const folders = await screen.findByRole('group', { name: 'Folders' });

    fireEvent.click(within(folders).getByRole('button', { name: 'Change output folder' }));
    expect(await within(folders).findByRole('alert')).toHaveTextContent('That folder is inside your input folder');
    expect(within(folders).getByText('C:\\Intern\\Output')).toBeVisible();

    fireEvent.click(within(folders).getByRole('button', { name: 'Change input folder' }));
    expect(await within(folders).findByRole('alert')).toHaveTextContent('Your output folder is inside that folder');
    expect(within(folders).getByText('C:\\Intern\\Input')).toBeVisible();
    expect(onSave).not.toHaveBeenCalled();
  });

  it('folds the shared-folder options away under Advanced', async () => {
    const bridge = createInMemoryBridge();
    openSettings(bridge, await bridge.getSettings(), async () => ({ path: 'D:\\Scans', displayName: 'Scans' }));
    const advanced = (await screen.findByText('Advanced: shared folders and support details')).closest('details')!;
    expect(advanced).not.toHaveAttribute('open');
    expect(within(advanced).getByRole('checkbox', { name: 'Also process documents uploaded by others', hidden: true })).toBeInTheDocument();
    expect(within(advanced).getByLabelText('This machine\'s name', { selector: 'input' })).toBeInTheDocument();
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
