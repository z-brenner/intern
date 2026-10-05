import { describe, expect, it, vi } from 'vitest';
import { SUPPORT_LINKS } from './bridge';
import { createInMemoryBridge } from './inMemoryBridge';

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
