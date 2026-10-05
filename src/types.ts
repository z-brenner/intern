export type QueueStatus = 'ready' | 'review' | 'processing' | 'waiting' | 'completed' | 'failed';
export type QueueView = 'queue' | 'review' | 'completed';
/** What a processing document is doing: reading its text, proposing a name, or being renamed. */
export type ProcessingStage = 'reading' | 'naming' | 'filing';

export interface QueueItem {
  id: string;
  originalFilename: string;
  status: QueueStatus;
  stage?: ProcessingStage;
  proposedFilename?: string;
  confidence?: number;
  description?: string;
  evidence?: { date?: string; type?: string; parties?: string };
  reason?: string;
  /**
   * The backend's code for why the item is in review or failed, kept beside
   * the humanized `reason`: which actions the backend accepts depends on it
   * (only some review codes can be retried), and the sentence cannot say.
   */
  errorCode?: string;
  /** The name the document was filed under, which can differ from the proposal (a " (2)" suffix, a layout folder). */
  filedName?: string;
  /** Where the filed document is, for showing it in its folder. */
  filedPath?: string;
  /** Completed without a rename: the document kept the name it arrived with. */
  keptOriginal?: boolean;
  /**
   * A rename stopped part-way and its files need checking before anything
   * else can happen: approve and keep are refused until "Check again", and
   * removing it needs the person to say they sorted the files out themselves.
   */
  parked?: boolean;
  progress?: number;
  cancelable?: boolean;
  undoable?: boolean;
  /**
   * A ready item whose name a person has already approved. While the queue
   * is busy with another document the backend keeps the approval and files
   * it between documents, so the item stays ready: decided, and waiting only
   * for the queue.
   */
  approved?: boolean;
  proposalRevision?: string;
  /**
   * A date the model proposed that Intern could not find written in the
   * document, so it was left out of the filename. Offered in review for a
   * person to accept with one click.
   */
  suggestedDate?: string;
  /** Every date the document states, in the order it states them, for a name that has none yet. */
  datesInDocument?: string[];
  /** The file's own last-modified date, a labelled last resort when the document states none. */
  fileModifiedDate?: string;
  /**
   * The reviewer's own spellings applied to this name. The evidence still
   * shows the document's words; these say how the name differs from them.
   */
  houseRules?: HouseRule[];
  /**
   * The name a document with nearly this text was already filed under, when
   * there is one - a second scan, a re-export, a copy saved again. Such a
   * document waits for a person instead of being filed twice.
   */
  nearDuplicateOf?: string;
}

/**
 * How filed documents are arranged under the destination: flat, or in
 * subfolders derived from each document's facts. A missing fact goes to a
 * named catch-all ("Undated", "Unsorted"), never the root.
 */
export type DestinationLayout = 'flat' | 'year' | 'year_type' | 'type' | 'party';

/**
 * Which model reads documents. `local` keeps every document on this
 * computer. `hosted` sends the distilled text of each document to a service
 * the user named, under their own API key - the one setting that moves
 * document text off the machine, and never the default.
 */
export type ModelSource = 'local' | 'hosted';

/** The wire format a hosted model speaks. */
export type HostedProvider = 'anthropic' | 'openai_compatible';

export interface AppSettings {
  destination: string;
  destinationLayout: DestinationLayout;
  startMinimized: boolean;
  automaticRename: boolean;
  /** Watched intake folder path; "" = none configured. */
  intakeFolder: string;
  intakeEnabled: boolean;
  /** Only for explicitly private local folders, never a shared sync root. */
  intakeLocalOnly?: boolean;
  /**
   * The watched folder is a OneDrive or SharePoint folder the person chose as
   * one only they add documents to, so its documents count as theirs without
   * a Microsoft check. Set by folder setup; never honored beside the verified
   * team Inbox.
   */
  intakeMyFolder?: boolean;
  /** false = only process documents uploaded from this machine ("mine" scope). */
  processOthersUploads: boolean;
  /** Overrides the hostname shown to other machines; "" = use hostname. */
  machineLabel: string;
  /** Keep Intern in the system tray when the window is closed. */
  runInBackground: boolean;
  /** Start Intern automatically when the user signs in. */
  startAtLogin: boolean;
  /**
   * Write a description record beside every document filed into the
   * destination (`<destination>/.intern/descriptions/`), so a SharePoint
   * column can be filled from it. Needs a destination folder.
   */
  recordDescriptions: boolean;
  modelSource: ModelSource;
  hostedProvider: HostedProvider;
  /** The hosted model's API root; "" = the provider's default. */
  hostedBaseUrl: string;
  /** The hosted model's name; "" = the provider's default, where there is one. */
  hostedModel: string;
}

