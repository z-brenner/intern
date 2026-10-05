import { describe, expect, it, vi } from 'vitest';
import { SUPPORT_LINKS } from './bridge';
import { createInMemoryBridge } from './inMemoryBridge';
import { humanizeReason } from './reasons';
import type { QueueItem } from '../types';

describe('createInMemoryBridge onboarding state', () => {
  it('seeds the completed version and retains completion for later reads', async () => {
    const seeded = createInMemoryBridge({ completedOnboardingVersion: 1 });
    expect(await seeded.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: false });

    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'active' } });
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 0, required: true, sharePointAvailable: true });
    await bridge.completeOnboarding();
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: true });
  });

  it('never requires onboarding while the packaged deployment is unavailable', async () => {
    expect(await createInMemoryBridge().getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: false });
    expect(await createInMemoryBridge({ completedOnboardingVersion: 0 }).getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 0, required: false, sharePointAvailable: false });
  });
});

describe('createInMemoryBridge fake SharePoint deployment', () => {
  it('connects Microsoft after a poll, stays pending for the configured rescans, then activates managed settings', async () => {
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { signInPolls: 2, pendingRescans: 2, account: { displayName: 'Pat Lee', email: 'pat@contoso.test' } } });

    expect(await bridge.microsoftIntakeStatus?.()).toMatchObject({ connected: false, account: null, error: null });
    await expect(bridge.getSharePointSetup()).rejects.toMatchObject({ code: 'MICROSOFT_ACCOUNT_MISSING' });
    expect((await bridge.microsoftSignInStart!()).userCode).toBeTruthy();
    expect(await bridge.microsoftSignInPoll!()).toMatchObject({ state: 'pending' });
    expect(await bridge.microsoftSignInPoll!()).toMatchObject({ state: 'connected' });
    expect(await bridge.microsoftIntakeStatus?.()).toMatchObject({ connected: true, account: { displayName: 'Pat Lee', email: 'pat@contoso.test' } });

    expect(await bridge.getSharePointSetup()).toMatchObject({ phase: 'enrollment_pending', account: { displayName: 'Pat Lee', email: 'pat@contoso.test' }, site: 'InternTestSite', library: 'Files', intake: 'Inbox', destination: 'Filed' });
    expect((await bridge.startSharePointSync()).phase).toBe('enrollment_pending');
    expect((await bridge.getSharePointSetup()).phase).toBe('enrollment_pending');
    expect((await bridge.getSharePointSetup()).phase).toBe('enrollment_pending');
    expect((await bridge.getSharePointSetup()).phase).toBe('ready_to_activate');

    expect((await bridge.activateOnboarding()).phase).toBe('active');
    expect(await bridge.getSettings()).toMatchObject({ intakeEnabled: true, processOthersUploads: false, intakeLocalOnly: false, runInBackground: true, startAtLogin: true, startMinimized: true });
    expect((await bridge.getSettings()).intakeFolder).toMatch(/Inbox$/);
    expect((await bridge.getSettings()).destination).toMatch(/Filed$/);
  });

  it('reports an injected record problem while the library is pending, and none once it appears', async () => {
    const problem = { code: 'SHAREPOINT_ROOT_RECORD_CONFLICT', message: "OneDrive's sync records disagree (library records)." };
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, pendingRescans: 1, pendingProblem: problem } });

    expect(await bridge.getSharePointSetup()).toMatchObject({ phase: 'enrollment_pending', problem });
    expect(await bridge.startSharePointSync()).toMatchObject({ phase: 'enrollment_pending', problem });
    expect(await bridge.getSharePointSetup()).toMatchObject({ phase: 'enrollment_pending', problem });
    expect(await bridge.getSharePointSetup()).toMatchObject({ phase: 'ready_to_activate', problem: null });
  });

  it('reports no problem by default', async () => {
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true } });
    expect((await bridge.getSharePointSetup()).problem).toBeNull();
  });

  it('throws injected failures once each, in order', async () => {
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'active', failures: { completeOnboarding: [{ code: 'ONBOARDING_STATE_WRITE_FAILED', message: 'disk full' }] } } });

    await expect(bridge.completeOnboarding()).rejects.toMatchObject({ code: 'ONBOARDING_STATE_WRITE_FAILED' });
    await expect(bridge.completeOnboarding()).resolves.toBeUndefined();
  });
});

