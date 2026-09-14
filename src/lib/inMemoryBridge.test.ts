import { describe, expect, it } from 'vitest';
import { createInMemoryBridge } from './inMemoryBridge';

describe('createInMemoryBridge onboarding state', () => {
  it('seeds the completed version and retains completion for later reads', async () => {
    const seeded = createInMemoryBridge({ completedOnboardingVersion: 1 });
    expect(await seeded.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false });

    const bridge = createInMemoryBridge();
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 0, required: true });
    await bridge.completeOnboarding();
    expect(await bridge.getOnboarding()).toEqual({ currentVersion: 1, completedVersion: 1, required: false });
  });
});