/** Versioned backend-owned progress for the guided SharePoint setup. */
export interface OnboardingStatus {
  currentVersion: number;
  completedVersion: number;
  required: boolean;
  /**
   * Whether this build carries an enabled fixed SharePoint deployment. When
   * false, onboarding covers only the local model and the manual settings stay.
   */
  sharePointAvailable: boolean;
}

export type SharePointSetupPhase = 'enrollment_pending' | 'ready_to_activate' | 'active';

/** Fixed-deployment setup status; backend-only identifiers and local paths are omitted. */
export interface SharePointSetupStatus {
  phase: SharePointSetupPhase;
  account: {
    displayName: string;
    email: string;
  };
  site: 'InternTestSite';
  library: 'Files';
  intake: 'Inbox';
  destination: 'Filed';
  /**
   * While enrollment is pending, the stable `{ code, message }` for the
   * OneDrive record problem that kept the library from being confirmed, so
   * waiting is never silent when the records are the reason. Null otherwise.
   */
  problem?: SharePointSetupProblem | null;
}

export interface SharePointSetupProblem {
  code: string;
  message: string;
}

/** What Settings shows about the hosted model. The key itself never comes back. */
export interface HostedModelStatus {
  keyStored: boolean;
  /** The tail of the stored key, e.g. "…a1b2". */
  keyHint: string | null;
  /** The endpoint the saved settings resolve to, when they do. */
  endpoint: string | null;
  providers: HostedProviderDefaults[];
}

export interface HostedProviderDefaults {
  provider: HostedProvider;
  baseUrl: string;
  model: string;
}

/** A successful test connection: the calibration document came back named. */
export interface HostedModelTestResult {
  model: string;
  endpoint: string;
  filename: string;
  inferenceMillis: number;
}

/** One finished rename/undo operation from the durable receipt journal. */
export interface HistoryEntry {
  receiptId: string;
  queueItemId: string;
  /** Unix seconds when the operation reached its terminal stage. */
  at: number;
  direction: 'apply' | 'undo';
  kind: 'rename' | 'verified_copy';
  /** Only terminal receipts are listed. */
  stage: 'complete' | 'rolled_back';
  originalPath: string;
  newPath: string;
  /** The one-sentence description applied with the rename, when the item still has it. */
  description?: string;
}

export type CloudProvider = 'onedrive_personal' | 'onedrive_business' | 'sharepoint' | 'network_share';

/**
 * A folder recognised as living inside a OneDrive/SharePoint sync root, or
 * reached over the network (a UNC path or a mapped drive).
 */
export interface CloudLocation {
  provider: CloudProvider;
  displayName: string;
}

/** One sync root the sync client keeps on this computer. */
export interface CloudRoot {
  provider: CloudProvider;
  displayName: string;
  path: string;
}

/** What the description records are doing. */
export interface DescriptionsStatus {
  /** The setting, as saved. */
  enabled: boolean;
  /** Where records go, or "" when no destination is configured. */
  folder: string;
  recordedThisSession: number;
  lastRecordedAt: number | null;
  /** The last write that failed, until the next success. */
  lastError: string | null;
}

export interface BackfillResult {
  written: number;
  failed: number;
}

export interface IntakeMachine {
  machineId: string;
  machineName: string;
  userName: string;
  lastSeenAt: number;
  active: boolean;
}

export interface IntakeStatus {
  enabled: boolean;
  watching: boolean;
  folder: string;
  machineId: string;
  machineName: string;
  cloud: CloudLocation | null;
  machines: IntakeMachine[];
  heldForOthers: number;
  uploaderUnknown?: number;
  syncConflicts: number;
  awaitingHydration: number;
  /** Subfolders the last scan could not read; the rest was still scanned. */
  unreadableFolders: number;
  claimedByOthers: number;
  processedHere: number;
  lastScanAt: number | null;
  error: string | null;
  /** For a OneDrive or SharePoint folder, whether OneDrive is running; null when that does not apply or cannot be told. */
  oneDriveRunning?: boolean | null;
}

export interface SetupState {
  state: 'ready' | 'required' | 'downloading' | 'failed';
  downloadedBytes: number;
  totalBytes: number;
  error?: string;
  /** A hosted model is chosen and configured, so documents can be processed without the local one. */
  hostedModelReady?: boolean;
}

/** Which part of a name a learned spelling rewrites. */
export type RuleKind = 'party' | 'document_type';

/** A spelling the reviewer prefers over the document's own, learned from edits made in review. */
export interface HouseRule {
  kind: RuleKind;
  /** As the document writes it. */
  from: string;
  /** As the reviewer wrote it. */
  to: string;
}

/** A learned spelling and how settled it is. */
export interface LearnedRule extends HouseRule {
  id: string;
  /** How many times a reviewer has made exactly this change. */
  seen: number;
  /** Whether Intern applies it: made twice, or told to use it now. */
  active: boolean;
  /** Unix seconds of the latest edit that taught it. */
  learnedAt: number;
}
