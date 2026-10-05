import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { SettingsDialog } from '../../components/SettingsDialog';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { DesktopBridge } from '../../lib/bridge';
import type { AppSettings, IntakeStatus, OnboardingStatus, SharePointSetupStatus } from '../../types';
import type { MicrosoftAccount, MicrosoftIntakeStatus } from '../intake/microsoft';
import { describeSharePointProblem } from '../sharepoint/sharePointProblems';

const account: MicrosoftAccount = { tenantId: 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', id: 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', displayName: 'Pat Doe', email: 'pat@contoso.test' };
const active: SharePointSetupStatus = { phase: 'active', account: { displayName: 'Pat Doe', email: 'pat@contoso.test' }, site: 'InternTestSite', library: 'Files', intake: 'Inbox', destination: 'Filed' };
const managedSettings: AppSettings = {
  destination: 'C:\\Users\\pat\\Contoso\\InternTestSite - Files\\Filed', destinationLayout: 'flat', startMinimized: true, automaticRename: false,
  intakeFolder: 'C:\\Users\\pat\\Contoso\\InternTestSite - Files\\Inbox', intakeEnabled: true, intakeLocalOnly: false, processOthersUploads: false,
  machineLabel: 'Front desk', runInBackground: true, startAtLogin: true, recordDescriptions: false,
  modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '',
};
const watching: IntakeStatus = {
  enabled: true, watching: true, folder: managedSettings.intakeFolder, machineId: 'm1', machineName: 'PAT-LAPTOP', cloud: { provider: 'sharepoint', displayName: 'Contoso' },
  machines: [], heldForOthers: 1, uploaderUnknown: 2, syncConflicts: 3, awaitingHydration: 4, unreadableFolders: 0, claimedByOthers: 0, processedHere: 5,
  lastScanAt: 1_700_000_000, error: null,
};
const connected: MicrosoftIntakeStatus = {
  connected: true, account, error: null,
  binding: { localFolder: managedSettings.intakeFolder, driveId: 'drive-123', folderId: 'folder-456', webUrl: 'https://contoso.sharepoint.com/sites/InternTestSite/Files/Inbox', tenantId: account.tenantId },
  documents: [
    { path: 'a', filename: 'agreement.pdf', state: 'filed', reason: 'Fresh upload verified.', uploader: account, processedBy: account, filedAs: '2026-01-02 Agreement.pdf', checkedAt: 0 },
    { path: 'b', filename: 'copied.pdf', state: 'unknown', reason: 'Upload metadata did not match a fresh upload.', uploader: null, processedBy: null, filedAs: null, checkedAt: 0 },
  ],
};

function managedBridge(overrides: Partial<DesktopBridge> = {}) {
  const base = createInMemoryBridge();
  let microsoft = connected;
  const bridge = {
    ...base,
    getOnboarding: vi.fn(async () => ({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: true })),
    getSharePointSetup: vi.fn(async (): Promise<SharePointSetupStatus> => active),
    intakeStatus: vi.fn(async () => watching),
    scanIntakeNow: vi.fn(async () => {}),
    microsoftIntakeStatus: vi.fn(async () => microsoft),
    microsoftSignInStart: vi.fn(async () => { microsoft = { ...microsoft, connected: false, account: null }; return { userCode: 'WXYZ-1234', verificationUri: 'https://microsoft.com/devicelogin', intervalSeconds: 5, expiresAt: 9_999_999_999 }; }),
    microsoftSignInPoll: vi.fn(async () => { microsoft = { ...connected }; return { state: 'connected' as const, account }; }),
    microsoftDisconnect: vi.fn(async () => { microsoft = { ...microsoft, connected: false, account: null }; }),
    microsoftBindIntake: vi.fn(async () => { throw new Error('not used'); }),
    microsoftOpenSignIn: vi.fn(async () => {}),
    saveSettings: vi.fn(async () => {}),
    ...overrides,
  };
  return bridge;
}

