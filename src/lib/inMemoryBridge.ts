import { GUIDE_URL, SUPPORT_LINKS } from './bridge';
import { SUPPORTED_EXTENSIONS } from './formats';
import type { DesktopBridge, DocumentTextPage, FileSelection, FolderSelection, SelectionBoundary, SelectionResult, UpdateStatus } from './bridge';
import type { AddReport, AppSettings, CloudLocation, CloudRoot, DescriptionsStatus, HistoryEntry, HostedModelStatus, HostedModelTestResult, IntakeStatus, LearnedRule, OnboardingStatus, QueueItem, SetupState, SharePointSetupPhase, TierSetup, SharePointSetupProblem, SharePointSetupStatus } from '../types';
import { filenameExtension, leadingDate, validateFilename, validateLeafFilename } from './filenames';
import { OWN_NAME_MIN_CHARS, nameKey } from './ownNames';
import { humanizeReason, reasonCodes } from './reasons';
import { reviewLevelOf } from './reviewLevel';
import type { QueueBridgeEvent } from './tauriBridge';
import { outputBeside, rootFor } from '../features/intake/folderNames';
import type { MicrosoftIntakeBridge } from '../features/intake/microsoft';
import { nameParts } from '../features/queue/check';
import { isParked, retryAccepted, withoutReviewPrefix } from '../features/review/actions';

/** Review codes raised when a document arrives, before it is read: reading it again does not raise them. */
const PRE_ANALYSIS_CODES = new Set(['DUPLICATE', 'UPLOADER_UNVERIFIED']);

/** A filename as Windows compares two in one folder, as pipeline.rs name_key does: regardless of case and of trailing dots and spaces. */
const fileNameKey = (name: string) => name.replace(/[ .]+$/, '').toLowerCase();

/**
 * The demo's input and output folders. The browser build has no files behind
 * it, but the app is never used without both folders, so the demo starts with
 * a pair already chosen; `settings` replaces them, and empty ones walk setup.
 */
export const DEMO_INPUT_FOLDER = 'C:\\Intern\\Input';
export const DEMO_OUTPUT_FOLDER = 'C:\\Intern\\Output';

/**
 * Exact sizes of the two pinned models, from model-manifest.json: the fast
 * one setup downloads first, which is all a person waits for, and the
 * accurate one that follows behind the window.
 */
export const FAST_MODEL_BYTES = 2_740_937_888;
export const ACCURATE_MODEL_BYTES = 5_680_522_464;
/** What setup downloads before the first document: the fast model. */
export const PINNED_MODEL_BYTES = FAST_MODEL_BYTES;

const seedItems: QueueItem[] = [
  { id: 'employment', originalFilename: 'Employment Agreement - John Smith.pdf', status: 'ready', proposedFilename: '2024-04-12 Employment Agreement with John Smith.pdf', confidence: 0.98 },
  { id: 'lease', originalFilename: 'Lease Agreement - 123 Main St.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf', confidence: 0.72, description: 'Commercial lease agreement between landlord and tenant for 123 Main St.', evidence: { date: 'Sep 15, 2023', type: 'Lease Agreement', parties: 'ABC Properties LLC; TenantCo Inc.' }, reason: 'Lower confidence due to unclear document type keywords and multiple possible dates.' },
  { id: 'nda', originalFilename: 'NDA - Acme Corp.docx', status: 'ready', proposedFilename: '2024-03-01 Non-Disclosure Agreement with Acme Corp.docx', confidence: 0.95 },
  // Stays a PDF even though .xlsx is now supported: the reviewed QA capture
  // pins this queue's rendered contents, and changing a demo row would force a
  // re-sign-off for no product reason.
  // A stage and no percentage, as the desktop backend reports it: no figure
  // ever arrives from it, and a seeded "60%" hid that from every screenshot.
  { id: 'financials', originalFilename: 'Q1 Financials.pdf', status: 'processing', proposedFilename: '2024-03-31 Q1 Financial Statements.pdf', stage: 'naming' },
  { id: 'service', originalFilename: 'Service Agreement - BlueSky LLC.pdf', status: 'ready', proposedFilename: '2024-02-28 Service Agreement with BlueSky LLC.pdf', confidence: 0.96 },
  { id: 'minutes', originalFilename: 'Board Meeting Minutes - May 7, 2024.docx', status: 'waiting' },
  { id: 'invoice', originalFilename: 'Invoice INV-1001.pdf', status: 'waiting' },
  { id: 'notes', originalFilename: 'Notes from Call - 2024-05-02.txt', status: 'waiting' },
  { id: 'completed', originalFilename: 'Completed lease.pdf', status: 'completed', proposedFilename: '2024-01-22 Lease Agreement.pdf', filedName: '2024-01-22 Lease Agreement.pdf', filedPath: 'C:\\Filed\\2024-01-22 Lease Agreement.pdf', confidence: 0.93, undoable: true, description: 'Residential lease agreement for a twelve-month term beginning January 22, 2024.' },
];

/**
 * A morning's scans waiting on a reviewer: enough undecided items to clear
 * the queue from the keyboard, for the browser build's `?reviewBatch=1`.
 */
