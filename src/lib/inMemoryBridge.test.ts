import { describe, expect, it } from 'vitest';
import { createInMemoryBridge } from './inMemoryBridge';

describe('createInMemoryBridge onboarding state', () => {
  it('seeds the completed version and retains completion for later reads', async () => {
    const seeded = createInMemoryBridge({ completedOnboardingVersion: 1 });
    expect(await seeded.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: false });

    const bridge = createInMemoryBridge();
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 0, required: true, sharePointAvailable: false });
    await bridge.completeOnboarding();
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false, sharePointAvailable: false });
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