describe('createInMemoryBridge fake SharePoint contract', () => {
  it('refuses completion until filing is on, as the backend does', async () => {
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'ready_to_activate' } });

    await expect(bridge.completeOnboarding()).rejects.toMatchObject({ code: 'ONBOARDING_SETUP_INCOMPLETE' });
    expect((await bridge.getOnboarding()).required).toBe(true);
    await bridge.activateOnboarding();
    await expect(bridge.completeOnboarding()).resolves.toBeUndefined();
    expect((await bridge.getOnboarding()).required).toBe(false);
  });

  it('needs filing turned on again after signing in as a different account', async () => {
    const bridge = createInMemoryBridge({ sharePoint: 'fake', sharePointFake: { connected: true, phase: 'active', signInAccount: { displayName: 'Sam Roe', email: 'sam@contoso.test' } } });

    await bridge.microsoftSignInStart!();
    expect(await bridge.microsoftSignInPoll!()).toMatchObject({ state: 'connected', account: { displayName: 'Sam Roe' } });
    expect(await bridge.getSharePointSetup()).toMatchObject({ phase: 'ready_to_activate', account: { displayName: 'Sam Roe', email: 'sam@contoso.test' } });
    expect((await bridge.activateOnboarding()).phase).toBe('active');
    await bridge.microsoftSignInStart!();
    await bridge.microsoftSignInPoll!();
    expect((await bridge.getSharePointSetup()).phase).toBe('active');
  });
});

describe('createInMemoryBridge managed Microsoft boundary', () => {
  it('exposes no-argument methods that preserve the packaged deployment failure', async () => {
    const bridge = createInMemoryBridge();
    const unavailable = 'SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build.';

    await expect(bridge.microsoftIntakeStatus?.()).resolves.toMatchObject({
      connected: false,
      account: null,
      binding: null,
      error: unavailable,
    });
    await expect(bridge.microsoftSignInStart?.()).rejects.toThrow(unavailable);
    await expect(bridge.microsoftBindIntake?.()).rejects.toThrow(unavailable);
  });
});

describe('createInMemoryBridge support links', () => {
  it('opens the fixed support links in a new tab, the way the guide opens', async () => {
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    const bridge = createInMemoryBridge();

    await bridge.openSupportLink('sharepoint-site');
    await bridge.openSupportLink('onedrive-download');

    expect(open.mock.calls).toEqual([
      [SUPPORT_LINKS['sharepoint-site'], '_blank', 'noopener,noreferrer'],
      [SUPPORT_LINKS['onedrive-download'], '_blank', 'noopener,noreferrer'],
    ]);
    open.mockRestore();
  });
});

