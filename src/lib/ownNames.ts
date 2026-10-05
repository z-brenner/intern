/**
 * How the backend compares the organisation's names (own_names.rs), for the
 * places the window has to agree with it.
 */

/** An own name whose key is shorter than this is ignored: it would match far too much. */
export const OWN_NAME_MIN_CHARS = 4;

/** HouseRule::key: case, punctuation and spacing disregarded. */
export function nameKey(value: string): string {
  return value.replace(/[^\p{L}\p{N}\s]/gu, '').toLowerCase().split(/\s+/).filter(Boolean).join(' ');
}

/**
 * The names Intern will not match, as typed (trimmed). A key that short -
 * "Co", "LLP" - would match far too much, so the backend ignores it, and a
 * firm known as "EY" or "PwC" would otherwise save a name that silently does
 * nothing.
 */
export function tooShortOwnNames(names: readonly string[] | undefined): string[] {
  return (names ?? []).map((name) => name.trim()).filter((name) => name.length > 0 && [...nameKey(name)].length < OWN_NAME_MIN_CHARS);
}
