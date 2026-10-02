import { GUIDE_URL, SUPPORT_LINKS } from './bridge';
import type { DesktopBridge, FileSelection, FolderSelection, SelectionBoundary, SelectionResult, UpdateStatus } from './bridge';
import type { AppSettings, CloudLocation, CloudRoot, DescriptionsStatus, HistoryEntry, HostedModelStatus, HostedModelTestResult, IntakeStatus, LearnedRule, OnboardingStatus, QueueItem, SetupState, SharePointSetupPhase, SharePointSetupProblem, SharePointSetupStatus } from '../types';
import { leadingDate } from './filenames';
import { filedBeside, rootFor } from '../features/intake/folderNames';
import type { MicrosoftIntakeBridge } from '../features/intake/microsoft';

/** Exact size of the single pinned model file this build downloads. */
export const PINNED_MODEL_BYTES = 1_280_835_840;

const seedItems: QueueItem[] = [
  { id: 'employment', originalFilename: 'Employment Agreement - John Smith.pdf', status: 'ready', proposedFilename: '2024-04-12 Employment Agreement with John Smith.pdf', confidence: 0.98 },
  { id: 'lease', originalFilename: 'Lease Agreement - 123 Main St.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf', confidence: 0.72, description: 'Commercial lease agreement between landlord and tenant for 123 Main St.', evidence: { date: 'Sep 15, 2023', type: 'Lease Agreement', parties: 'ABC Properties LLC; TenantCo Inc.' }, reason: 'Lower confidence due to unclear document type keywords and multiple possible dates.' },
  { id: 'nda', originalFilename: 'NDA - Acme Corp.docx', status: 'ready', proposedFilename: '2024-03-01 Non-Disclosure Agreement with Acme Corp.docx', confidence: 0.95 },
  // Stays a PDF even though .xlsx is now supported: the reviewed QA capture
  // pins this queue's rendered contents, and changing a demo row would force a
  // re-sign-off for no product reason.
  { id: 'financials', originalFilename: 'Q1 Financials.pdf', status: 'processing', proposedFilename: '2024-03-31 Q1 Financial Statements.pdf', progress: 60 },
  { id: 'service', originalFilename: 'Service Agreement - BlueSky LLC.pdf', status: 'ready', proposedFilename: '2024-02-28 Service Agreement with BlueSky LLC.pdf', confidence: 0.96 },
  { id: 'minutes', originalFilename: 'Board Meeting Minutes - May 7, 2024.docx', status: 'waiting' },
  { id: 'invoice', originalFilename: 'Invoice INV-1001.pdf', status: 'waiting' },
  { id: 'notes', originalFilename: 'Notes from Call - 2024-05-02.txt', status: 'waiting' },
  { id: 'completed', originalFilename: 'Completed lease.pdf', status: 'completed', proposedFilename: '2024-01-22 Lease Agreement.pdf', confidence: 0.93, undoable: true, description: 'Residential lease agreement for a twelve-month term beginning January 22, 2024.' },
];

/**
 * Plausible finished operations for browser dev and tests, newest first —
 * the order the desktop backend returns. Timestamps are fixed so renders are
 * deterministic.
 */