const reviewBatchItems: QueueItem[] = [
  { id: 'scan-0101', originalFilename: 'Scan 0101.pdf', status: 'review', proposedFilename: '2024-02-01 Engagement Letter with Northwind Traders.pdf', confidence: 0.74, description: 'Engagement letter between Northwind Traders and the firm for 2024 advisory work.', reason: humanizeReason('LOW_CONFIDENCE'), errorCode: 'LOW_CONFIDENCE' },
  { id: 'scan-0102', originalFilename: 'Scan 0102.pdf', status: 'review', proposedFilename: '2024-02-09 Invoice INV-3301 from Fabrikam Inc.pdf', confidence: 0.81, description: 'Invoice INV-3301 from Fabrikam Inc. for February consulting hours.', reason: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.', errorCode: 'DATE_AMBIGUOUS' },
  { id: 'scan-0103', originalFilename: 'Scan 0103.pdf', status: 'ready', proposedFilename: '2024-03-14 Notice of Termination to Contoso Ltd.pdf', confidence: 0.95, description: 'Notice terminating the services agreement with Contoso Ltd.' },
  { id: 'scan-0104', originalFilename: 'Scan 0104.pdf', status: 'review', proposedFilename: '2024-03-30 Statement of Work with Litware Inc.pdf', confidence: 0.69, description: 'Statement of work with Litware Inc. for the spring data migration.', reason: humanizeReason('LOW_CONFIDENCE'), errorCode: 'LOW_CONFIDENCE', partyChoices: ['Tailspin Toys'], nameParties: ['Litware Inc'] },
  { id: 'scan-0105', originalFilename: 'Scan 0105.pdf', status: 'ready', proposedFilename: '2024-04-02 Board Resolution of Tailspin Toys.pdf', confidence: 0.97, description: 'Board resolution of Tailspin Toys approving the annual budget.' },
  { id: 'scan-0106', originalFilename: 'Scan 0106.pdf', status: 'review', proposedFilename: '2024-04-18 Lease Amendment for 500 Pine St.pdf', confidence: 0.77, description: 'First amendment to the lease of 500 Pine St.', reason: humanizeReason('LOW_CONFIDENCE'), errorCode: 'LOW_CONFIDENCE' },
  { id: 'scan-0107', originalFilename: 'Scan 0107.pdf', status: 'waiting' },
];

/**
 * What a seeded proposal naming two parties was composed from, so that naming
 * the organization in Settings renames it the way the desktop backend's
 * counterparty view does. A row naming one party needs no entry: the only
 * party is never left out.
 */
interface SeedName { stem: string; parties: [string, string]; extension: string }

const seedNames: Record<string, SeedName> = {
  lease: { stem: '2023-09-15 Lease Agreement', parties: ['ABC Properties LLC', 'TenantCo Inc.'], extension: 'pdf' },
};

/**
 * The backend's is_own_name: the same key, or that key followed by more whole
 * words, either way round. An own name under four characters names nobody.
 */
function isOwnName(party: string, own: string): boolean {
  const ownKey = nameKey(own);
  const partyKey = nameKey(party);
  if ([...ownKey].length < OWN_NAME_MIN_CHARS || !partyKey) return false;
  return partyKey === ownKey || partyKey.startsWith(`${ownKey} `) || ownKey.startsWith(`${partyKey} `);
}

/** A party as a filename carries it, without the trailing period a name segment cannot end in. */
const nameSegment = (party: string) => party.replace(/[. ]+$/, '');

/** The counterparty view of a seeded two-party proposal, composed as its name. */
function seededName(name: SeedName, own: string[]): Pick<QueueItem, 'proposedFilename' | 'omittedParties'> {
  const mine = (party: string) => own.some((entry) => isOwnName(party, entry));
  const others = name.parties.filter((party) => !mine(party));
  const omitted = name.parties.filter(mine);
  if (!omitted.length || !others.length) {
    return { proposedFilename: `${name.stem} between ${nameSegment(name.parties[0])} and ${nameSegment(name.parties[1])}.${name.extension}` };
  }
  // "between" needs two sides; the one left is "with" it.
  return { proposedFilename: `${name.stem} with ${nameSegment(others[0])}.${name.extension}`, omittedParties: omitted };
}

/**
 * The review codes a seeded row's reading carried, where the row itself says
 * why it waits only in a sentence of its own. The review level is judged by
 * codes, so this is what makes the demo's lease ready with a note at
 * Balanced - a type the document does not confirm, a model unsure of its
 * reading - while the row at Strict stays word for word what it was.
 */
const seedReadings: Record<string, string[]> = {
  lease: ['TYPE_UNSUPPORTED', 'LOW_CONFIDENCE'],
};

/**
 * The demo's copy of the Balanced review level, for the rows it seeds. The
 * rule is the engine's and is written there once; this stands in for it
 * where there is no engine, and reads only what a demo row has: its codes,
 * the model's own figure, and its name.
 */
const BALANCED_CODES = new Set(['TYPE_MISSING', 'TYPE_UNSUPPORTED', 'TYPE_INFERRED', 'DESCRIPTION_UNSUPPORTED', 'LOW_CONFIDENCE']);
/** The engine's READY_CONFIDENCE (validate.rs): under it the model's own figure holds a reading at every level. */
const READY_CONFIDENCE = 0.6;

function readyAtBalanced(item: QueueItem, codes: readonly string[]): boolean {
  // No codes is a reason given in words, which nobody can judge: it waits.
  if (!codes.length || !codes.every((code) => BALANCED_CODES.has(code))) return false;
  // A date, and a type that is more than the word "Document".
  if (!item.proposedFilename || !leadingDate(item.proposedFilename)) return false;
  const type = nameParts(item.proposedFilename, []).find((part) => part.field === 'type')?.text.trim() ?? '';
  if (!type || type.toLowerCase() === 'document') return false;
  // Unsure by what Intern measured only: a low figure of the model's own waits.
  if (codes.includes('LOW_CONFIDENCE') && !(item.confidence !== undefined && item.confidence >= READY_CONFIDENCE)) return false;
  // A title standing in for a type the document did not confirm waits.
  if (codes.includes('TYPE_UNSUPPORTED') && codes.includes('TYPE_INFERRED') && !codes.includes('TYPE_MISSING')) return false;
  return true;
}

/**
 * An item as the backend lists it (queue_item_dto): the other parties a name
 * could give, and the words they replace, go only with an item in review,
 * whose name a person is deciding.
 */
function listed({ partyChoices, nameParties, ...item }: QueueItem): QueueItem {
  if (item.status !== 'review' || !partyChoices?.length) return item;
  return { ...item, partyChoices: [...partyChoices], ...(nameParties?.length ? { nameParties: [...nameParties] } : {}) };
}

/** Own names as the backend stores them: trimmed, blank lines left out. */
const ownNames = (names: string[]) => names.map((name) => name.trim()).filter((name) => name.length > 0);

/**
 * Plausible finished operations for browser dev and tests, newest first —
 * the order the desktop backend returns. Timestamps are fixed so renders are
 * deterministic. Only the lease (receipt 3) belongs to an item still in the
 * seeded queue; the others were cleared from it long ago.
 */
const seedHistory: HistoryEntry[] = [
  { receiptId: '9', queueItemId: 'minutes-filed', at: 1716282900, direction: 'undo', kind: 'rename', stage: 'complete', originalPath: 'C:\\Filed\\2024-05-07 Board Meeting Minutes.docx', newPath: 'C:\\Drop\\Board Meeting Minutes - May 7, 2024.docx' },
  { receiptId: '7', queueItemId: 'minutes-filed', at: 1716282600, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\Board Meeting Minutes - May 7, 2024.docx', newPath: 'C:\\Filed\\2024-05-07 Board Meeting Minutes.docx' },
  { receiptId: '5', queueItemId: 'invoice-filed', at: 1716196500, direction: 'apply', kind: 'verified_copy', stage: 'complete', originalPath: 'C:\\Drop\\Invoice INV-1001.pdf', newPath: 'D:\\Archive\\2024-05-02 Invoice INV-1001 from BlueSky LLC.pdf', },
  { receiptId: '3', queueItemId: 'completed', at: 1716108300, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\Completed lease.pdf', newPath: 'C:\\Filed\\2024-01-22 Lease Agreement.pdf' },
  { receiptId: '1', queueItemId: 'nda-filed', at: 1716021900, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\Drop\\NDA - Acme Corp.docx', newPath: 'C:\\Filed\\2024-03-01 Non-Disclosure Agreement with Acme Corp.docx' },
];

export interface InMemoryBridgeOptions {
  items?: QueueItem[];
  /**
   * The review codes behind a row that says why it waits only in words, by
   * id: what the review level judges it by. The demo's own queue has them
   * for its lease; a queue passed in has none unless it brings them.
   */
  readings?: Record<string, string[]>;
  /** Finished renames and undos for Rename history to list, newest first, in place of the demo's own. */
  history?: HistoryEntry[];
  /** Settings already saved, such as an earlier release's manual intake folders. */
  settings?: Partial<AppSettings>;
  setup?: Partial<SetupState>;
  /** The accurate model's own setup, when not what the first model's implies. */
  accurate?: Partial<TierSetup>;
  /** The quick model's own setup, when it is not what setup is about. */
  quick?: Partial<TierSetup>;
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
  /**
   * Push queue changes the bridge makes on its own - a re-analysis coming
   * back - the way the desktop backend's `queue://changed` does. Off by
   * default: a subscription makes the queue read itself once more on start,
   * which tests that script the first read do not expect.
   */
  liveEvents?: boolean;
  /**
   * Hands over a way to change the queue with nobody asking, as the desktop
   * backend does: `change` a document - read again, given another proposal,
   * filed from another machine - or take it out of the queue (`leave`). Each
   * says the queue changed, so it is for a bridge with `liveEvents`.
   */
  byItself?: (queue: QueueByItself) => void;
  /** How long a re-analysis takes before the item is back in review. */
  analysisDelayMs?: number;
  /**
   * File an approval only while no other document is being processed, as
   * the backend's begin_applying does; otherwise keep the approved name and
   * leave the item ready, for the queue to file between documents. Off by
   * default: the demo's processing document never finishes.
   */
  deferApprovalsWhileBusy?: boolean;
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
  }
  return { id: `file-${crypto.randomUUID()}`, originalFilename: file.displayName, status: 'waiting' };
}

/**
 * Why the desktop backend would leave a file out of an add, by its name alone
 * (the in-memory bridge has no bytes to weigh): a lock or hidden file, or an
 * extension it does not read. `undefined` means it would be queued.
 */
function skipCode(file: FileSelection): string | undefined {
  const name = file.displayName.split('/').at(-1) ?? file.displayName;
  if (name.startsWith('~$') || name.startsWith('.')) return 'TEMPORARY_FILE';
  // A document Intern renamed because it could not read it waits in the
  // queue for a person, and is never taken in again as a new one.
  if (withoutReviewPrefix(name) !== name) return 'NEEDS_REVIEW_FILE';
  const dot = name.lastIndexOf('.');
  const extension = dot > 0 ? name.slice(dot + 1).toLowerCase() : '';
  return SUPPORTED_EXTENSIONS.includes(extension) ? undefined : 'UNSUPPORTED_FORMAT';
}

/** Whether `path` is inside `folder`, compared the way Windows does. */
function within(folder: string, path: string): boolean {
  const root = folder.replace(/[\\/]+$/, '').toLowerCase();
  const lower = path.toLowerCase();
  return root.length > 0 && (lower.startsWith(`${root}\\`) || lower.startsWith(`${root}/`));
}

/**
 * The backend's folder-pair rules on save (validate_intake_settings): while
 * watching, the output folder may not be the input folder or inside it, and -
 * asked only of a save that chooses the folders - the input folder may not be
 * inside the output folder.
 */
function folderPairProblem(previous: AppSettings, next: AppSettings): { code: string; message: string } | undefined {
  const input = next.intakeFolder.trim();
  const output = next.destination.trim();
  if (!next.intakeEnabled || !input || !output) return undefined;
  const same = input.replace(/[\\/]+$/, '').toLowerCase() === output.replace(/[\\/]+$/, '').toLowerCase();
  if (same || within(input, output)) return { code: 'DESTINATION_INSIDE_INTAKE', message: 'the destination cannot be the intake folder or live inside it' };
  const choosesFolders = !previous.intakeEnabled
    || previous.intakeFolder.trim() !== input
    || previous.destination.trim() !== output;
  if (choosesFolders && within(output, input)) return { code: 'INTAKE_INSIDE_DESTINATION', message: 'the intake folder cannot live inside the destination' };
  return undefined;
}

/** "Lease.pdf", then "Lease (2).pdf", "Lease (3).pdf": the backend's numbering. */
/**
 * The document viewer's stand-in for a document, in the browser build, which
 * has no files behind it: a page made from what the row knows - its type,
 * date and parties as the evidence quotes them, and its description - and a
 * second that says what it is. The words the viewer highlights are on it, so
 * the preview shows that too.
 */
function demoPages(item: QueueItem): DocumentTextPage[] {
  const name = (item.filedName ?? item.proposedFilename ?? item.originalFilename).replace(/\.[A-Za-z0-9]{1,6}$/, '');
  const date = item.evidence?.date ?? leadingDate(name);
  const type = item.evidence?.type ?? name.replace(/^\d{4}-\d{2}-\d{2}\s*/, '');
  const parties = (item.evidence?.parties ?? '').split(';').map((party) => party.trim()).filter(Boolean);
  const first = [
    type.toUpperCase(),
    '',
    ...(date ? [`Date: ${date}`] : []),
    ...parties.map((party, index) => `${index === 0 ? 'From' : 'To'}: ${party}`),
    '',
    item.description ?? 'The terms of this document are set out on the pages that follow.',
  ];
  return [
    { number: 1, text: first.join('\n'), ocr: false },
    { number: 2, text: 'This page stands in for the rest of the document: the browser preview has no files behind it.', ocr: false },
  ];
}

/**
 * The same pages as a PDF, for the viewer's Original tab: Helvetica, a line
 * at a time, laid out by hand with the byte offsets PDF readers look for.
 */
function demoPdf(pages: readonly DocumentTextPage[]): Uint8Array {
  const plain = (text: string) => text.replace(/[^\x20-\x7e]/g, '?').replace(/[\\()]/g, (character) => `\\${character}`);
  const wrap = (line: string) => line.match(/.{1,80}(\s|$)/g)?.map((part) => part.trimEnd()) ?? [''];
  const objects: string[] = ['<< /Type /Catalog /Pages 2 0 R >>', ''];
  const kids: string[] = [];
  const font = 3 + pages.length * 2;
  pages.forEach((page, index) => {
    const pageObject = 3 + index * 2;
    kids.push(`${pageObject} 0 R`);
    const lines = page.text.split('\n').flatMap(wrap);
    const stream = `BT /F1 12 Tf 16 TL 72 720 Td ${lines.map((line) => `(${plain(line)}) Tj T*`).join(' ')} ET`;
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents ${pageObject + 1} 0 R /Resources << /Font << /F1 ${font} 0 R >> >> >>`);
    objects.push(`<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`);
  });
  objects[1] = `<< /Type /Pages /Kids [${kids.join(' ')}] /Count ${pages.length} >>`;
  objects.push('<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');
  let body = '%PDF-1.4\n';
  const offsets = objects.map((object, index) => {
    const offset = body.length;
    body += `${index + 1} 0 obj\n${object}\nendobj\n`;
    return offset;
  });
  const xref = body.length;
  body += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n${offsets.map((offset) => `${String(offset).padStart(10, '0')} 00000 n \n`).join('')}`;
  body += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return new TextEncoder().encode(body);
}

function numberedName(name: string, index: number): string {
  if (index === 1) return name;
  const dot = name.lastIndexOf('.');
  return dot > 0 ? `${name.slice(0, dot)} (${index})${name.slice(dot)}` : `${name} (${index})`;
}

/** See `InMemoryBridgeOptions.byItself`. */
export interface QueueByItself { change(id: string, change: Partial<QueueItem>): void; leave(id: string): void }

function createBridge(options: InMemoryBridgeOptions, fixtureBatch: boolean): DesktopBridge {
  let items = (options.items ?? seedItems).map((item) => ({ ...item }));
  let history = (options.history ?? seedHistory).map((entry) => ({ ...entry }));
  // skipUpdateChecks is spelled out because the desktop backend always
  // returns it - false for a file that predates it - so a bridge that left it
  // out would differ from the real one in exactly the case worth testing.
  // reviewLevel likewise: strict for a file that predates it.
  let settings: AppSettings = { destination: DEMO_OUTPUT_FOLDER, destinationLayout: 'flat', ourNames: [], startMinimized: false, automaticRename: false, reviewLevel: 'strict', intakeFolder: DEMO_INPUT_FOLDER, intakeEnabled: true, intakeLocalOnly: true, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, skipUpdateChecks: false, recordDescriptions: false, modelSource: 'local', modelPreference: 'automatic', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '', ...options.settings };
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
  // The accurate model, which the backend downloads behind the window once
  // the fast one is in place.
  let accurate: TierSetup = {
    tier: 'accurate',
    displayName: 'Qwen3.5 9B',
    ...(setup.state === 'ready'
      ? { state: 'ready', downloadedBytes: ACCURATE_MODEL_BYTES }
      : { state: 'required', downloadedBytes: 0 }),
    totalBytes: ACCURATE_MODEL_BYTES,
    ...options.accurate,
  };
  // The quick model's own entry, when it is not the one setup is about: as
  // when only the accurate model's file was chosen from disk, which leaves
  // the quick one to be fetched when asked.
  let quick: TierSetup | undefined = options.quick && {
    tier: 'fast', displayName: 'Qwen3.5 4B', state: 'required', downloadedBytes: 0, totalBytes: FAST_MODEL_BYTES, ...options.quick,
  };
  const tiers = (): TierSetup[] => [
    quick ? { ...quick } : { tier: 'fast', displayName: 'Qwen3.5 4B', state: setup.state, downloadedBytes: setup.downloadedBytes, totalBytes: setup.totalBytes, error: setup.state === 'failed' ? setup.error : undefined },
    { ...accurate },
  ];
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
  let accurateTimer: ReturnType<typeof setInterval> | undefined;
  const accurateStep = Math.max(1, Math.ceil(ACCURATE_MODEL_BYTES / 4));
  const startAccurate = () => {
    if (accurateTimer || accurate.state === 'ready') return;
    accurate = { ...accurate, state: 'downloading', error: undefined };
    accurateTimer = setInterval(() => {
      const downloadedBytes = Math.min(accurate.totalBytes, accurate.downloadedBytes + accurateStep);
      accurate = { ...accurate, downloadedBytes, state: downloadedBytes >= accurate.totalBytes ? 'ready' : 'downloading' };
      if (accurate.state === 'ready' && accurateTimer) { clearInterval(accurateTimer); accurateTimer = undefined; }
    }, downloadIntervalMs);
  };
  let quickTimer: ReturnType<typeof setInterval> | undefined;
  const startQuick = () => {
    if (quickTimer || !quick || quick.state === 'ready') return;
    quick = { ...quick, state: 'downloading', error: undefined };
    quickTimer = setInterval(() => {
      if (!quick) return;
      const downloadedBytes = Math.min(quick.totalBytes, quick.downloadedBytes + Math.ceil(quick.totalBytes / 4));
      quick = { ...quick, downloadedBytes, state: downloadedBytes >= quick.totalBytes ? 'ready' : 'downloading' };
      if (quick.state === 'ready' && quickTimer) { clearInterval(quickTimer); quickTimer = undefined; }
    }, downloadIntervalMs);
  };
  const idByPath = new Map<string, string>();
  const pathById = new Map<string, string>();
  const update = (id: string, change: Partial<QueueItem>) => { items = items.map((item) => item.id === id ? { ...item, ...change } : item); };
  // Replaces an item outright, so a field cleared here is gone rather than
  // carried over by the spread in `update`.
  const replace = (next: QueueItem) => { items = items.map((item) => item.id === next.id ? next : item); };
  const find = (id: string) => {
    const item = items.find((entry) => entry.id === id);
    if (!item) throw { code: 'ITEM_NOT_FOUND', message: 'queue item does not exist' };
    return item;
  };
  // The backend's refusal for a parked item (unsettled_files), word for word.
  const parkedRefusal = () => ({ code: 'RECONCILIATION_REQUIRED', message: 'files need checking: use Check again' });
  const queueListeners = new Set<(event: QueueBridgeEvent) => void>();
  const queueChanged = () => queueListeners.forEach((listener) => listener({ type: 'changed' }));
  // Long enough to see the item wait before it is back in review.
  const analysisDelayMs = options.analysisDelayMs ?? 800;
  /**
   * What review leaves behind, by queue status: no proposal while waiting, no
   * failure once settled. Nor a mark `byItself` gave the row: a filing that
   * settles here is the person's, and the backend marks none of those.
   */
  const settledFields = ({ stage: _stage, progress: _progress, reason: _reason, errorCode: _errorCode, parked: _parked, renamedWithoutApproval: _renamed, ...rest }: QueueItem): QueueItem => rest;
  // What an approval leaves: the backend clears the proposal's reasons with
  // it (approve_user_edit), so the codes behind them go too. A document a
  // person approved is renamed with no note left on it, whatever it carried.
  const approvedFields = (item: QueueItem): QueueItem => { const { reasonCodes: _reasonCodes, ...rest } = settledFields(item); return rest; };
  // Pipeline::check_again, for a parked item: here the files are always
  // found in order. A renamed copy whose original could not be deleted was
  // safe all along, so checking finishes the filing; any other rename that
  // stopped part-way had not happened, and the item is back in review.
  const checkAgain = (item: QueueItem): QueueItem => {
    const checked: QueueItem = item.errorCode === 'SOURCE_DELETE_FAILED'
      ? { ...settledFields(item), status: 'completed', filedName: item.proposedFilename, undoable: true, keptOriginal: false }
      : { ...settledFields(item), status: 'review', parked: false };
    replace(checked);
    return checked;
  };
  options.byItself?.({
    change: (id, change) => { update(id, change); queueChanged(); },
    leave: (id) => { items = items.filter((item) => item.id !== id); queueChanged(); },
  });
  // The seeded proposals still named from their facts. A name a person
  // approved is theirs: it leaves this map and keeps what they typed.
  const composedNames = new Map(options.items ? [] : Object.entries(seedNames));
  // Mirrors the backend's refresh_own_names: a document still waiting for a
  // decision is named by the other side when Settings names the organization,
  // and by both sides again when it no longer does.
  const renameWaiting = () => {
    const own = settings.ourNames ?? [];
    items = items.map((item) => {
      const name = composedNames.get(item.id);
      if (!name || (item.status !== 'ready' && item.status !== 'review')) return item;
      const { proposedFilename, omittedParties } = seededName(name, own);
      if (proposedFilename === item.proposedFilename && omittedParties?.join('\n') === item.omittedParties?.join('\n')) return item;
      // A new revision, as the backend's recompose records one, so an open
      // inspector takes the new name instead of keeping the old one as a draft.
      const renamed: QueueItem = { ...item, proposedFilename, proposalRevision: String(Number(item.proposalRevision ?? 1) + 1) };
      delete renamed.omittedParties;
      return omittedParties ? { ...renamed, omittedParties } : renamed;
    });
  };
  renameWaiting();
  // Mirrors the backend's re-judging when the review level changes: a
  // document that waits for a decision, and that no person has approved,
  // is judged again from the reading it has, with nothing read again. At
  // Balanced one whose codes the level lets through is ready, with those
  // codes still on it as notes. Back at Strict it waits again as it did:
  // what it carried while it waited is kept here meanwhile, so the queue at
  // Strict is the queue as it was. Automatic renaming is not acted out, as
  // it never has been here: nothing in the demo is renamed by itself. So the
  // window says what a level change did here (levelChanged), from the rows
  // this re-judging leaves; but no row here is marked renamed with nobody's
  // approval (renamedWithoutApproval), so the window has no such rename to
  // say (renamedByItself), and none is made up for it. A copy filed here is
  // not marked either: the backend's copy record is approved by the queue
  // itself. Only `byItself` can mark a row, changing it as the backend does.
  const readings = new Map(Object.entries(options.readings ?? (options.items ? {} : seedReadings)));
  const waitedWith = new Map<string, Pick<QueueItem, 'reason' | 'errorCode' | 'reasonCodes'>>();
  /** A waiting row made ready at Balanced when its codes allow it; otherwise as it is. */
  const readyWithNotes = (item: QueueItem): QueueItem => {
    if (item.status !== 'review' || isParked(item)) return item;
    const codes = readings.get(item.id) ?? (item.reasonCodes?.length ? item.reasonCodes : reasonCodes(item.errorCode ?? ''));
    if (!readyAtBalanced(item, codes)) return item;
    waitedWith.set(item.id, { reason: item.reason, errorCode: item.errorCode, reasonCodes: item.reasonCodes });
    const { errorCode: _errorCode, ...rest } = item;
    return { ...rest, status: 'ready', reason: humanizeReason(codes.join(', ')), reasonCodes: [...codes] };
  };
  /**
   * A row that is ready with notes, waiting again as Strict has it: one
   * made ready above as it waited before, and one that came ready with its
   * codes with those codes as why it waits. Any other row as it is.
   */
  const waitingAgain = (item: QueueItem): QueueItem => {
    if (item.status !== 'ready' || item.approved || item.copyOf !== undefined) return item;
    const waited = waitedWith.get(item.id);
    if (!waited) return item.reasonCodes?.length ? { ...item, status: 'review' } : item;
    waitedWith.delete(item.id);
    const { reason: _reason, errorCode: _errorCode, reasonCodes: _reasonCodes, ...rest } = item;
    const back: QueueItem = { ...rest, status: 'review' };
    if (waited.reason !== undefined) back.reason = waited.reason;
    if (waited.errorCode !== undefined) back.errorCode = waited.errorCode;
    if (waited.reasonCodes !== undefined) back.reasonCodes = waited.reasonCodes;
    return back;
  };
  const rejudgeWaiting = () => {
    const balanced = reviewLevelOf(settings) === 'balanced';
    items = items.map(balanced ? readyWithNotes : waitingAgain);
    // A row decided meanwhile - approved, kept, removed - is no longer waiting to be put back.
    if (!balanced) waitedWith.clear();
  };
  // A queue that starts at Balanced starts judged by it. One that starts at
  // Strict is shown as it was given: nothing changed the level.
  if (reviewLevelOf(settings) === 'balanced') rejudgeWaiting();
  // As the backend: pressing Download asked for both models, and the
  // accurate one follows once the fast one is in place.
  const finishDownload = () => { if (downloadTimer) clearInterval(downloadTimer); downloadTimer = undefined; setup = { ...setup, state: 'ready', downloadedBytes: setup.totalBytes }; startAccurate(); };
  // Items filed as a copy of another (Pipeline::file_duplicate). A copy is
  // never what a later duplicate is named after.
  const copyIds = new Set<string>();
  // Where each copy goes: beside the original, or the output folder's root.
  const copyPaths = new Map<string, string>();
  // Whether a person paused the queue: a duplicate added meanwhile is staged
  // as a copy, and filed once the queue runs again.
  let paused = false;
  // The names NEEDS REVIEW marks took away, by item (the backend's
  // review_renames journal): Keep original gives back only a name Intern
  // took, never a prefix the file arrived with.
  const marks = new Map<string, string>();
  // A document the demo cannot read. The bridge has no bytes to try, so a
  // file whose name says it is password-protected stands in for one: it
  // waits in review, and - as the backend does, only for a file in the
  // input folder (Pipeline::review_mark) - its file is renamed
  // "NEEDS REVIEW - ..." where it lies.
  const unreadableFrom = (file: FileSelection): QueueItem | undefined => {
    const name = file.displayName.split('/').at(-1) ?? file.displayName;
    if (!/password/i.test(name)) return undefined;
    const id = `file-${crypto.randomUUID()}`;
    const item: QueueItem = { id, originalFilename: name, status: 'review', errorCode: 'PASSWORD_PROTECTED', reason: humanizeReason('PASSWORD_PROTECTED') };
    if (!inInputFolder(file.path)) return item;
    marks.set(id, name);
    return { ...item, originalFilename: `NEEDS REVIEW - ${name}` };
  };
  // Whether a path is inside the input folder Settings names, compared as
  // the backend compares them: separators alike, case ignored.
  const inInputFolder = (path: string): boolean => {
    const key = (value: string) => value.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
    const folder = key(settings.intakeFolder.trim());
    return folder !== '' && key(path).startsWith(`${folder}/`);
  };
  // Pipeline::file_duplicate, for a document whose bytes are already filed.
  // The bridge has no bytes to compare, so a file named like the original of
  // a filed document stands in for the same bytes added again. With an output
  // folder it is filed there at once as "COPY - <the original's filed name>",
  // " (2)" and on for further copies, beside the original; without one it
  // waits in review as a duplicate, untouched.
  const duplicateOf = (file: FileSelection): QueueItem | undefined => {
    const name = file.displayName.split('/').at(-1) ?? file.displayName;
    const original = items.find((entry) => entry.status === 'completed' && !copyIds.has(entry.id) && fileNameKey(entry.originalFilename) === fileNameKey(name));
    if (!original) return undefined;
    const filedAs = original.keptOriginal ? original.originalFilename : original.filedName ?? original.originalFilename;
    const id = `file-${crypto.randomUUID()}`;
    if (!settings.destination.trim()) return { id, originalFilename: name, status: 'review', errorCode: 'DUPLICATE', reason: `Duplicate of ${filedAs}` };
    const extension = filenameExtension(name);
    const originalExtension = filenameExtension(filedAs);
    const stem = originalExtension === undefined ? filedAs : filedAs.slice(0, filedAs.length - originalExtension.length - 1);
    const suffix = extension === undefined ? '' : `.${extension}`;
    const taken = new Set(items.map((entry) => fileNameKey(entry.filedName ?? '')));
    let copy = `COPY - ${stem}${suffix}`;
    for (let index = 2; taken.has(fileNameKey(copy)); index += 1) copy = `COPY - ${stem} (${index})${suffix}`;
    // Beside the original when it was filed into a folder, otherwise in the
    // output folder's root, in that folder's own spelling of separators.
    const filedFolder = !original.keptOriginal && original.filedPath
      ? original.filedPath.slice(0, Math.max(original.filedPath.lastIndexOf('\\'), original.filedPath.lastIndexOf('/')))
      : '';
    const folder = filedFolder || settings.destination.replace(/[\\/]+$/, '');
    const separator = folder.includes('/') && !folder.includes('\\') ? '/' : '\\';
    copyIds.add(id);
    copyPaths.set(id, `${folder}${separator}${copy}`);
    // Paused, the copy is staged and waits for the queue, filed by nobody's
    // decision (Pipeline::apply_pending_ready); otherwise it is filed now.
    if (paused) return { id, originalFilename: name, status: 'ready', proposedFilename: copy, approved: true, copyOf: filedAs, reason: humanizeReason('EXACT_COPY') };
    return { id, originalFilename: name, status: 'completed', proposedFilename: copy, filedName: copy, filedPath: `${folder}${separator}${copy}`, undoable: true, keptOriginal: false, copyOf: filedAs, reason: humanizeReason('EXACT_COPY') };
  };
  // The scheduler's pass once the queue runs again: staged copies are filed.
  const fileStagedCopies = () => {
    items = items.map((item) => {
      if (item.status !== 'ready' || item.copyOf === undefined || !copyIds.has(item.id)) return item;
      const { approved: _approved, ...rest } = item;
      const filed: QueueItem = { ...rest, status: 'completed', undoable: true, keptOriginal: false };
      if (item.proposedFilename !== undefined) filed.filedName = item.proposedFilename;
      const filedPath = copyPaths.get(item.id);
      if (filedPath !== undefined) filed.filedPath = filedPath;
      return filed;
    });
  };
  // The names taken in the input folder, as Windows compares them, so a copy
  // is numbered the way the desktop backend numbers one; and what each
  // original was copied as, so the same file added again while its copy
  // still waits is copied once.
  // Seeded with what is still waiting, which is in the input folder; a filed
  // document has left it.
  const inputNames = new Set(items.filter((item) => item.status !== 'completed').map((item) => fileNameKey(item.originalFilename)));
  const copyOf = new Map<string, string>();
  // Copies held in the simulated verified Inbox, by path: they stay in the
  // folder, as the backend leaves them for the watcher, and "Process them
  // too" finds them there and holds them again.
  const heldCopies = new Map<string, string>();
  // What Clear queue took out of the queue, by path, with the name it is
  // there under: still in the input folder, not taken in again until added.
  const setAside = new Map<string, string>();
  const inputFolder = () => {
    if (!settings.intakeEnabled || !settings.intakeFolder.trim()) throw { code: 'INPUT_FOLDER_NOT_SET', message: 'no Queue folder is set up; choose one in Settings' };
    return settings.intakeFolder.replace(/[\\/]+$/, '');
  };
  // Every document queued in the input folder - copied there, or found
  // there - meets the backend's own rules: an exact copy of a filed one is
  // filed as a COPY, and one it cannot read is renamed NEEDS REVIEW where it
  // lies. The names the input folder then holds follow.
  const queueAt = (path: string, name: string) => {
    const file = { path, displayName: name };
    const item = duplicateOf(file) ?? unreadableFrom(file) ?? itemFromFile(file, fixtureBatch);
    idByPath.set(path, item.id);
    pathById.set(item.id, path);
    items = [...items, item];
    if (item.status === 'completed' || item.originalFilename !== name) inputNames.delete(fileNameKey(name));
    if (item.status !== 'completed') inputNames.add(fileNameKey(item.originalFilename));
    return item;
  };
  const waitingCopy = (source: string) => {
    const id = copyOf.get(source);
    return items.some((item) => item.id === id && item.status !== 'completed' && item.status !== 'failed');
  };
  // Mirrors the backend's input_copy: what it would leave out is reported by
  // name, a folder is flattened (passing over silently what is not a
  // document, as Add folder always has), a clash is numbered, a file already
  // in the input folder is queued where it is, and a filed one in the output
  // folder is not copied back. In the simulated deployment's verified Inbox a
  // copy is held, as the backend holds one until OneDrive has uploaded it.
  // There are no bytes here, so the same content under two names is two
  // documents.
  const copyToInput = (selection: SelectionResult): AddReport => {
    const input = inputFolder();
    const separator = input.includes('\\') ? '\\' : '/';
    const report: AddReport = { added: 0, alreadyQueued: 0, skipped: [], copied: 0, inputFolder: input };
    const verifiedInbox = sharePointEnabled && within(libraryRoot, input);
    const files: FileSelection[] = [...(selection.files ?? [])];
    if (selection.folder?.files) files.push(...selection.folder.files.filter((file) => skipCode(file) === undefined));
    else if (selection.folder) report.skipped.push({ name: selection.folder.displayName, code: 'FOLDER_UNAVAILABLE' });
    for (const file of files) {
      const name = file.displayName.split('/').at(-1) ?? file.displayName;
      const code = skipCode(file);
      if (code) { report.skipped.push({ name, code }); continue; }
      if (within(input, file.path)) {
        if (idByPath.has(file.path)) report.alreadyQueued += 1;
        else { queueAt(file.path, name); report.added += 1; report.inPlace = (report.inPlace ?? 0) + 1; }
        continue;
      }
      if (settings.destination.trim() && within(settings.destination, file.path)) { report.skipped.push({ name, code: 'IN_OUTPUT_FOLDER' }); continue; }
      if (waitingCopy(file.path)) { report.alreadyQueued += 1; continue; }
      let index = 1;
      while (inputNames.has(fileNameKey(numberedName(name, index)))) index += 1;
      const copy = numberedName(name, index);
      inputNames.add(fileNameKey(copy));
      report.copied = (report.copied ?? 0) + 1;
      if (verifiedInbox) {
        heldCopies.set(`${input}${separator}${copy}`, copy);
        report.held = (report.held ?? 0) + 1;
        report.heldReason = 'AWAITING_MICROSOFT';
        continue;
      }
      copyOf.set(file.path, queueAt(`${input}${separator}${copy}`, copy).id);
      report.added += 1;
    }
    return report;
  };
  return {
    listItems: async () => items.map(listed),
    copyToInput: async (selection) => copyToInput(selection),
    // What folder setup found in the input folder (`existingDocuments`), queued where it is.
    queueInputFolder: async () => {
      const input = inputFolder();
      const count = options.existingDocuments ?? 0;
      for (let index = 1; index <= count; index += 1) {
        const name = `Document ${index}.pdf`;
        inputNames.add(fileNameKey(name));
        queueAt(`${input}\\${name}`, name);
      }
      // Added on purpose, what Clear queue set aside comes in again.
      const again = [...setAside].filter(([path]) => within(input, path));
      for (const [path, name] of again) {
        setAside.delete(path);
        queueAt(path, name);
      }
      const added = count + again.length;
      const held = [...heldCopies.keys()].filter((path) => within(input, path)).length;
      return held > 0
        ? { added, alreadyQueued: 0, skipped: [], held, heldReason: 'AWAITING_MICROSOFT' }
        : { added, alreadyQueued: 0, skipped: [] };
    },
    pauseQueue: async () => { paused = true; items = items.map((item) => item.status === 'processing' ? { ...item, status: 'waiting' as const, stage: undefined, progress: undefined } : item); },
    resumeQueue: async () => { paused = false; fileStagedCopies(); const item = items.find((entry) => entry.status === 'waiting'); if (item) update(item.id, { status: 'processing', stage: 'reading' }); },
    cancel: async (id) => update(id, { status: 'failed', stage: undefined, progress: undefined, reason: 'Canceled.' }),
    // Mirrors Pipeline::approve, in its order: the name itself, its date,
    // then whether this item can take it - its files settled, the source's
    // extension kept, and a proposal in review or ready.
    approve: async (id, filename, description) => {
      const name = filename.trim();
      if (validateLeafFilename(name) !== undefined) throw { code: 'NAME_INVALID', message: 'filename must be one nonblank path component' };
      if (!leadingDate(name)) throw { code: 'DATE_REQUIRED', message: 'the filename must start with the document\'s date as YYYY-MM-DD' };
      let item = find(id);
      // Approving is how a person asks for a stopped rename to be settled, so
      // a parked item's files are checked first. The check can find the
      // document filed already: approving what that rename was filing is
      // then done, and anything else is refused rather than reported as
      // applied (Pipeline::already_filed).
      if (isParked(item)) {
        item = checkAgain(item);
        if (item.status === 'completed') {
          if ((name === item.proposedFilename || name === item.filedName) && description.trim() === (item.description ?? '')) return;
          throw { code: 'ALREADY_FILED', message: `The earlier rename had already finished, so the document is filed as ${item.filedName ?? item.originalFilename} and your changes were not applied. Undo it to rename it again.` };
        }
      }
      if (isParked(item)) throw parkedRefusal();
      if (validateFilename(name, filenameExtension(item.originalFilename)) !== undefined) throw { code: 'NAME_INVALID', message: 'approved filename must preserve the source extension' };
      if (item.status !== 'review' && item.status !== 'ready') throw { code: 'INVALID_TRANSITION', message: 'proposal is not reviewable' };
      // A name a person approved is theirs: Settings' organization names no
      // longer recompose it. Nor is it judged by the seeded reading again:
      // undone, it waits for the undo, at either level.
      composedNames.delete(id);
      readings.delete(id);
      // Pipeline::complete_already_named: filed in its own folder (no
      // destination, flat), a name that is already the document's has
      // nowhere to go. It completes as kept, with nothing to undo, and says
      // why nothing moved - before any wait for a busy queue, as there.
      if (!settings.destination.trim() && settings.destinationLayout === 'flat' && fileNameKey(name) === fileNameKey(item.originalFilename)) {
        replace({ ...approvedFields(item), status: 'completed', proposedFilename: name, description: description.trim(), undoable: false, keptOriginal: true, reason: humanizeReason('ALREADY_NAMED') });
        return;
      }
      // APPLY_DEFERRED: the approval is kept and the command succeeds, but
      // the document is not renamed yet.
      if (options.deferApprovalsWhileBusy && items.some((entry) => entry.id !== id && entry.status === 'processing')) {
        replace({ ...approvedFields(item), status: 'ready', proposedFilename: name, description: description.trim(), approved: true, proposalRevision: `${Number(item.proposalRevision ?? 0) + 1}` });
        return;
      }
      // Approval clears the proposal's review reasons, and the filing is
      // recorded under the name it was given.
      replace({ ...approvedFields(item), status: 'completed', proposedFilename: name, filedName: name, description: description.trim(), undoable: true, keptOriginal: false });
      noteRecorded();
    },
    // complete_keep_original changes the status and nothing else: the
    // proposal stays, unapplied, and with no rename there is nothing to undo.
    // A document Intern could not read gets its own name back first: the
    // NEEDS REVIEW prefix Intern gave the file comes off again.
    keepOriginal: async (id) => {
      const item = find(id);
      if (isParked(item)) throw parkedRefusal();
      if (item.status !== 'review' && item.status !== 'ready') throw { code: 'INVALID_TRANSITION', message: 'only ready or review items can keep their original name' };
      const originalFilename = marks.get(id) ?? item.originalFilename;
      marks.delete(id);
      update(id, { status: 'completed', keptOriginal: true, undoable: false, originalFilename });
    },
    // Pipeline::retry: failed and canceled items, and only the review items
    // whose code says what a retry would do. A parked item's retry is
    // "Check again", which here finds the files in order.
    retry: async (id) => {
      const item = find(id);
      if (!retryAccepted(item)) throw { code: 'INVALID_TRANSITION', message: 'only failed or canceled items can be retried' };
      if (isParked(item)) {
        checkAgain(item);
        return;
      }
      copyIds.delete(id);
      replace({ id: item.id, originalFilename: item.originalFilename, status: 'waiting' });
    },
    // Re-analysis forgets the proposal and reads the document again. Here the
    // reading takes `analysisDelayMs`, after which the same proposal returns
    // as a new revision for review - without a flag raised before any reading
    // (a duplicate, an unverified uploader), which a new reading never raises.
    reanalyze: async (id) => {
      const item = find(id);
      if (item.status !== 'review' && item.status !== 'ready') throw { code: 'INVALID_TRANSITION', message: 'only a document waiting for review can be analyzed again' };
      // requeue_for_analysis refuses while an operation of the item never
      // finished, with INVALID_TRANSITION rather than unsettled_files' code.
      if (isParked(item)) throw { code: 'INVALID_TRANSITION', message: 'an earlier rename of this document did not finish; check it again first' };
      // A row the Balanced level made ready is read again as the reading it
      // is: it comes back as it waited, and is judged by the level again.
      const reading = waitingAgain(item);
      replace({ id: item.id, originalFilename: item.originalFilename, status: 'waiting' });
      setTimeout(() => {
        const current = items.find((entry) => entry.id === id);
        if (current?.status !== 'waiting') return;
        const read = PRE_ANALYSIS_CODES.has(reading.errorCode ?? '') ? settledFields(reading) : reading;
        const back: QueueItem = { ...read, status: 'review', proposalRevision: `${Number(item.proposalRevision ?? 0) + 1}` };
        replace(reviewLevelOf(settings) === 'balanced' ? readyWithNotes(back) : back);
        queueChanged();
      }, analysisDelayMs);
    },
    // remove_item never deletes an item mid-flight, and a parked one only
    // with the person's word that they resolved its files themselves.
    remove: async (id, removal) => {
      const item = find(id);
      if (item.status === 'processing') throw { code: 'STATE_CONFLICT', message: 'active or missing item cannot be removed' };
      if (isParked(item) && removal?.confirmed !== true) throw parkedRefusal();
      const path = pathById.get(id);
      if (path) idByPath.delete(path);
      pathById.delete(id);
      items = items.filter((entry) => entry.id !== id);
    },
    // Pipeline::undo: only a filed rename, and the document then waits for a
    // decision again rather than being renamed by the next pass.
    undo: async (id) => {
      const item = find(id);
      if (item.status !== 'completed') throw { code: 'INVALID_TRANSITION', message: 'only completed operations can be undone' };
      if (!item.undoable) throw { code: 'STATE_CONFLICT', message: 'completed item has no durable operation receipt' };
      // Back to waiting: a mark `byItself` gave the filing goes with it.
      const { filedName: _filedName, filedPath: _filedPath, keptOriginal: _keptOriginal, renamedWithoutApproval: _renamed, ...rest } = item;
      // A copy back where it came from waits as the duplicate it is, and
      // nothing moves it again on its own; Process anyway reads it.
      if (copyIds.has(id)) {
        replace({ ...rest, status: 'review', undoable: false, errorCode: 'DUPLICATE', reason: humanizeReason('EXACT_COPY, UNDONE') });
        return;
      }
      // A rename made while the reading carried codes - by Intern itself,
      // at the Balanced level - is undone with them still on it, and the
      // undo beside them, as the backend adds it to the record's reasons.
      if (rest.reasonCodes?.length) {
        const codes = [...rest.reasonCodes, 'UNDONE'];
        replace({ ...rest, status: 'review', undoable: false, reasonCodes: codes, reason: humanizeReason(codes.join(', ')) });
        return;
      }
      replace({ ...rest, status: 'review', undoable: false, reason: humanizeReason('UNDONE') });
    },
    // There are no files behind the browser preview to hand to a program,
    // so after the same checks the desktop makes it says so.
    openItem: async (id) => { find(id); throw new Error('Opening documents is only available in the desktop application'); },
    revealItem: async (id) => { find(id); throw new Error('Showing documents in their folder is only available in the desktop application'); },
    // The viewer is shown a stand-in made from the row (demoPages), under the
    // backend's rules: a document mid-rename has no one place to read it from,
    // and only a PDF is drawn - the preview has no picture to show.
    documentFile: async (id) => {
      const item = find(id);
      if (item.status === 'processing' && item.stage === 'filing') throw { code: 'PATH_UNAVAILABLE', message: 'the document is not where Intern last saw it' };
      if (filenameExtension(item.originalFilename)?.toLowerCase() !== 'pdf') throw { code: 'VIEW_UNSUPPORTED', message: 'the window shows this format as the text Intern read' };
      return demoPdf(demoPages(item));
    },
    documentText: async (id) => {
      const item = find(id);
      if (item.status === 'processing' && item.stage === 'filing') throw { code: 'PATH_UNAVAILABLE', message: 'the document is not where Intern last saw it' };
      return { pages: demoPages(item), readNow: false, changed: false };
    },
    getSettings: async () => ({ ...settings }),
    // Mirrors the backend: a hosted model is refused at save time without a
    // stored key or a usable address, never silently kept.
    saveSettings: async (next) => {
      if (next.modelSource === 'hosted') {
        if (hostedKey === undefined) throw { code: 'HOSTED_MODEL_KEY_MISSING', message: 'no API key is stored for the hosted model' };
        if (hostedEndpoint(next) === null) throw { code: 'HOSTED_MODEL_MISCONFIGURED', message: 'the hosted model\'s address or model name is not usable' };
      }
      const pairProblem = folderPairProblem(settings, next);
      if (pairProblem) throw pairProblem;
      const levelBefore = reviewLevelOf(settings);
      settings = next.ourNames === undefined ? { ...next } : { ...next, ourNames: ownNames(next.ourNames) };
      renameWaiting();
      if (reviewLevelOf(settings) !== levelBefore) rejudgeWaiting();
    },
    getSetup: async () => ({ ...setup, hostedModelReady: settings.modelSource === 'hosted' && hostedConfigured(settings), tiers: tiers() }),
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
    startModelDownload: async (tier) => {
      if (tier === 'accurate' && setup.state === 'ready') { startAccurate(); return; }
      if (tier === 'fast' && quick && setup.state === 'ready') { startQuick(); return; }
      if (setup.state === 'downloading') return;
      setup = { ...setup, state: 'downloading', error: undefined };
      downloadTimer = setInterval(() => {
        const downloadedBytes = Math.min(setup.totalBytes, setup.downloadedBytes + downloadStepBytes);
        setup = { ...setup, downloadedBytes };
        if (downloadedBytes >= setup.totalBytes) finishDownload();
      }, downloadIntervalMs);
    },
    setupCancel: async (tier) => {
      if (tier === 'fast' && quickTimer && quick) {
        clearInterval(quickTimer);
        quickTimer = undefined;
        quick = { ...quick, state: 'required' };
        return;
      }
      if (tier === 'accurate' && accurateTimer) {
        clearInterval(accurateTimer);
        accurateTimer = undefined;
        accurate = { ...accurate, state: 'required' };
        return;
      }
      if (downloadTimer) clearInterval(downloadTimer);
      downloadTimer = undefined;
      setup = { ...setup, state: 'required', error: 'MODEL_DOWNLOAD_CANCELED' };
    },
    setupChooseExisting: async (files) => {
      if (setup.state === 'downloading') throw { code: 'SETUP_BUSY', message: 'a model setup operation is already active' };
      setup = { ...setup, state: 'ready', downloadedBytes: setup.totalBytes, error: undefined };
      // Two files are both models; one is the fast one.
      if (files.modelPaths.length > 1) accurate = { ...accurate, state: 'ready', downloadedBytes: accurate.totalBytes, error: undefined };
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
    // Pipeline::clear_queue: everything not filed goes, its file left where
    // it is (and given its own name back from NEEDS REVIEW), but a parked
    // document - whose files need checking - and one being renamed now. What
    // went waits in the input folder until it is added again.
    clearQueue: async () => {
      const report = { cleared: 0, unsettled: 0, busy: 0, stillMarked: 0 };
      const folder = settings.intakeFolder.replace(/[\\/]+$/, '');
      items = items.filter((item) => {
        if (item.status === 'completed') return true;
        if (isParked(item)) { report.unsettled += 1; return true; }
        if (item.status === 'processing' && item.cancelable === false) { report.busy += 1; return true; }
        const name = withoutReviewPrefix(item.originalFilename);
        const path = pathById.get(item.id) ?? `${folder}\\${name}`;
        idByPath.delete(path);
        pathById.delete(item.id);
        setAside.set(path, name);
        report.cleared += 1;
        return false;
      });
      return report;
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
        arriving: 0,
        unreadableDocuments: 0,
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
    createInputFolder: async (root) => `${root.replace(/[\\/]+$/, '')}\\Input`,
    // Mirrors the backend: a drive or share root has no folder beside it.
    createOutputFolder: async (intakeFolder) => {
      const filed = outputBeside(intakeFolder);
      if (filed === undefined) throw { code: 'FILED_FOLDER_UNAVAILABLE', message: 'a drive\'s top folder has nothing beside it to file into' };
      return filed;
    },
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
      // In use, not checked: the first name it changes still waits for a person.
      learnedRules = learnedRules.map((rule) => rule.id === id ? { ...rule, seen: Math.max(rule.seen, 2), active: true } : rule);
    },
    houseRuleCheck: async (id, shown) => {
      const rule = learnedRules.find((candidate) => candidate.id === id);
      if (!rule) throw { code: 'RULE_NOT_FOUND', message: 'learned spelling does not exist' };
      // Only the spelling the person was shown, exactly, in use.
      if (!rule.active || rule.kind !== shown.kind || rule.from !== shown.from || rule.to !== shown.to) throw { code: 'RULE_CHANGED', message: 'learned spelling is not in use as it was shown' };
      learnedRules = learnedRules.map((candidate) => candidate.id === id ? { ...candidate, confirmed: candidate.confirmed + 1 } : candidate);
    },
    ...(options.liveEvents ? {
      subscribeQueue: async (listener: (event: QueueBridgeEvent) => void) => {
        queueListeners.add(listener);
        return () => { queueListeners.delete(listener); };
      },
    } : {}),
  };
}

export function createInMemoryBridge(options: InMemoryBridgeOptions = {}): DesktopBridge {
  return createBridge(options, false);
}

export function createFixtureBatchBridge(): DesktopBridge {
  return createBridge({ items: [] }, true);
}

/** Seven scans, six of them undecided, for clearing a review queue from the keyboard. */
export function createReviewBatchBridge(options: InMemoryBridgeOptions = {}): DesktopBridge {
  return createBridge({ items: reviewBatchItems, ...options }, false);
}

/**
 * A statement of work held because two of the learned spellings its name
 * used have not been checked, ahead of the demo's queue, for walking "Use on
 * its own" in Settings and what the review panel says of such a name: the
 * browser build's `?learnedSpellings=held`.
 */
const heldForSpelling: QueueItem = {
  id: 'sow', originalFilename: 'SOW final v3.pdf', status: 'review', confidence: 0.93,
  proposedFilename: '2026-04-01 SOW between Ridgeline Cartography and Contoso.pdf',
  description: 'Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc. for the 2026 member-map engagement.',
  evidence: { date: 'effective as of April 1, 2026', type: 'STATEMENT OF WORK', parties: 'Ridgeline Cartography LLC; Contoso Worldwide, Inc.' },
  errorCode: 'LEARNED_SPELLING_UNCONFIRMED', reasonCodes: ['LEARNED_SPELLING_UNCONFIRMED'], reason: humanizeReason('LEARNED_SPELLING_UNCONFIRMED'),
  houseRules: [
    { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' },
    { kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography' },
    { kind: 'document_type', from: 'Statement of Work', to: 'SOW' },
  ],
};

const heldForSpellingRules: LearnedRule[] = [
  { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1775001600 },
  { id: '2', kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography', seen: 2, active: true, confirmed: 0, learnedAt: 1775001700 },
  { id: '3', kind: 'document_type', from: 'Statement of Work', to: 'SOW', seen: 2, active: true, confirmed: 1, learnedAt: 1775001800 },
];

export function createLearnedSpellingBridge(): DesktopBridge {
  return createBridge({ items: [heldForSpelling, ...seedItems], learnedRules: heldForSpellingRules }, false);
}

/**
 * A scan as an automatic rename at the Balanced review level leaves one:
 * renamed by Intern itself while its reading carried codes, which are still
 * on it. No approval made it - an approval clears them - so the demo, which
 * renames nothing by itself, has to be given one.
 */
const renamedWithNote: QueueItem = {
  id: 'renamed-noted', originalFilename: 'Scan 0098.pdf', status: 'completed', confidence: 0.97,
  proposedFilename: '2024-03-08 Letter to Fabrikam Inc.pdf', filedName: '2024-03-08 Letter to Fabrikam Inc.pdf', filedPath: `${DEMO_OUTPUT_FOLDER}\\2024-03-08 Letter to Fabrikam Inc.pdf`,
  description: 'Letter to Fabrikam Inc. about renewing its service agreement.',
  undoable: true, keptOriginal: false,
  reasonCodes: ['TYPE_INFERRED', 'TYPE_MISSING'], reason: humanizeReason('TYPE_INFERRED, TYPE_MISSING'),
};

const renamedWithNoteReceipt: HistoryEntry = {
  receiptId: '11', queueItemId: 'renamed-noted', at: 1716369300, direction: 'apply', kind: 'rename', stage: 'complete',
  originalPath: `${DEMO_INPUT_FOLDER}\\Scan 0098.pdf`, newPath: `${DEMO_OUTPUT_FOLDER}\\2024-03-08 Letter to Fabrikam Inc.pdf`,
  description: 'Letter to Fabrikam Inc. about renewing its service agreement.',
};

/**
 * The demo's queue with the review level at Balanced, for looking at what
 * that leaves on screen: the lease ready with a note, and in Completed a
 * scan renamed with one. The browser build's `?reviewLevel=balanced`.
 */
export function createBalancedLevelBridge(): DesktopBridge {
  return createBridge({ items: [...seedItems, renamedWithNote], readings: seedReadings, history: [renamedWithNoteReceipt, ...seedHistory], settings: { reviewLevel: 'balanced' } }, false);
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
