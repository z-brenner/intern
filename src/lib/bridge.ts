import type { MicrosoftIntakeBridge } from '../features/intake/microsoft';
import type { AppSettings, BackfillResult, CloudLocation, CloudRoot, DescriptionsStatus, HistoryEntry, HostedModelStatus, HostedModelTestResult, IntakeStatus, LearnedRule, OnboardingStatus, QueueItem, SetupState, SharePointSetupStatus } from '../types';

/** A JSON-safe local document reference that Task 6 can pass to Tauri. */
export interface FileSelection {
  path: string;
  displayName: string;
}

export interface FolderSelection {
  path: string;
  displayName: string;
  files?: FileSelection[];
}

export interface SelectionResult {
  files?: FileSelection[];
  folder?: FolderSelection;
}

/** Intern needs one file: the text model. There is no vision projector. */
export interface ExistingModelFiles {
  modelPath: string;
}

/** Platform-specific selection is injected; the Tauri bridge remains path-only. */
export interface SelectionBoundary {
  pickFiles(): Promise<FileSelection[]>;
  pickFolder(): Promise<FolderSelection | undefined>;
  pickExistingModelFiles(): Promise<ExistingModelFiles | undefined>;
  resolveDrop(payload: unknown): Promise<SelectionResult>;
  /**
   * Native "save as" for the history CSV. Resolves with the chosen path, or
   * undefined when the dialog is canceled. Optional: the browser boundary has
   * no native save dialog, and the in-memory export ignores the path anyway.
   */
  pickHistoryExportPath?(): Promise<string | undefined>;
}

/**
 * The published user guide. It lives here, on the bridge, rather than in a
 * component: the desktop build hands this exact string to the operating
 * system's browser, and the Tauri capability in
 * `src-tauri/capabilities/default.json` is scoped to this site and the fixed `SUPPORT_LINKS`. One
 * constant keeps the two in step, and keeps every caller from being able to
 * ask the shell to open something else.
 */
export const GUIDE_URL = 'https://z-brenner.github.io/intern/guide.html';

/** The fixed recovery destinations SharePoint setup can send a person to. */
export type SupportLinkTarget = 'sharepoint-site' | 'onedrive-download';

/**
 * Where each support link goes. Like `GUIDE_URL`, these are the exact
 * addresses `src-tauri/capabilities/default.json` admits for the opener, so
 * the webview can open these and nothing else. The OneDrive address is
 * Microsoft's own download page; without a locale it redirects to the
 * visitor's language.
 */
export const SUPPORT_LINKS: Readonly<Record<SupportLinkTarget, string>> = {
  'sharepoint-site': 'https://teamcontoso.sharepoint.com/sites/InternTestSite',
  'onedrive-download': 'https://www.microsoft.com/microsoft-365/onedrive/download',
};