function renderDialog(bridge: DesktopBridge, settings: AppSettings = managedSettings, onSave = vi.fn(async () => {})) {
  render(<SettingsDialog settings={settings} bridge={bridge} onSave={onSave} onClose={() => {}} onCheckForUpdate={async () => ({ state: 'unsupported' })} onInstallUpdate={async () => {}} />);
  return { onSave };
}

afterEach(() => { vi.useRealTimers(); });

describe('SharePoint connection in Settings', () => {
  it('shows the fixed library, account, and health instead of manual shared-intake controls', async () => {
    renderDialog(managedBridge());
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });

    expect(within(card).getByText('InternTestSite')).toBeVisible();
    expect(within(card).getByText('Files')).toBeVisible();
    expect(within(card).getByText('Inbox')).toBeVisible();
    expect(within(card).getByText('Filed')).toBeVisible();
    expect(await within(card).findByText('Pat Doe')).toBeVisible();
    expect(within(card).getByText('pat@contoso.test')).toBeVisible();
    expect(await within(card).findByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/active/i);
    expect(await within(card).findByRole('status', { name: 'Watcher health' })).toHaveTextContent(/watching inbox/i);
    expect(within(card).getByRole('status', { name: 'Background status' })).toHaveTextContent(/runs in the background: on/i);
    expect(within(card).getByRole('status', { name: 'Background status' })).toHaveTextContent(/starts when you sign in: on/i);
    expect(within(card).getByRole('button', { name: 'Reconnect Microsoft' })).toBeEnabled();
    expect(within(card).getByRole('button', { name: 'Check again' })).toBeEnabled();
    expect(within(card).getByText('Support details')).toBeVisible();

    // Nothing that would let a person point Intern somewhere else, or weaken the policy.
    expect(screen.queryByLabelText('Destination folder')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Intake folder')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /browse for/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'Synced locations on this computer' })).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Watch a folder for new documents')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Also process documents uploaded by others')).not.toBeInTheDocument();
    expect(screen.queryByLabelText("This machine's name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/private local intake/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Run in background')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Start Intern when you sign in')).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/tenant|client|application ID|drive|folder ID/i)).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Verify folder pairing' })).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(screen.queryByText(account.tenantId)).not.toBeVisible();
  });

  it('keeps model, spellings, layout, renaming, descriptions, updates, and help', async () => {
    renderDialog(managedBridge());
    await screen.findByRole('region', { name: 'SharePoint connection' });
    expect(screen.getByLabelText('Arrange filed documents')).toBeVisible();
    expect(screen.getByLabelText('Automatically rename high-confidence files')).toBeVisible();
    expect(screen.getByText('Spellings Intern has learned')).toBeVisible();
    expect(screen.getByLabelText(/local model on this computer/i)).toBeVisible();
    expect(screen.getByLabelText('Write a description record for each filed document')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Check for updates' })).toBeVisible();
    expect(screen.getByRole('button', { name: /open the guide/i })).toBeVisible();
  });

  it('keeps support details collapsed until asked, then shows codes, scans, counts, and reasons', async () => {
    renderDialog(managedBridge({ getSharePointSetup: vi.fn(async () => { throw { code: 'SHAREPOINT_SYNC_PENDING', message: 'The fixed SharePoint library is not synced yet.' }; }) }));
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    await within(card).findByRole('alert');
    const details = within(card).getByRole('group', { name: 'Support details' });
    expect(within(details).getByText('SHAREPOINT_SYNC_PENDING')).not.toBeVisible();

    fireEvent.click(within(card).getByText('Support details'));

    expect(within(details).getByText('SHAREPOINT_SYNC_PENDING')).toBeVisible();
    expect(within(details).getByText(/1 verified · 1 held/)).toBeVisible();
    expect(within(details).getByText('Upload metadata did not match a fresh upload.')).toBeVisible();
    expect(within(details).getByText(/3 sync conflicts/)).toBeVisible();
    expect(within(details).getByText(/4 waiting for OneDrive to download/)).toBeVisible();
    expect(within(details).getByText(new Date(watching.lastScanAt! * 1000).toLocaleString())).toBeVisible();
    expect(within(details).getByText(account.id)).toBeVisible();
  });

  it('maps setup failures to an action-first sentence and keeps the code for support', async () => {
    renderDialog(managedBridge({ getSharePointSetup: vi.fn(async () => { throw { code: 'ONEDRIVE_MISSING', message: 'OneDrive is not installed.' }; }) }));
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    const alert = await within(card).findByRole('alert');
    expect(alert).toHaveTextContent(describeSharePointProblem('ONEDRIVE_MISSING').action);
    expect(alert).not.toHaveTextContent('ONEDRIVE_MISSING');
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/needs attention/i);
  });

  it('offers SharePoint and the OneDrive download through the desktop shell when setup needs attention', async () => {
    const openSupportLink = vi.fn(async () => {});
    renderDialog(managedBridge({ openSupportLink, getSharePointSetup: vi.fn(async () => { throw { code: 'ONEDRIVE_MISSING', message: 'OneDrive is not installed.' }; }) }));
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    await within(card).findByRole('alert');

    expect(within(card).queryByRole('link')).not.toBeInTheDocument();
    fireEvent.click(within(card).getByRole('button', { name: 'Open SharePoint' }));
    fireEvent.click(within(card).getByRole('button', { name: 'Get OneDrive' }));
    await waitFor(() => expect(openSupportLink.mock.calls).toEqual([['sharepoint-site'], ['onedrive-download']]));
  });

  it('keeps the recovery links out of the way while the connection is healthy', async () => {
    renderDialog(managedBridge());
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    await within(card).findByText(/Active\./);

    expect(within(card).queryByRole('button', { name: 'Open SharePoint' })).not.toBeInTheDocument();
    expect(within(card).queryByRole('button', { name: 'Get OneDrive' })).not.toBeInTheDocument();
  });

  it('falls back to a generic message for codes it does not know', () => {
    const generic = describeSharePointProblem('SOMETHING_NEW').action;
    expect(generic).toMatch(/try again/i);
    expect(generic).not.toContain('SOMETHING_NEW');
    expect(describeSharePointProblem('MICROSOFT_ACCOUNT_WRONG_TENANT').action).toMatch(/Sign in with your work account/);
    expect(describeSharePointProblem('SHAREPOINT_SYNC_PENDING').action).not.toBe(generic);
  });

  it('checks the setup and watcher again on request', async () => {
    const bridge = managedBridge({ intakeStatus: vi.fn().mockResolvedValueOnce({ ...watching, watching: false }).mockResolvedValue(watching) });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    // The dialog's own intake subscription may also have read the status once.
    await waitFor(() => expect(bridge.getSharePointSetup).toHaveBeenCalledTimes(1));
    const setupCalls = vi.mocked(bridge.getSharePointSetup).mock.calls.length;
    const intakeCalls = vi.mocked(bridge.intakeStatus).mock.calls.length;

    fireEvent.click(within(card).getByRole('button', { name: 'Check again' }));

    await waitFor(() => expect(bridge.getSharePointSetup).toHaveBeenCalledTimes(setupCalls + 1));
    expect(vi.mocked(bridge.intakeStatus).mock.calls.length).toBeGreaterThan(intakeCalls);
    expect(await within(card).findByRole('status', { name: 'Watcher health' })).toHaveTextContent(/watching inbox/i);
  });

  it('reconnects Microsoft with the device code, then checks the setup again', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const bridge = managedBridge();
    renderDialog(bridge);
    await act(async () => {});
    const card = screen.getByRole('region', { name: 'SharePoint connection' });
    const setupCalls = vi.mocked(bridge.getSharePointSetup).mock.calls.length;

    await act(async () => { fireEvent.click(within(card).getByRole('button', { name: 'Reconnect Microsoft' })); });
    expect(within(card).getByRole('status', { name: 'Microsoft sign-in' })).toHaveTextContent('WXYZ-1234');
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });

    expect(bridge.microsoftSignInPoll).toHaveBeenCalled();
    expect(within(card).queryByRole('status', { name: 'Microsoft sign-in' })).not.toBeInTheDocument();
    expect(vi.mocked(bridge.getSharePointSetup).mock.calls.length).toBeGreaterThan(setupCalls);
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/active/i);
    expect(within(card).queryByRole('alert')).not.toBeInTheDocument();
    expect(bridge.microsoftDisconnect).not.toHaveBeenCalled();
  });

  it('surfaces a wrong-tenant reconnect and disconnects that account instead of adopting it', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const getSharePointSetup = vi.fn()
      .mockResolvedValueOnce(active)
      .mockRejectedValue({ code: 'MICROSOFT_ACCOUNT_WRONG_TENANT', message: 'The connected Microsoft account is outside the provisioned organization tenant.' });
    const bridge = managedBridge({ getSharePointSetup });
    renderDialog(bridge);
    await act(async () => {});
    const card = screen.getByRole('region', { name: 'SharePoint connection' });

    await act(async () => { fireEvent.click(within(card).getByRole('button', { name: 'Reconnect Microsoft' })); });
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });

    expect(within(card).getByRole('alert')).toHaveTextContent(describeSharePointProblem('MICROSOFT_ACCOUNT_WRONG_TENANT').action);
    expect(bridge.microsoftDisconnect).toHaveBeenCalledOnce();
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/needs attention/i);
    expect(within(card).getByText('MICROSOFT_ACCOUNT_WRONG_TENANT')).not.toBeVisible();
  });

  it('cancels a pending reconnect and reports the connection as needing attention', async () => {
    const getSharePointSetup = vi.fn()
      .mockResolvedValueOnce(active)
      .mockRejectedValue({ code: 'MICROSOFT_ACCOUNT_MISSING', message: 'Connect the Microsoft account.' });
    const bridge = managedBridge({ getSharePointSetup, microsoftSignInPoll: vi.fn(async () => ({ state: 'pending' as const, intervalSeconds: 5 })) });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    await within(card).findByText('Pat Doe');

    fireEvent.click(within(card).getByRole('button', { name: 'Reconnect Microsoft' }));
    await within(card).findByRole('status', { name: 'Microsoft sign-in' });
    fireEvent.click(within(card).getByRole('button', { name: 'Cancel sign-in' }));

    await waitFor(() => expect(within(card).queryByRole('status', { name: 'Microsoft sign-in' })).not.toBeInTheDocument());
    expect(bridge.microsoftDisconnect).toHaveBeenCalledOnce();
    expect(await within(card).findByRole('alert')).toHaveTextContent(describeSharePointProblem('MICROSOFT_ACCOUNT_MISSING').action);
    expect(within(card).getByRole('button', { name: 'Reconnect Microsoft' })).toBeEnabled();
  });

  it('asks OneDrive to sync the library from Settings when it is not synced yet, then checks again', async () => {
    const pending: SharePointSetupStatus = { ...active, phase: 'enrollment_pending' };
    let finishSync!: () => void;
    const startSharePointSync = vi.fn(() => new Promise<SharePointSetupStatus>((done) => { finishSync = () => done(pending); }));
    const bridge = managedBridge({ getSharePointSetup: vi.fn(async () => pending), startSharePointSync });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    const sync = await within(card).findByRole('button', { name: 'Sync Files with OneDrive' });
    expect(within(card).queryByRole('button', { name: 'Turn on filing' })).not.toBeInTheDocument();
    const setupCalls = vi.mocked(bridge.getSharePointSetup).mock.calls.length;

    fireEvent.click(sync);
    expect(await within(card).findByRole('button', { name: 'Asking OneDrive…' })).toBeDisabled();
    fireEvent.click(within(card).getByRole('button', { name: 'Asking OneDrive…' }));
    await act(async () => { finishSync(); });

    expect(startSharePointSync).toHaveBeenCalledOnce();
    await waitFor(() => expect(vi.mocked(bridge.getSharePointSetup).mock.calls.length).toBeGreaterThan(setupCalls));
    expect(await within(card).findByRole('button', { name: 'Sync Files with OneDrive' })).toBeEnabled();
    expect(within(card).queryByRole('alert')).not.toBeInTheDocument();
  });

  it('explains a failed sync request and offers OneDrive instead of a sync request that would fail again', async () => {
    const pending: SharePointSetupStatus = { ...active, phase: 'enrollment_pending' };
    const openSupportLink = vi.fn(async () => {});
    const bridge = managedBridge({ openSupportLink, getSharePointSetup: vi.fn(async () => pending), startSharePointSync: vi.fn(async () => { throw { code: 'ONEDRIVE_MISSING', message: 'OneDrive is not installed.' }; }) });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(await within(card).findByRole('button', { name: 'Sync Files with OneDrive' }));

    expect(await within(card).findByRole('alert')).toHaveTextContent(describeSharePointProblem('ONEDRIVE_MISSING').action);
    expect(within(card).queryByRole('button', { name: 'Sync Files with OneDrive' })).not.toBeInTheDocument();
    expect(within(card).getByRole('button', { name: 'Check again' })).toBeEnabled();
    fireEvent.click(within(card).getByRole('button', { name: 'Get OneDrive' }));
    await waitFor(() => expect(openSupportLink).toHaveBeenCalledWith('onedrive-download'));

    // Checking again clears the failed request, and the sync action returns.
    fireEvent.click(within(card).getByRole('button', { name: 'Check again' }));
    expect(await within(card).findByRole('button', { name: 'Sync Files with OneDrive' })).toBeEnabled();
  });

  it('keeps the sync action when the request never reached OneDrive', async () => {
    const pending: SharePointSetupStatus = { ...active, phase: 'enrollment_pending' };
    const bridge = managedBridge({ getSharePointSetup: vi.fn(async () => pending), startSharePointSync: vi.fn(async () => { throw { code: 'ONEDRIVE_OPEN_FAILED', message: 'launch failed' }; }) });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(await within(card).findByRole('button', { name: 'Sync Files with OneDrive' }));

    expect(await within(card).findByRole('alert')).toHaveTextContent(describeSharePointProblem('ONEDRIVE_OPEN_FAILED').action);
    expect(within(card).getByRole('button', { name: 'Sync Files with OneDrive' })).toBeEnabled();
  });

  it('shows a OneDrive record problem that keeps the library pending, with its code for support', async () => {
    const problem = { code: 'SHAREPOINT_ROOT_RECORD_CONFLICT', message: "OneDrive's sync records disagree (library records)." };
    const pending: SharePointSetupStatus = { ...active, phase: 'enrollment_pending', problem };
    renderDialog(managedBridge({ getSharePointSetup: vi.fn(async () => pending) }));
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });

    expect(await within(card).findByRole('alert')).toHaveTextContent(describeSharePointProblem(problem).action);
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/needs attention/i);
    expect(within(card).getByRole('button', { name: 'Sync Files with OneDrive' })).toBeEnabled();
    expect(within(card).getByRole('button', { name: 'Open SharePoint' })).toBeVisible();

    const details = within(card).getByRole('group', { name: 'Support details' });
    fireEvent.click(within(details).getByText('Support details'));
    expect(within(details).getByText('SHAREPOINT_ROOT_RECORD_CONFLICT').closest('dd')).toHaveTextContent(problem.message);
  });

  it('turns on filing from Settings when setup is ready but not active, then checks again', async () => {
    let phase: SharePointSetupStatus['phase'] = 'ready_to_activate';
    const activateOnboarding = vi.fn(async () => { phase = 'active'; return { ...active, phase }; });
    const bridge = managedBridge({ getSharePointSetup: vi.fn(async () => ({ ...active, phase })), activateOnboarding });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    expect(within(card).queryByRole('button', { name: 'Sync Files with OneDrive' })).not.toBeInTheDocument();

    fireEvent.click(await within(card).findByRole('button', { name: 'Turn on filing' }));

    expect(await within(card).findByText(/Active\./)).toBeVisible();
    expect(activateOnboarding).toHaveBeenCalledOnce();
    expect(within(card).queryByRole('button', { name: 'Turn on filing' })).not.toBeInTheDocument();
    expect(within(card).queryByRole('alert')).not.toBeInTheDocument();
  });

  it('reports a failed activation and leaves the action to try again', async () => {
    const bridge = managedBridge({ getSharePointSetup: vi.fn(async () => ({ ...active, phase: 'ready_to_activate' as const })), activateOnboarding: vi.fn(async () => { throw { code: 'FILED_UNWRITABLE', message: 'Filed is read-only' }; }) });
    renderDialog(bridge);
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(await within(card).findByRole('button', { name: 'Turn on filing' }));

    expect(await within(card).findByRole('alert')).toHaveTextContent(describeSharePointProblem('FILED_UNWRITABLE').action);
    expect(within(card).getByRole('button', { name: 'Turn on filing' })).toBeEnabled();
    fireEvent.click(within(card).getByText('Support details'));
    expect(within(card).getByText('FILED_UNWRITABLE')).toBeVisible();
  });

  it('offers to turn filing on again after reconnecting as a different account', async () => {
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] });
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'active', account: { displayName: 'Pat Doe', email: 'pat@contoso.test' }, signInAccount: { displayName: 'Sam Roe', email: 'sam@contoso.test' } } });
    renderDialog(bridge);
    await act(async () => {});
    const card = screen.getByRole('region', { name: 'SharePoint connection' });
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/active/i);

    await act(async () => { fireEvent.click(within(card).getByRole('button', { name: 'Reconnect Microsoft' })); });
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });

    expect(within(card).getByText('Sam Roe')).toBeVisible();
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/needs attention/i);
    expect(within(card).getByRole('button', { name: 'Turn on filing' })).toBeEnabled();

    await act(async () => { fireEvent.click(within(card).getByRole('button', { name: 'Turn on filing' })); });
    expect(within(card).getByRole('status', { name: 'SharePoint setup' })).toHaveTextContent(/active/i);
  });

  it('saves unrelated settings with the managed paths and flags exactly as loaded', async () => {
    const { onSave } = renderDialog(managedBridge());
    await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.change(screen.getByLabelText('Arrange filed documents'), { target: { value: 'year' } });
    fireEvent.click(screen.getByLabelText('Automatically rename high-confidence files'));
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
    expect(onSave).toHaveBeenCalledWith({ ...managedSettings, destinationLayout: 'year', automaticRename: true });
  });

  it('explains a refused managed save with the same sentence onboarding uses', async () => {
    const onSave = vi.fn(async () => { throw { code: 'SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE', message: 'the managed binding could not be read' }; });
    renderDialog(managedBridge(), managedSettings, onSave);
    await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    const alert = await screen.findByText(describeSharePointProblem('SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE').action, { exact: false });
    expect(alert).toHaveTextContent('(SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE)');
    expect(alert).not.toHaveTextContent('the managed binding could not be read');
  });

  it('asks for a moment when a save lands while filing is being turned on', async () => {
    const onSave = vi.fn(async () => { throw { code: 'SHAREPOINT_ACTIVATION_IN_PROGRESS', message: 'activation holds the settings gate' }; });
    renderDialog(managedBridge(), managedSettings, onSave);
    await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    const alert = await screen.findByText(describeSharePointProblem('SHAREPOINT_ACTIVATION_IN_PROGRESS').action, { exact: false });
    expect(alert).toHaveTextContent('(SHAREPOINT_ACTIVATION_IN_PROGRESS)');
  });

  it('never sends managed values that differ from what was loaded', async () => {
    // A draft edited before the deployment status arrived must not leak through.
    let resolveOnboarding!: (value: OnboardingStatus) => void;
    const bridge = managedBridge({ getOnboarding: vi.fn(() => new Promise<OnboardingStatus>((done) => { resolveOnboarding = done; })) });
    const { onSave } = renderDialog(bridge);
    fireEvent.change(screen.getByLabelText('Destination folder'), { target: { value: 'C:\\Elsewhere' } });
    await waitFor(() => expect(bridge.getOnboarding).toHaveBeenCalled());
    await act(async () => { resolveOnboarding({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: true }); });
    await screen.findByRole('region', { name: 'SharePoint connection' });

    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
    expect(onSave).toHaveBeenCalledWith(managedSettings);
  });

  it('shows no shared-intake or pairing controls until it knows whether this build is managed', async () => {
    let resolveOnboarding!: (value: OnboardingStatus) => void;
    const bridge = managedBridge({ getOnboarding: vi.fn(() => new Promise<OnboardingStatus>((done) => { resolveOnboarding = done; })) });
    renderDialog(bridge);
    await waitFor(() => expect(bridge.getOnboarding).toHaveBeenCalled());

    expect(screen.getByRole('status', { name: 'Loading intake settings' })).toBeVisible();
    expect(screen.queryByLabelText('Intake folder')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Watch a folder for new documents')).not.toBeInTheDocument();
    expect(screen.queryByLabelText("This machine's name")).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Run in background')).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Connect my Microsoft account' })).not.toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'SharePoint connection' })).not.toBeInTheDocument();

    await act(async () => { resolveOnboarding({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: true }); });
    expect(await screen.findByRole('region', { name: 'SharePoint connection' })).toBeVisible();
    expect(screen.queryByRole('status', { name: 'Loading intake settings' })).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Intake folder')).not.toBeInTheDocument();
  });

  // A failed read is not an answer: the build may well carry the
  // deployment, so the Microsoft panel stays and asks the backend itself.
  it('keeps the manual controls when the onboarding status cannot be read', async () => {
    const bridge = managedBridge({ getOnboarding: vi.fn(async () => { throw { code: 'ONBOARDING_STATE_UNREADABLE', message: 'corrupt' }; }) });
    renderDialog(bridge);

    expect(await screen.findByLabelText('Intake folder')).toBeVisible();
    expect(screen.getByRole('group', { name: 'Microsoft upload identity' })).toBeVisible();
    expect(screen.queryByRole('status', { name: 'Loading intake settings' })).not.toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'SharePoint connection' })).not.toBeInTheDocument();
  });

  it('keeps the manual settings, without Microsoft verification, when this build has no SharePoint deployment', async () => {
    const base = createInMemoryBridge();
    const getSharePointSetup = vi.fn(base.getSharePointSetup);
    const getOnboarding = vi.fn(base.getOnboarding);
    const { onSave } = renderDialog({ ...base, getOnboarding, getSharePointSetup }, { ...managedSettings, runInBackground: false });
    await waitFor(() => expect(getOnboarding).toHaveBeenCalled());
    await act(async () => {});

    expect(screen.queryByRole('region', { name: 'SharePoint connection' })).not.toBeInTheDocument();
    expect(screen.getByLabelText('Destination folder')).toBeVisible();
    expect(screen.getByLabelText('Intake folder')).toBeVisible();
    expect(screen.getByLabelText("This machine's name")).toBeVisible();
    expect(screen.getByLabelText('Also process documents uploaded by others')).toBeVisible();
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(getSharePointSetup).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText("This machine's name"), { target: { value: 'Reception' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith(expect.objectContaining({ machineLabel: 'Reception' })));
  });

  // The deployment-unavailable error is what every shipping build reports.
  // Whether the build said so (no deployment) or the read failed and the
  // panel asked for itself, Settings must not show a broken Microsoft panel
  // that calls the installed app a browser preview.
  it('hides the Microsoft panel when the deployment is unavailable', async () => {
    const unavailable = 'SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build.';
    const microsoftIntakeStatus = vi.fn(async (): Promise<MicrosoftIntakeStatus> => ({ connected: false, account: null, binding: null, documents: [], error: unavailable }));
    const settings = { ...managedSettings, intakeLocalOnly: false };
    const unmanaged = { currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: false };

    const view = render(<SettingsDialog settings={settings} bridge={managedBridge({ getOnboarding: vi.fn(async () => unmanaged), microsoftIntakeStatus })} onSave={vi.fn(async () => {})} onClose={() => {}} onCheckForUpdate={async () => ({ state: 'unsupported' })} onInstallUpdate={async () => {}} />);
    expect(await screen.findByLabelText('Intake folder')).toBeVisible();
    await act(async () => {});
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(screen.queryByText(/browser preview/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/deployment configuration/i)).not.toBeInTheDocument();
    view.unmount();

    renderDialog(managedBridge({ getOnboarding: vi.fn(async () => { throw new Error('unreadable'); }), microsoftIntakeStatus }), settings);
    expect(await screen.findByRole('status', { name: 'Microsoft connection status' })).toHaveTextContent('Microsoft upload verification is not part of this build.');
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(screen.queryByText(/browser preview/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/deployment configuration/i)).not.toBeInTheDocument();
  });

  // Focus moves when `managed` lands. It used to move in a passive effect,
  // which React flushes in a later scheduler task, while findByRole resolves
  // on the DOM mutation of the commit itself: under a loaded run the
  // assertions could look before the effect had run. The product now moves
  // focus in a layout effect, inside that commit; the test also waits rather
  // than looking once.
  it('moves focus into the dialog when the manual fields it started on are replaced', async () => {
    renderDialog(managedBridge());
    await screen.findByRole('region', { name: 'SharePoint connection' }, { timeout: 5_000 });
    await waitFor(() => {
      const active = document.activeElement as HTMLElement;
      expect(screen.getByRole('dialog', { name: 'Settings' })).toContainElement(active);
      expect(screen.getByLabelText('Arrange filed documents')).toHaveFocus();
    });
  });

  // The layout effect is the product half of the fix: by the time the commit
  // that inserts the region has finished, focus has already moved.
  it('has moved focus by the time the managed region is in the document', async () => {
    renderDialog(managedBridge());
    const seen: Array<Element | null> = [];
    const observer = new MutationObserver(() => {
      if (!seen.length && document.querySelector('section.sharepoint-connection')) seen.push(document.activeElement);
    });
    observer.observe(document.body, { childList: true, subtree: true });
    try {
      await screen.findByRole('region', { name: 'SharePoint connection' }, { timeout: 5_000 });
    } finally {
      observer.disconnect();
    }
    expect(seen[0]).toBe(screen.getByLabelText('Arrange filed documents'));
  });

  it('lets the keyboard reach the support details disclosure', async () => {
    renderDialog(managedBridge());
    const card = await screen.findByRole('region', { name: 'SharePoint connection' });
    const summary = within(card).getByText('Support details');
    summary.focus();
    expect(document.activeElement).toBe(summary);
    fireEvent.keyDown(document, { key: 'Tab' });
    expect(document.activeElement).not.toBe(summary);
    expect(screen.getByRole('dialog', { name: 'Settings' })).toContainElement(document.activeElement as HTMLElement);
  });
});