// The browser build and every RTL test run against this bridge, so a rule it
// does not mirror is a refusal no test can see: Retry on an ordinary review
// item looked fine here and failed on the desktop.
describe('createInMemoryBridge review rules mirror the backend', () => {
  const review = (id: string, extra: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status: 'review', proposedFilename: `2024-05-01 ${id}.pdf`, ...extra });

  it('retries failed items and only the review codes Pipeline::retry handles', async () => {
    const bridge = createInMemoryBridge({ items: [
      { id: 'failed', originalFilename: 'failed.pdf', status: 'failed', reason: 'Extraction failed.' },
      review('plain', { errorCode: 'LOW_CONFIDENCE' }),
      review('duplicate', { errorCode: 'DUPLICATE', proposedFilename: undefined }),
      review('unverified', { errorCode: 'UPLOADER_UNVERIFIED', proposedFilename: undefined }),
      review('copied', { errorCode: 'SOURCE_DELETE_FAILED' }),
      review('stuck', { errorCode: 'FILE_CHANGED', parked: true }),
      { id: 'ready', originalFilename: 'ready.pdf', status: 'ready', proposedFilename: '2024-05-01 Ready.pdf' },
    ] });

    await expect(bridge.retry('plain')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    await expect(bridge.retry('ready')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    await bridge.retry('failed');
    await bridge.retry('duplicate');
    await bridge.retry('unverified');
    await bridge.retry('copied');
    await bridge.retry('stuck');

    const items = await bridge.listItems();
    const byId = (id: string) => items.find((item) => item.id === id);
    expect(byId('failed')).toEqual({ id: 'failed', originalFilename: 'failed.pdf', status: 'waiting' });
    expect(byId('duplicate')?.status).toBe('waiting');
    expect(byId('unverified')?.status).toBe('waiting');
    // Checking again finishes a filing whose renamed copy was safe all along.
    expect(byId('copied')).toMatchObject({ status: 'completed', filedName: '2024-05-01 copied.pdf', undoable: true });
    expect(byId('stuck')).toMatchObject({ status: 'review', parked: false });
    expect(byId('stuck')).not.toHaveProperty('errorCode');
  });

  it('keeps only ready and review items, leaves the proposal, and records that nothing was renamed', async () => {
    const bridge = createInMemoryBridge({ items: [
      review('lease'),
      { id: 'waiting', originalFilename: 'waiting.pdf', status: 'waiting' },
      review('stuck', { errorCode: 'RECONCILIATION_REQUIRED' }),
    ] });

    await expect(bridge.keepOriginal('waiting')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    await expect(bridge.keepOriginal('stuck')).rejects.toMatchObject({ code: 'RECONCILIATION_REQUIRED' });
    await bridge.keepOriginal('lease');

    expect((await bridge.listItems())[0]).toMatchObject({ status: 'completed', proposedFilename: '2024-05-01 lease.pdf', keptOriginal: true, undoable: false });
  });

  it('removes anything not mid-flight, and a parked item only once the person confirms', async () => {
    const bridge = createInMemoryBridge({ items: [
      { id: 'active', originalFilename: 'active.pdf', status: 'processing', stage: 'reading' },
      { id: 'waiting', originalFilename: 'waiting.pdf', status: 'waiting' },
      review('stuck', { parked: true }),
    ] });

    await expect(bridge.remove('active')).rejects.toMatchObject({ code: 'STATE_CONFLICT' });
    await expect(bridge.remove('stuck')).rejects.toMatchObject({ code: 'RECONCILIATION_REQUIRED' });
    await expect(bridge.remove('stuck', { confirmed: false })).rejects.toMatchObject({ code: 'RECONCILIATION_REQUIRED' });
    await expect(bridge.remove('missing')).rejects.toMatchObject({ code: 'ITEM_NOT_FOUND' });
    await bridge.remove('waiting');
    await bridge.remove('stuck', { confirmed: true });

    expect((await bridge.listItems()).map((item) => item.id)).toEqual(['active']);
  });

  it('analyzes a ready or review item again: waiting first, then back in review as a new revision', async () => {
    vi.useFakeTimers();
    try {
      const bridge = createInMemoryBridge({ items: [
        review('lease', { proposalRevision: '3', reason: 'The model reported low confidence in its own proposal.' }),
        review('stuck', { parked: true }),
        { id: 'done', originalFilename: 'done.pdf', status: 'completed', undoable: true },
      ], analysisDelayMs: 100, liveEvents: true });
      const changes: string[] = [];
      const source = bridge as typeof bridge & { subscribeQueue(listener: (event: { type: string }) => void): Promise<() => void> };
      await source.subscribeQueue((event) => changes.push(event.type));

      // requeue_for_analysis's own refusal: the files come first.
      await expect(bridge.reanalyze('stuck')).rejects.toMatchObject({ code: 'INVALID_TRANSITION', message: expect.stringMatching(/check it again first/) });
      await expect(bridge.reanalyze('done')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
      await bridge.reanalyze('lease');
      expect((await bridge.listItems())[0]).toEqual({ id: 'lease', originalFilename: 'lease.pdf', status: 'waiting' });

      await vi.advanceTimersByTimeAsync(100);

      expect((await bridge.listItems())[0]).toMatchObject({ status: 'review', proposedFilename: '2024-05-01 lease.pdf', proposalRevision: '4' });
      expect(changes).toEqual(['changed']);
    } finally {
      vi.useRealTimers();
    }
  });

  // A duplicate is flagged as it arrives, never by a reading, so reading it
  // again is processing it anyway.
  it('analyzes a duplicate again as a document like any other', async () => {
    vi.useFakeTimers();
    try {
      const bridge = createInMemoryBridge({ items: [
        review('copy', { proposedFilename: undefined, errorCode: 'DUPLICATE', reason: 'Duplicate of 2024-05-01 lease.pdf' }),
      ], analysisDelayMs: 100 });

      await bridge.reanalyze('copy');
      await vi.advanceTimersByTimeAsync(100);

      const [copy] = await bridge.listItems();
      expect(copy).toMatchObject({ status: 'review', proposalRevision: '1' });
      expect(copy).not.toHaveProperty('errorCode');
      expect(copy).not.toHaveProperty('reason');
    } finally {
      vi.useRealTimers();
    }
  });

  // Pipeline::approve checks a parked item's files before anything else:
  // approving again is how a person asks for a stopped rename to be settled.
  it('checks a parked item\'s files before approving it', async () => {
    const bridge = createInMemoryBridge({ items: [
      review('stuck', { errorCode: 'FILE_CHANGED', parked: true }),
      review('copied', { errorCode: 'SOURCE_DELETE_FAILED', parked: true, description: 'A lease.' }),
      review('renamed', { errorCode: 'SOURCE_DELETE_FAILED', parked: true, description: 'A lease.' }),
    ] });

    // A rename that never happened: checked, then approved as asked.
    await bridge.approve('stuck', '2024-05-01 Stuck renamed.pdf', '');
    // A rename that had finished: approving what it filed is done...
    await bridge.approve('copied', '2024-05-01 copied.pdf', 'A lease.');
    // ...and a different name is refused, not reported as applied.
    await expect(bridge.approve('renamed', '2024-05-01 Something else.pdf', 'A lease.')).rejects.toMatchObject({ code: 'ALREADY_FILED' });

    const items = await bridge.listItems();
    const byId = (id: string) => items.find((item) => item.id === id);
    expect(byId('stuck')).toMatchObject({ status: 'completed', filedName: '2024-05-01 Stuck renamed.pdf' });
    expect(byId('copied')).toMatchObject({ status: 'completed', filedName: '2024-05-01 copied.pdf', undoable: true });
    expect(byId('renamed')).toMatchObject({ status: 'completed', filedName: '2024-05-01 renamed.pdf' });
  });

  it('pushes no queue events unless asked to', () => {
    expect('subscribeQueue' in createInMemoryBridge()).toBe(false);
  });

  it('approves only a writable name that keeps the extension, and records the name it was filed under', async () => {
    const bridge = createInMemoryBridge({ items: [review('lease'), { id: 'done', originalFilename: 'done.pdf', status: 'completed' }] });

    await expect(bridge.approve('lease', '2024-05-01 Lease: Acme.pdf', '')).rejects.toMatchObject({ code: 'NAME_INVALID' });
    await expect(bridge.approve('lease', 'Lease.pdf', '')).rejects.toMatchObject({ code: 'DATE_REQUIRED' });
    await expect(bridge.approve('lease', '2024-05-01 Lease.docx', '')).rejects.toMatchObject({ code: 'NAME_INVALID', message: 'approved filename must preserve the source extension' });
    await expect(bridge.approve('done', '2024-05-01 Done.pdf', '')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    await bridge.approve('lease', '  2024-05-01 Lease.PDF ', ' A lease. ');

    expect((await bridge.listItems())[0]).toMatchObject({ status: 'completed', proposedFilename: '2024-05-01 Lease.PDF', filedName: '2024-05-01 Lease.PDF', description: 'A lease.', undoable: true });
  });

  // Pipeline::complete_already_named: filed where it is, a name that is the
  // document's own (as Windows compares names) completes it without a rename,
  // with nothing to undo, and says why.
  it('completes an approval of the name a document already has as kept, not renamed', async () => {
    const named = (id: string): QueueItem => ({ id, originalFilename: '2024-05-01 Lease.pdf', status: 'ready', proposedFilename: '2024-05-01 Lease.pdf' });
    const bridge = createInMemoryBridge({ items: [named('same'), named('case')] });

    await bridge.approve('same', '2024-05-01 Lease.pdf', '');
    await bridge.approve('case', '2024-05-01 lease.PDF', '');

    for (const item of await bridge.listItems()) {
      expect(item, item.id).toMatchObject({ status: 'completed', keptOriginal: true, undoable: false, reason: humanizeReason('ALREADY_NAMED') });
      expect(item, item.id).not.toHaveProperty('filedName');
    }
    // Filed into a destination folder, the same name is a rename into it.
    const filing = createInMemoryBridge({ items: [named('same')], settings: { destination: 'C:\\Filed' } });
    await filing.approve('same', '2024-05-01 Lease.pdf', '');
    expect((await filing.listItems())[0]).toMatchObject({ status: 'completed', keptOriginal: false, undoable: true, filedName: '2024-05-01 Lease.pdf' });
  });

  it('can keep an approval for later while another document is processing, as begin_applying does', async () => {
    const items = () => [review('lease', { reason: 'Low confidence.', errorCode: 'LOW_CONFIDENCE' }), review('memo'), { id: 'busy', originalFilename: 'busy.pdf', status: 'processing' as const, stage: 'reading' as const }];
    const deferring = createInMemoryBridge({ items: items(), deferApprovalsWhileBusy: true });

    await deferring.approve('lease', '2024-05-01 Lease.pdf', ' A lease. ');

    const [lease] = await deferring.listItems();
    expect(lease).toMatchObject({ status: 'ready', approved: true, proposedFilename: '2024-05-01 Lease.pdf', description: 'A lease.' });
    expect(lease).not.toHaveProperty('reason');
    expect(lease).not.toHaveProperty('errorCode');
    // Off unless asked for: the demo's processing document never finishes.
    const demo = createInMemoryBridge({ items: items() });
    await demo.approve('memo', '2024-05-02 Memo.pdf', '');
    expect((await demo.listItems())[1]).toMatchObject({ status: 'completed' });
  });

  it('undoes only a filed rename, sending the document back to review', async () => {
    const bridge = createInMemoryBridge({ items: [
      { id: 'filed', originalFilename: 'filed.pdf', status: 'completed', proposedFilename: '2024-05-01 Filed.pdf', filedName: '2024-05-01 Filed.pdf', undoable: true },
      { id: 'kept', originalFilename: 'kept.pdf', status: 'completed', keptOriginal: true, undoable: false },
      review('lease'),
    ] });

    await expect(bridge.undo('kept')).rejects.toMatchObject({ code: 'STATE_CONFLICT' });
    await expect(bridge.undo('lease')).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });
    await bridge.undo('filed');

    const undone = (await bridge.listItems())[0];
    expect(undone).toMatchObject({ status: 'review', undoable: false, reason: expect.stringMatching(/You undid this rename/) });
    expect(undone).not.toHaveProperty('filedName');
  });
});

describe('createInMemoryBridge organisation names', () => {
  const lease = async (bridge: ReturnType<typeof createInMemoryBridge>) => (await bridge.listItems()).find((item) => item.id === 'lease')!;

  it('starts with no names and stores them the way the backend does', async () => {
    const bridge = createInMemoryBridge();
    expect((await bridge.getSettings()).ourNames).toEqual([]);

    await bridge.saveSettings({ ...await bridge.getSettings(), ourNames: [' TenantCo Inc. ', '', '  '] });
    expect((await bridge.getSettings()).ourNames).toEqual(['TenantCo Inc.']);
  });

  it('names a waiting seeded proposal by the other side, and by both again when the names go', async () => {
    const bridge = createInMemoryBridge();
    expect(await lease(bridge)).not.toHaveProperty('omittedParties');

    // Matched as the backend matches: case and punctuation disregarded.
    await bridge.saveSettings({ ...await bridge.getSettings(), ourNames: ['TENANTCO INC'] });
    // A new proposal revision, as the backend records one, so an open
    // inspector shows the new name rather than keeping the old as a draft.
    expect(await lease(bridge)).toMatchObject({ proposedFilename: '2023-09-15 Lease Agreement with ABC Properties LLC.pdf', omittedParties: ['TenantCo Inc.'], proposalRevision: '2' });
    // A save that changes nothing about the name is not a new revision.
    await bridge.saveSettings({ ...await bridge.getSettings(), ourNames: ['TenantCo Inc.'] });
    expect((await lease(bridge)).proposalRevision).toBe('2');

    await bridge.saveSettings({ ...await bridge.getSettings(), ourNames: [] });
    const restored = await lease(bridge);
    expect(restored.proposedFilename).toBe('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf');
    expect(restored).not.toHaveProperty('omittedParties');
    expect(restored.proposalRevision).toBe('3');
  });

  it('applies names already saved, and never to a name a person approved or to rows a test supplied', async () => {
    const named = createInMemoryBridge({ settings: { ourNames: ['ABC Properties'] } });
    expect(await lease(named)).toMatchObject({ proposedFilename: '2023-09-15 Lease Agreement with TenantCo Inc.pdf', omittedParties: ['ABC Properties LLC'] });

    const bridge = createInMemoryBridge();
    await bridge.approve('lease', '2023-09-15 Lease for 123 Main St.pdf', 'Lease.');
    await bridge.undo('lease');
    await bridge.saveSettings({ ...await bridge.getSettings(), ourNames: ['TenantCo Inc.'] });
    expect((await lease(bridge)).proposedFilename).toBe('2023-09-15 Lease for 123 Main St.pdf');

    const supplied = createInMemoryBridge({ items: [{ id: 'lease', originalFilename: 'lease.pdf', status: 'review', proposedFilename: '2023-09-15 Lease.pdf' }], settings: { ourNames: ['TenantCo Inc.'] } });
    expect(await lease(supplied)).toEqual({ id: 'lease', originalFilename: 'lease.pdf', status: 'review', proposedFilename: '2023-09-15 Lease.pdf' });
  });

  it('never matches on an own name too short to mean one organisation', async () => {
    const bridge = createInMemoryBridge({ settings: { ourNames: ['ABC'] } });
    expect(await lease(bridge)).not.toHaveProperty('omittedParties');
  });
});

describe('createInMemoryBridge update switch', () => {
  // The desktop backend reads a file that never mentions the switch as checks
  // on, and the browser bridge has to agree with it.
  it('starts with automatic checks on and round-trips the choice', async () => {
    const bridge = createInMemoryBridge();
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(false);

    await bridge.saveSettings({ ...await bridge.getSettings(), skipUpdateChecks: true });
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(true);

    await bridge.saveSettings({ ...await bridge.getSettings(), skipUpdateChecks: false });
    expect((await bridge.getSettings()).skipUpdateChecks).toBe(false);
    expect((await createInMemoryBridge({ settings: { skipUpdateChecks: true } }).getSettings()).skipUpdateChecks).toBe(true);
  });
});