const seedHistory: HistoryEntry[] = [
  { receiptId: '9', queueItemId: 'completed', at: 1716282900, direction: 'undo', kind: 'rename', stage: 'complete', originalPath: 'C:\\Filed\\2024-05-07 Board Meeting Minutes.docx', newPath: 'C:\\Drop\\Board Meeting Minutes - May 7, 2024.docx' },
  { receiptId: '7', queueItemId: 'completed', at: 1716282600, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\Board Meeting Minutes - May 7, 2024.docx', newPath: 'C:\\Filed\\2024-05-07 Board Meeting Minutes.docx' },
  { receiptId: '5', queueItemId: 'completed', at: 1716196500, direction: 'apply', kind: 'verified_copy', stage: 'complete', originalPath: 'C:\\Drop\\Invoice INV-1001.pdf', newPath: 'D:\\Archive\\2024-05-02 Invoice INV-1001 from BlueSky LLC.pdf', },
  { receiptId: '3', queueItemId: 'completed', at: 1716108300, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\Completed lease.pdf', newPath: 'C:\\Filed\\2024-01-22 Lease Agreement.pdf' },
  { receiptId: '1', queueItemId: 'completed', at: 1716021900, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\NDA - Acme Corp.docx', newPath: 'C:\\Filed\\2024-03-01 Non-Disclosure Agreement with Acme Corp.docx' },
];

export interface InMemoryBridgeOptions {
  items?: QueueItem[];
  /** Settings already saved, such as an earlier release's manual intake folders. */
  settings?: Partial<AppSettings>;
  setup?: Partial<SetupState>;
  /** A hosted-model key already in the (fake) credential store. */
  hostedKey?: string;
  /** Spellings already learned from review. */
  learnedRules?: LearnedRule[];
  downloadStepBytes?: number;
  downloadIntervalMs?: number;
  update?: UpdateStatus;
  /** Backend-owned progress seeded for browser development and tests. */
  completedOnboardingVersion?: number;
  /**
   * The packaged SharePoint deployment. `unavailable` (the default) mirrors
   * the shipping build, whose deployment resource is disabled; `fake`
   * simulates an enabled deployment so guided onboarding can be exercised.
   */
  sharePoint?: 'unavailable' | 'fake';
  sharePointFake?: FakeSharePointOptions;
  /** Documents folder setup finds already in a chosen folder. */
  existingDocuments?: number;
}

export type FakeSharePointCall = 'microsoftSignInStart' | 'microsoftSignInPoll' | 'microsoftOpenSignIn' | 'microsoftDisconnect' | 'getSharePointSetup' | 'startSharePointSync' | 'activateOnboarding' | 'completeOnboarding';

export interface FakeSharePointOptions {
  /** A Microsoft account is already connected, as after a relaunch mid-setup. */
  connected?: boolean;
  account?: { displayName: string; email: string };
  /**
   * The account device sign-in connects as, when it is not `account`. Signing
   * in as a different account than the one that turned filing on takes an
   * active library back to ready_to_activate, as the backend does.
   */
  signInAccount?: { displayName: string; email: string };
  /** Polls that report pending before device sign-in connects (at least 1). */
  signInPolls?: number;
  /** The library's starting phase; enrollment_pending until OneDrive is asked to sync. */
  phase?: SharePointSetupPhase;
  /** Rescans that still report pending after a sync request before the library appears. */
  pendingRescans?: number;
  /** A OneDrive record problem every pending status carries, as the backend reports one. */
  pendingProblem?: SharePointSetupProblem;
  /** Errors each call throws, one per call in order, before it behaves normally again. */
  failures?: Partial<Record<FakeSharePointCall, unknown[]>>;
}

function itemFromFile(file: FileSelection, fixtureBatch = false): QueueItem {
  if (fixtureBatch) {
    if (file.displayName === 'duplicate-invoice-a.pdf') return {
      id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'review',
      proposedFilename: '2025-04-30 Invoice from Nimbus Orchard Supply Co.pdf', confidence: 0.82,
      description: 'Invoice INV-2048 dated April 30, 2025 for Atlas Threadworks LLC.',
      evidence: { date: 'Invoice date: April 30, 2025', type: 'INVOICE INV-2048', parties: 'Nimbus Orchard Supply Co.; Atlas Threadworks LLC' },
      reason: 'Needs review because the invoice and due dates are both present.',
    };
    if (file.displayName === 'duplicate-invoice-b.pdf') return {
      id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'review',
      proposedFilename: '2025-04-30 Invoice from Nimbus Orchard Supply Co.pdf', confidence: 0.82,
      description: 'Invoice INV-2048 dated April 30, 2025 for Atlas Threadworks LLC.',
      evidence: { date: 'Invoice date: April 30, 2025', type: 'INVOICE INV-2048', parties: 'Nimbus Orchard Supply Co.; Atlas Threadworks LLC' },
      reason: 'Identical content from a different path is retained as a separate review result.',
    };
    if (file.displayName === 'unsupported.csv') return { id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'failed', reason: 'Unsupported format skipped: .csv.' };
    if (file.displayName.startsWith('~$')) return { id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'failed', reason: 'Office lock file skipped.' };
  }
  return { id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'waiting' };
}

function createBridge(options: InMemoryBridgeOptions, fixtureBatch: boolean): DesktopBridge {
  let items = (options.items ?? seedItems).map((item) => ({ ...item }));
  let history = seedHistory.map((entry) => ({ ...entry }));
  let settings: AppSettings = { destination: '', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: '', intakeEnabled: false, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '', ...options.settings };
  // The hosted model's key, as the desktop backend keeps it: out of the
  // settings, reported only as stored-or-not with a hint.
  let hostedKey: string | undefined = options.hostedKey;
  // What review has taught, as the backend keeps it. Learning itself lives
  // in the queue, so approving here teaches nothing; the list is fixed.
  let learnedRules: LearnedRule[] = (options.learnedRules ?? []).map((rule) => ({ ...rule }));
  const providerDefaults = [
    { provider: 'anthropic' as const, baseUrl: 'https://api.anthropic.com/v1', model: 'claude-opus-5' },
    { provider: 'openai_compatible' as const, baseUrl: 'https://api.openai.com/v1', model: '' },
  ];
  const hostedEndpoint = (draft: AppSettings): string | null => {
    const defaults = providerDefaults.find((entry) => entry.provider === draft.hostedProvider)!;
    const base = (draft.hostedBaseUrl.trim() || defaults.baseUrl).replace(/\/+$/, '');
    const model = draft.hostedModel.trim() || defaults.model;
    if (!model || !/^https:\/\//.test(base) && !/^http:\/\/(localhost|127\.0\.0\.1)/.test(base)) return null;
    return `${base}/${draft.hostedProvider === 'anthropic' ? 'messages' : 'chat/completions'}`;
  };
  const hostedConfigured = (draft: AppSettings) => hostedKey !== undefined && hostedEndpoint(draft) !== null;
  const hostedModelStatus = (): HostedModelStatus => ({
    keyStored: hostedKey !== undefined,
    keyHint: hostedKey === undefined ? null : `…${hostedKey.slice(-4)}`,
    endpoint: hostedConfigured(settings) ? hostedEndpoint(settings) : null,
    providers: providerDefaults.map((entry) => ({ ...entry })),
  });
  // Deterministic classification so browser dev and e2e runs can exercise the
  // cloud badge without a real sync client: the path only has to mention the
  // provider, or start like a UNC path. Mirrors the DTO the desktop backend
  // returns from folder_classify.
  const classifyPath = async (path: string): Promise<CloudLocation | null> => {
    const root = rootFor(roots, path);
    if (root) return { provider: root.provider, displayName: root.displayName };
    const lower = path.toLowerCase();
    if (lower.includes('onedrive')) return { provider: 'onedrive_business', displayName: 'OneDrive – Contoso' };
    if (lower.includes('sharepoint')) return { provider: 'sharepoint', displayName: 'Contoso' };
    const share = /^\\\\([^\\]+)\\([^\\]+)/.exec(path);
    if (share) return { provider: 'network_share', displayName: `\\\\${share[1]}\\${share[2]}` };
    return null;
  };
  // The sync roots a developer machine would show; fixed so the Settings
  // list renders the same in dev and tests.
  const roots: CloudRoot[] = [
    { provider: 'sharepoint', displayName: 'Contoso', path: 'C:\\Users\\pat\\Contoso\\Legal - Documents' },
    { provider: 'onedrive_business', displayName: 'OneDrive – Contoso', path: 'C:\\Users\\pat\\OneDrive - Contoso' },
  ];
  let recordedDescriptions = 0;
  let lastRecordedAt: number | null = null;
  const descriptionsStatus = (): DescriptionsStatus => ({
    enabled: settings.recordDescriptions,
    folder: settings.destination.trim() ? `${settings.destination.replace(/[\\/]+$/, '')}\\.intern\\descriptions` : '',
    recordedThisSession: recordedDescriptions,
    lastRecordedAt,
    lastError: null,
  });
  const noteRecorded = () => {
    if (!settings.recordDescriptions || !settings.destination.trim()) return;
    recordedDescriptions += 1;
    lastRecordedAt = Math.floor(Date.now() / 1000);
  };
  // The pinned model's exact size from src-tauri/resources/model-manifest.json.
  // The previous value, 3_278_329_184, was a model plus a vision projector that
  // this pipeline does not download.
  let setup: SetupState = { state: 'ready', downloadedBytes: PINNED_MODEL_BYTES, totalBytes: PINNED_MODEL_BYTES, ...options.setup };
  // Without a deployment the demo opens straight into the app; pass 0 to walk
  // folder setup instead.
  let completedOnboardingVersion = options.completedOnboardingVersion ?? (options.sharePoint === 'fake' ? 0 : 1);
  const microsoftUnavailable = 'SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build.';
  const sharePointEnabled = options.sharePoint === 'fake';
  const fake = options.sharePointFake ?? {};
  const failures = new Map(Object.entries(fake.failures ?? {}).map(([call, errors]) => [call, [...(errors ?? [])]]));
  const failNext = (call: FakeSharePointCall) => { const queued = failures.get(call); if (queued?.length) throw queued.shift(); };
  const firstAccount = { tenantId: 'fake-contoso-tenant', id: 'fake-account', ...(fake.account ?? { displayName: 'Pat Lee', email: 'pat.lee@contoso.example' }) };
  let fakeAccount = firstAccount;
  let microsoftConnected = fake.connected ?? false;
  let signInPollsLeft: number | undefined;
  let libraryPhase: SharePointSetupPhase = fake.phase ?? 'enrollment_pending';
  // Activation belongs to the account that turned filing on.
  let activatedFor: string | undefined = libraryPhase === 'active' ? fakeAccount.id : undefined;
  let rescansLeft: number | undefined;
  const libraryRoot = 'C:\\Users\\pat\\Contoso\\InternTestSite - Files';
  const sharePointStatus = (): SharePointSetupStatus => {
    if (!microsoftConnected) throw { code: 'MICROSOFT_ACCOUNT_MISSING', message: 'Connect the Microsoft account before setting up the Files library.' };
    const problem = libraryPhase === 'enrollment_pending' ? fake.pendingProblem ?? null : null;
    return { phase: libraryPhase, account: { displayName: fakeAccount.displayName, email: fakeAccount.email }, site: 'InternTestSite', library: 'Files', intake: 'Inbox', destination: 'Filed', problem };
  };
  // A simulated enabled deployment: device sign-in, OneDrive enrollment that
  // takes a few rescans to appear, and activation to the managed settings.
  const fakeSharePoint: Pick<DesktopBridge, 'getSharePointSetup' | 'startSharePointSync' | 'activateOnboarding' | 'completeOnboarding'> & MicrosoftIntakeBridge = {
    getSharePointSetup: async () => {
      failNext('getSharePointSetup');
      const status = sharePointStatus();
      if (libraryPhase !== 'enrollment_pending' || rescansLeft === undefined) return status;
      if (rescansLeft > 0) { rescansLeft -= 1; return status; }
      libraryPhase = 'ready_to_activate';
      rescansLeft = undefined;
      return sharePointStatus();
    },
    startSharePointSync: async () => {
      failNext('startSharePointSync');
      const status = sharePointStatus();
      if (libraryPhase === 'enrollment_pending') rescansLeft = fake.pendingRescans ?? 2;
      return status;
    },
    activateOnboarding: async () => {
      failNext('activateOnboarding');
      sharePointStatus();
      if (libraryPhase === 'enrollment_pending') throw { code: 'SHAREPOINT_SYNC_PENDING', message: 'OneDrive has not registered the verified Files library yet. Keep OneDrive open and try again.' };
      libraryPhase = 'active';
      activatedFor = fakeAccount.id;
      // The managed values activation derives, as the backend enforces them.
      settings = { ...settings, intakeFolder: `${libraryRoot}\\Inbox`, destination: `${libraryRoot}\\Filed`, intakeEnabled: true, processOthersUploads: false, intakeLocalOnly: false, runInBackground: true, startAtLogin: true, startMinimized: true };
      return sharePointStatus();
    },
    completeOnboarding: async () => {
      failNext('completeOnboarding');
      // The backend records completion only for a library that is actually filing.
      if (!microsoftConnected || libraryPhase !== 'active') throw { code: 'ONBOARDING_SETUP_INCOMPLETE', message: 'SharePoint setup is not active, so onboarding cannot be completed.' };
      completedOnboardingVersion = Math.max(completedOnboardingVersion, 1);
    },
    microsoftIntakeStatus: async () => ({
      connected: microsoftConnected,
      account: microsoftConnected ? { ...fakeAccount } : null,
      binding: microsoftConnected && libraryPhase === 'active' ? { localFolder: `${libraryRoot}\\Inbox`, driveId: 'fake-drive', folderId: 'fake-inbox', webUrl: 'https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox', tenantId: fakeAccount.tenantId } : null,
      documents: [],
      error: null,
    }),
    microsoftSignInStart: async () => {
      failNext('microsoftSignInStart');
      signInPollsLeft = Math.max(1, fake.signInPolls ?? 1);
      return { userCode: 'ABCD-EFGH', verificationUri: 'https://microsoft.com/devicelogin', intervalSeconds: 5, expiresAt: Math.floor(Date.now() / 1000) + 900 };
    },
    microsoftSignInPoll: async () => {
      failNext('microsoftSignInPoll');
      if (signInPollsLeft === undefined) throw new Error('Microsoft sign-in changed or was canceled. Files remain held.');
      signInPollsLeft -= 1;
      if (signInPollsLeft > 0) return { state: 'pending', intervalSeconds: 5 };
      signInPollsLeft = undefined;
      microsoftConnected = true;
      if (fake.signInAccount) fakeAccount = { tenantId: firstAccount.tenantId, id: 'fake-account-signed-in', ...fake.signInAccount };
      if (libraryPhase === 'active' && activatedFor !== fakeAccount.id) libraryPhase = 'ready_to_activate';
      return { state: 'connected', account: { ...fakeAccount } };
    },
    microsoftDisconnect: async () => { failNext('microsoftDisconnect'); microsoftConnected = false; signInPollsLeft = undefined; },
    microsoftBindIntake: async () => { throw new Error('The fixed deployment binds the Files library during activation.'); },
    microsoftOpenSignIn: async () => { failNext('microsoftOpenSignIn'); },
  };
  const downloadStepBytes = options.downloadStepBytes ?? Math.max(1, Math.ceil(setup.totalBytes / 4));
  const downloadIntervalMs = options.downloadIntervalMs ?? 40;
  let downloadTimer: ReturnType<typeof setInterval> | undefined;
  const idByPath = new Map<string, string>();
  const pathById = new Map<string, string>();
  const update = (id: string, change: Partial<QueueItem>) => { items = items.map((item) => item.id === id ? { ...item, ...change } : item); };
  const finishDownload = () => { if (downloadTimer) clearInterval(downloadTimer); downloadTimer = undefined; setup = { ...setup, state: 'ready', downloadedBytes: setup.totalBytes }; };
  const addFolder = (folder: FolderSelection) => {
    const sourceFiles = folder.files ?? [];
    const folderItems = sourceFiles.flatMap((file) => {
      if (idByPath.has(file.path)) return [];
      const item = itemFromFile(file, fixtureBatch);
      idByPath.set(file.path, item.id);
      pathById.set(item.id, file.path);
      return [item];
    });
    if (sourceFiles.length) { items = [...items, ...folderItems]; return; }
    if (idByPath.has(folder.path)) return;
    const item = { id: `folder-${crypto.randomUUID()}`, originalFilename: `${folder.displayName}/`, status: 'waiting' as const };
    idByPath.set(folder.path, item.id);
    pathById.set(item.id, folder.path);
    items = [...items, item];
  };
  return {
    listItems: async () => items.map((item) => ({ ...item })),
    addFiles: async (files) => {
      for (const file of files) {
        if (idByPath.has(file.path)) continue;
        const item = itemFromFile(file, fixtureBatch);
        idByPath.set(file.path, item.id);
        pathById.set(item.id, file.path);
        items = [...items, item];
      }
    },
    addFolder: async (folder) => addFolder(folder),
    pauseQueue: async () => { items = items.map((item) => item.status === 'processing' ? { ...item, status: 'waiting' as const } : item); },
    resumeQueue: async () => { const item = items.find((entry) => entry.status === 'waiting'); if (item) update(item.id, { status: 'processing', progress: 0 }); },
    cancel: async (id) => update(id, { status: 'failed', progress: undefined, reason: 'Canceled.' }),
    // Mirrors the backend's gate: a rename carries a date or it does not happen.
    approve: async (id, filename, description) => {
      if (!leadingDate(filename)) throw { code: 'DATE_REQUIRED', message: 'the filename must start with the document\'s date as YYYY-MM-DD' };
      update(id, { status: 'completed', proposedFilename: filename, description, undoable: true });
      noteRecorded();
    },
    keepOriginal: async (id) => update(id, { status: 'completed', proposedFilename: undefined, undoable: true }),
    retry: async (id) => update(id, { status: 'waiting', progress: undefined }),
    remove: async (id) => {
      const path = pathById.get(id);
      if (path) idByPath.delete(path);
      pathById.delete(id);
      items = items.filter((item) => item.id !== id);
    },
    undo: async (id) => update(id, { status: 'review', undoable: false }),
    getSettings: async () => ({ ...settings }),
    // Mirrors the backend: a hosted model is refused at save time without a
    // stored key or a usable address, never silently kept.
    saveSettings: async (next) => {
      if (next.modelSource === 'hosted') {
        if (hostedKey === undefined) throw { code: 'HOSTED_MODEL_KEY_MISSING', message: 'no API key is stored for the hosted model' };
        if (hostedEndpoint(next) === null) throw { code: 'HOSTED_MODEL_MISCONFIGURED', message: 'the hosted model\'s address or model name is not usable' };
      }
      settings = { ...next };
    },
    getSetup: async () => ({ ...setup, hostedModelReady: settings.modelSource === 'hosted' && hostedConfigured(settings) }),
    getOnboarding: async (): Promise<OnboardingStatus> => ({
      currentVersion: 1,
      completedVersion: completedOnboardingVersion,
      // Mirrors the backend: onboarding is only required for an enabled deployment.
      required: sharePointEnabled && completedOnboardingVersion < 1,
      sharePointAvailable: sharePointEnabled,
    }),
    getSharePointSetup: async () => { throw { code: 'SHAREPOINT_DEPLOYMENT_UNAVAILABLE', message: microsoftUnavailable }; },
    startSharePointSync: async () => { throw { code: 'SHAREPOINT_DEPLOYMENT_UNAVAILABLE', message: microsoftUnavailable }; },
    activateOnboarding: async () => { throw { code: 'SHAREPOINT_DEPLOYMENT_UNAVAILABLE', message: microsoftUnavailable }; },
    completeOnboarding: async () => { completedOnboardingVersion = Math.max(completedOnboardingVersion, 1); },
    microsoftIntakeStatus: async () => ({
      connected: false,
      account: null,
      binding: null,
      documents: [],
      error: microsoftUnavailable,
    }),
    microsoftSignInStart: async () => { throw new Error(microsoftUnavailable); },
    microsoftSignInPoll: async () => { throw new Error(microsoftUnavailable); },
    microsoftDisconnect: async () => { /* No credential exists in the disabled browser boundary. */ },
    microsoftBindIntake: async () => { throw new Error(microsoftUnavailable); },
    microsoftOpenSignIn: async () => { throw new Error(microsoftUnavailable); },
    ...(sharePointEnabled ? fakeSharePoint : {}),
    startModelDownload: async () => {
      if (setup.state === 'downloading') return;
      setup = { ...setup, state: 'downloading', error: undefined };
      downloadTimer = setInterval(() => {
        const downloadedBytes = Math.min(setup.totalBytes, setup.downloadedBytes + downloadStepBytes);
        setup = { ...setup, downloadedBytes };
        if (downloadedBytes >= setup.totalBytes) finishDownload();
      }, downloadIntervalMs);
    },
    setupCancel: async () => {
      if (downloadTimer) clearInterval(downloadTimer);
      downloadTimer = undefined;
      setup = { ...setup, state: 'required', error: 'MODEL_DOWNLOAD_CANCELED' };
    },
    setupChooseExisting: async () => {
      if (setup.state === 'downloading') throw { code: 'SETUP_BUSY', message: 'a model setup operation is already active' };
      setup = { ...setup, state: 'ready', downloadedBytes: setup.totalBytes, error: undefined };
    },
    // Mirrors the backend: clearing history deletes terminal queue rows and,
    // through cascade, the receipts the history view lists.
    clearHistory: async () => { items = items.filter((item) => item.status !== 'completed' && item.status !== 'failed'); history = []; },
    historyList: async () => history.map((entry) => ({ ...entry })),
    // No file is written in the browser; resolve with the same count the
    // desktop export reports.
    historyExport: async () => history.length,
    discardWaiting: async () => {
      const waiting = items.filter((item) => item.status === 'waiting');
      items = items.filter((item) => item.status !== 'waiting');
      return waiting.length;
    },
    // The browser build has no Tauri runtime and nothing to replace, so it says
    // so rather than pretending to be up to date.
    checkForUpdate: async () => options.update ?? { state: 'unsupported' },
    installUpdate: async () => { throw new Error('Updates are only available in the desktop application'); },
    // A coherent fake mirroring the current in-memory settings: enabled follows
    // intakeEnabled, and the counts are small fixed numbers so the dialog's
    // status block renders deterministically in dev and tests.
    intakeStatus: async (): Promise<IntakeStatus> => {
      const enabled = settings.intakeEnabled;
      const now = Math.floor(Date.now() / 1000);
      const machineName = settings.machineLabel.trim() || 'This machine';
      return {
        enabled,
        watching: enabled,
        folder: settings.intakeFolder,
        machineId: 'dev-machine',
        machineName,
        cloud: await classifyPath(settings.intakeFolder),
        machines: enabled ? [
          { machineId: 'dev-machine', machineName, userName: 'dev', lastSeenAt: now, active: true },
          { machineId: 'demo-peer', machineName: 'Front desk PC', userName: 'colleague', lastSeenAt: now - 90, active: true },
        ] : [],
        heldForOthers: enabled ? 2 : 0,
        syncConflicts: 0,
        awaitingHydration: 0,
        unreadableFolders: 0,
        claimedByOthers: enabled ? 1 : 0,
        processedHere: enabled ? 3 : 0,
        lastScanAt: enabled ? now - 5 : null,
        error: null,
        oneDriveRunning: enabled && (await classifyPath(settings.intakeFolder)) ? true : null,
      };
    },
    scanIntakeNow: async () => { /* Nothing is watching in the browser; the desktop backend wakes its scan loop. */ },
    classifyFolder: (path) => classifyPath(path),
    cloudRoots: async () => roots.map((root) => ({ ...root })),
    intakeFolderDocuments: async () => options.existingDocuments ?? 0,
    createFiledFolder: async (intakeFolder) => filedBeside(intakeFolder),
    openOneDrive: async () => { /* No OneDrive in the browser. */ },
    descriptionsStatus: async () => descriptionsStatus(),
    // Mirrors the backend: refused until the setting is saved on, otherwise
    // one record per completed item that still carries its sentence.
    descriptionsBackfill: async () => {
      if (!settings.recordDescriptions) throw { code: 'DESCRIPTIONS_DISABLED', message: 'turn on description records and save before writing them' };
      const written = items.filter((item) => item.status === 'completed' && item.description).length;
      recordedDescriptions += written;
      if (written) lastRecordedAt = Math.floor(Date.now() / 1000);
      return { written, failed: 0 };
    },
    // Already in a browser, so the guide opens the way any other link would.
    // noopener keeps the new tab from reaching back into this document.
    openGuide: async () => { window.open(GUIDE_URL, '_blank', 'noopener,noreferrer'); },
    openSupportLink: async (target) => { window.open(SUPPORT_LINKS[target], '_blank', 'noopener,noreferrer'); },
    hostedModelStatus: async () => hostedModelStatus(),
    hostedModelSetKey: async (key) => {
      if (!key.trim()) throw { code: 'HOSTED_MODEL_KEY_EMPTY', message: 'the API key is empty' };
      hostedKey = key.trim();
    },
    hostedModelClearKey: async () => { hostedKey = undefined; },
    // No request leaves the browser: the fake answers the way the desktop
    // backend does once the key, address, and model resolve.
    hostedModelTest: async (draft): Promise<HostedModelTestResult> => {
      if (hostedKey === undefined) throw { code: 'HOSTED_MODEL_KEY_MISSING', message: 'no API key is stored for the hosted model' };
      const endpoint = hostedEndpoint(draft);
      if (endpoint === null) throw { code: 'HOSTED_MODEL_MISCONFIGURED', message: 'the hosted model\'s address or model name is not usable' };
      if (hostedKey.startsWith('bad-')) throw { code: 'HOSTED_MODEL_UNAUTHORIZED', message: 'the hosted service rejected the API key' };
      const defaults = providerDefaults.find((entry) => entry.provider === draft.hostedProvider)!;
      return { model: draft.hostedModel.trim() || defaults.model, endpoint, filename: '2024-01-02 Notice of Calibration - Northstar Calibration Holdings LLC.pdf', inferenceMillis: 1840 };
    },
    houseRulesList: async () => learnedRules.map((rule) => ({ ...rule })),
    houseRuleForget: async (id) => {
      if (!learnedRules.some((rule) => rule.id === id)) throw { code: 'RULE_NOT_FOUND', message: 'learned spelling does not exist' };
      learnedRules = learnedRules.filter((rule) => rule.id !== id);
    },
    houseRuleUse: async (id) => {
      if (!learnedRules.some((rule) => rule.id === id)) throw { code: 'RULE_NOT_FOUND', message: 'learned spelling does not exist' };
      learnedRules = learnedRules.map((rule) => rule.id === id ? { ...rule, seen: Math.max(rule.seen, 2), active: true } : rule);
    },
  };
}

export function createInMemoryBridge(options: InMemoryBridgeOptions = {}): DesktopBridge {
  return createBridge(options, false);
}

export function createFixtureBatchBridge(): DesktopBridge {
  return createBridge({ items: [] }, true);
}

type BrowserFile = File & { webkitRelativePath?: string };

function browserFileSelection(file: BrowserFile): FileSelection {
  const displayName = file.webkitRelativePath || file.name;
  return { path: `browser://${displayName}`, displayName };
}

function chooseBrowserFiles(directory: boolean): Promise<File[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    if (directory) input.setAttribute('webkitdirectory', '');
    const settle = (files: File[]) => { input.remove(); resolve(files); };
    input.addEventListener('change', () => settle(Array.from(input.files ?? [])), { once: true });
    // A dismissed dialog fires `cancel` and nothing else. Without this the
    // promise never settled and the queue's one-action-at-a-time guard stayed
    // closed for the rest of the session.
    input.addEventListener('cancel', () => settle([]), { once: true });
    input.click();
  });
}

function browserFolderSelection(files: File[]): FolderSelection | undefined {
  if (!files.length) return undefined;
  const first = files[0] as BrowserFile;
  const displayName = first.webkitRelativePath?.split('/')[0] || first.name;
  return { path: `browser://${displayName}`, displayName, files: files.map((file) => browserFileSelection(file as BrowserFile)) };
}

/** Development-only conversion of browser File/DataTransfer objects into JSON-safe references. */
export function createBrowserSelectionBoundary(): SelectionBoundary {
  return {
    pickFiles: async () => (await chooseBrowserFiles(false)).map((file) => browserFileSelection(file as BrowserFile)),
    pickFolder: async () => browserFolderSelection(await chooseBrowserFiles(true)),
    pickExistingModelFiles: async () => undefined,
    resolveDrop: async (payload: unknown): Promise<SelectionResult> => {
      const transfer = payload as DataTransfer;
      const files = Array.from(transfer.files ?? []);
      const item = transfer.items?.[0] as (DataTransferItem & { getAsFileSystemHandle?: () => Promise<{ kind: string; name: string }> }) | undefined;
      const handle = await item?.getAsFileSystemHandle?.();
      if (handle?.kind === 'directory') return { folder: { path: `browser://${handle.name}`, displayName: handle.name, files: files.map((file) => browserFileSelection(file as BrowserFile)) } };
      return { files: files.map((file) => browserFileSelection(file as BrowserFile)) };
    },
  };
}
