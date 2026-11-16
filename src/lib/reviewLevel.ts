import type { AppSettings, ReviewLevel } from '../types';

const LEVELS: readonly ReviewLevel[] = ['strict', 'balanced'];

/**
 * The review level settings say. Strict when they say none - a settings file
 * from before the choice, an older backend - and when they say anything this
 * build does not know: the stricter reading of a value nobody can vouch for.
 */
export function reviewLevelOf(settings: { reviewLevel?: unknown }): ReviewLevel {
  return LEVELS.find((level) => level === settings.reviewLevel) ?? 'strict';
}

/**
 * A draft of the settings with `level` chosen. Choosing the level the loaded
 * settings already say puts their own field back, absent when it was absent:
 * Settings tells an edit from a loaded value by comparing the two, and a
 * level chosen and then taken back is no edit. Saved untouched, settings
 * with no level are saved with none.
 */
export function withReviewLevel(draft: AppSettings, level: ReviewLevel, loaded: AppSettings): AppSettings {
  if (level !== reviewLevelOf(loaded)) return { ...draft, reviewLevel: level };
  const { reviewLevel: _reviewLevel, ...rest } = draft;
  return 'reviewLevel' in loaded ? { ...rest, reviewLevel: loaded.reviewLevel } : rest;
}

/**
 * Settings as they are saved: a level this build does not know is saved as
 * the one Settings showed for it, Strict. Settings with no level, or with one
 * it knows, are saved as they are.
 */
export function withKnownReviewLevel(settings: AppSettings): AppSettings {
  return settings.reviewLevel === undefined || LEVELS.includes(settings.reviewLevel) ? settings : { ...settings, reviewLevel: 'strict' };
}
