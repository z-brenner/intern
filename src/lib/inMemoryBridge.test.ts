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
    expect(await createInMemoryBridge().getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 0, required: false, sharePointAvailable: false });
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
