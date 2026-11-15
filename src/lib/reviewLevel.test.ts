import { describe, expect, it } from 'vitest';
import { reviewLevelOf, withKnownReviewLevel, withReviewLevel } from './reviewLevel';
import type { AppSettings } from '../types';

const settings: AppSettings = { destination: 'D:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'D:\\In', intakeEnabled: true, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' };
/** Settings as a file holds them, which can say anything. */
const stored = (reviewLevel: unknown) => ({ ...settings, reviewLevel } as AppSettings);

describe('the review level settings say', () => {
  it('is strict when they say none, and when they say something this build does not know', () => {
    expect(reviewLevelOf(settings)).toBe('strict');
    expect(reviewLevelOf(stored('strict'))).toBe('strict');
    expect(reviewLevelOf(stored('balanced'))).toBe('balanced');
    for (const unknown of ['lenient', 'Balanced', 'BALANCED', '', ' balanced', null, undefined, 1, true, {}, ['balanced']]) {
      expect(reviewLevelOf(stored(unknown)), JSON.stringify(unknown)).toBe('strict');
    }
  });
});

describe('a level chosen in a draft of the settings', () => {
  it('is set when it differs from the loaded one, and taken back leaves the draft as loaded', () => {
    // Loaded with no level: an older settings file. Chosen and taken back, the draft has none again.
    const chosen = withReviewLevel(settings, 'balanced', settings);
    expect(chosen).toEqual({ ...settings, reviewLevel: 'balanced' });
    const back = withReviewLevel(chosen, 'strict', settings);
    expect(back).toEqual(settings);
    expect('reviewLevel' in back).toBe(false);
    // Choosing what is already loaded is no change at all.
    expect(withReviewLevel(settings, 'strict', settings)).toEqual(settings);

    const balanced = stored('balanced');
    expect(withReviewLevel(balanced, 'strict', balanced)).toEqual({ ...settings, reviewLevel: 'strict' });
    expect(withReviewLevel(withReviewLevel(balanced, 'strict', balanced), 'balanced', balanced)).toEqual(balanced);
  });

  it('keeps every other edit in the draft', () => {
    const draft = { ...settings, automaticRename: true, machineLabel: 'Front desk' };
    expect(withReviewLevel(draft, 'balanced', settings)).toEqual({ ...draft, reviewLevel: 'balanced' });
    expect(withReviewLevel({ ...draft, reviewLevel: 'balanced' }, 'strict', settings)).toEqual(draft);
  });

  it('puts an unknown loaded value back as it was, for the save to write as strict', () => {
    const unknown = stored('lenient');
    const chosen = withReviewLevel(unknown, 'balanced', unknown);
    expect(chosen.reviewLevel).toBe('balanced');
    expect(withReviewLevel(chosen, 'strict', unknown)).toEqual(unknown);
  });
});

describe('the level as it is saved', () => {
  it('is left alone when absent or known, and written as strict when unknown', () => {
    expect(withKnownReviewLevel(settings)).toBe(settings);
    expect('reviewLevel' in withKnownReviewLevel(settings)).toBe(false);
    const balanced = stored('balanced');
    expect(withKnownReviewLevel(balanced)).toBe(balanced);
    const strict = stored('strict');
    expect(withKnownReviewLevel(strict)).toBe(strict);
    for (const unknown of ['lenient', '', null, 3]) {
      expect(withKnownReviewLevel(stored(unknown)), JSON.stringify(unknown)).toEqual({ ...settings, reviewLevel: 'strict' });
    }
  });
});