export interface DesktopBridge extends Partial<MicrosoftIntakeBridge> {
  listItems(): Promise<QueueItem[]>;
  addFiles(files: FileSelection[]): Promise<void>;
  addFolder(folder: FolderSelection): Promise<void>;
  pauseQueue(): Promise<void>;
  resumeQueue(): Promise<void>;
  cancel(id: string): Promise<void>;
  approve(id: string, filename: string, description: string): Promise<void>;
  keepOriginal(id: string): Promise<void>;
  retry(id: string): Promise<void>;
  remove(id: string): Promise<void>;
  undo(id: string): Promise<void>;
  getSettings(): Promise<AppSettings>;
  saveSettings(settings: AppSettings): Promise<void>;
  getSetup(): Promise<SetupState>;
  getOnboarding(): Promise<OnboardingStatus>;
  /** Rescans sync roots and reports fixed-library setup without side effects. */
  getSharePointSetup(): Promise<SharePointSetupStatus>;
  startSharePointSync(): Promise<SharePointSetupStatus>;
  activateOnboarding(): Promise<SharePointSetupStatus>;
  completeOnboarding(): Promise<void>;
  startModelDownload(): Promise<void>;
  setupCancel(): Promise<void>;
  setupChooseExisting(files: ExistingModelFiles): Promise<void>;
  clearHistory(): Promise<void>;
  /** Finished rename/undo operations, newest first (capped at 500). */
  historyList(): Promise<HistoryEntry[]>;
  /**
   * Write the rename history to `path` as CSV. Resolves with the number of
   * operations written. The path must be absolute in the desktop app; the
   * in-memory bridge ignores it.
   */
  historyExport(path: string): Promise<number>;
  /**
   * Abandon every item still waiting, for a folder chosen by mistake.
   *
   * Resolves with how many were dropped. Items being processed, awaiting a
   * decision, or already renamed are untouched.
   */
  discardWaiting(): Promise<number>;
  /**
   * Ask GitHub whether a newer signed release exists.
   *
   * This is the only network request Intern makes apart from the one-off model
   * download (and a hosted model, only if a person chose one in Settings). It
   * sends nothing but a request for the release manifest - no filenames, no
   * document contents, no identifier of any kind - and runs once when Intern
   * starts, again on a fixed interval for as long as it keeps running, and
   * whenever someone presses the button in Settings. The first two stop when
   * `skipUpdateChecks` is set; the button always works. Nothing is downloaded
   * or installed by a check on its own; that is always a separate, explicit
   * click.
   */
  checkForUpdate(): Promise<UpdateStatus>;
  /**
   * Download and install the update found by the last check. `onProgress`
   * hears how much of the download has arrived as it arrives.
   */
  installUpdate(onProgress?: UpdateProgressListener): Promise<void>;
  /** Current shared-intake watcher status. Resolves with zeros when intake is disabled. */
  intakeStatus(): Promise<IntakeStatus>;
  /** Wake the intake watcher for an immediate scan. No-op when intake is disabled. */
  scanIntakeNow(): Promise<void>;
  /**
   * Say whether a folder lives inside a OneDrive/SharePoint sync root.
   *
   * Purely a local path lookup against the sync client's configuration - no
   * network request is made and nothing about the folder leaves the machine.
   */
  classifyFolder(path: string): Promise<CloudLocation | null>;
  /**
   * The OneDrive accounts and SharePoint libraries the sync client keeps on
   * this computer, so Settings can offer them instead of making a person hunt
   * for the folder under their profile. A local lookup of the sync client's
   * own configuration; no network request is made.
   */
  cloudRoots(): Promise<CloudRoot[]>;
  /** How many documents a folder already holds, counted the way adding the folder to the queue would. */
  intakeFolderDocuments(path: string): Promise<number>;
  /** Create (or find) an "Inbox" folder inside a synced location's top folder; resolves with its path. */
  createInboxFolder(root: string): Promise<string>;
  /** Create (or find) the "Filed" folder beside `intakeFolder`; resolves with its path. */
  createFiledFolder(intakeFolder: string): Promise<string>;
  /** Start OneDrive, or open its folder when it is already running. */
  openOneDrive(): Promise<void>;
  /** What the description records are doing: on or off, where, and the last failure. */
  descriptionsStatus(): Promise<DescriptionsStatus>;
  /**
   * Write a description record for every document already filed and not
   * undone, for a records folder switched on after the fact. Rejected with
   * DESCRIPTIONS_DISABLED until the setting is saved on.
   */
  descriptionsBackfill(): Promise<BackfillResult>;
  /**
   * Open the published guide (`GUIDE_URL`) in the user's own browser.
   *
   * Deliberately takes no URL. Inside Tauri a bare `<a target="_blank">` has
   * nowhere to go, so this has to reach the shell - and a method that accepted
   * any address would hand the webview a general-purpose "open anything"
   * capability for the sake of one help link.
   */
  openGuide(): Promise<void>;
  /**
   * Open one of the fixed SharePoint setup support links (`SUPPORT_LINKS`) in
   * the user's own browser. Takes a name, not a URL, for the same reason
   * `openGuide` takes nothing.
   */
  openSupportLink(target: SupportLinkTarget): Promise<void>;
  /** Whether a hosted-model key is stored, and each provider's defaults. */
  hostedModelStatus(): Promise<HostedModelStatus>;
  /**
   * Store the hosted model's API key in the operating system's credential
   * store. It never enters the settings file. Rejected with
   * HOSTED_MODEL_KEY_EMPTY for a blank key.
   */
  hostedModelSetKey(key: string): Promise<void>;
  hostedModelClearKey(): Promise<void>;
  /**
   * Send the calibration document to the hosted model described by
   * `settings` (the dialog's draft, so what is tested is what is on screen)
   * with the stored key. This is the one deliberate network request to a
   * third party Intern makes, and it carries no document of the user's.
   */
  hostedModelTest(settings: AppSettings): Promise<HostedModelTestResult>;
  /**
   * The spellings review has taught Intern, newest first: a party or a
   * document type respelled in review, remembered, and applied on its own
   * once the same change has been made twice.
   */
  houseRulesList(): Promise<LearnedRule[]>;
  /** Stop applying a learned spelling. Documents still waiting go back to the document's own words. */
  houseRuleForget(id: string): Promise<void>;
  /** Apply a learned spelling from now on without waiting for a second edit. */
  houseRuleUse(id: string): Promise<void>;
}

/**
 * Optional capability, duck-typed like QueueEventSource: bridges that can push
 * intake status changes expose it; callers feature-detect `subscribeIntake`.
 */
export interface IntakeEventSource {
  subscribeIntake(handler: (status: IntakeStatus) => void): () => void;
}

/**
 * Optional capability, duck-typed like IntakeEventSource: bridges that can
 * push description-record status changes expose it.
 */
export interface DescriptionsEventSource {
  subscribeDescriptions(handler: (status: DescriptionsStatus) => void): () => void;
}

/**
 * How far an update download has got, from 0 to 1, or `undefined` while it
 * downloads from a server that did not say how large it is.
 */
export type UpdateProgressListener = (fraction: number | undefined) => void;

export type UpdateStatus =
  | { state: 'current'; currentVersion: string }
  | { state: 'available'; currentVersion: string; version: string; notes?: string; date?: string }
  /** Running outside the desktop shell, where there is nothing to update. */
  | { state: 'unsupported' };
