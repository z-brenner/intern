/**
 * The date a filename begins with, as Intern writes it: `YYYY-MM-DD`, a real
 * calendar day, standing on its own before whatever follows. Every rename
 * must carry one - the backend refuses one that does not (DATE_REQUIRED) -
 * so the inspector checks here first and says so before the round trip.
 */
const LEADING_DATE = /^(\d{4})-(\d{2})-(\d{2})(?![0-9A-Za-z])/;

export function leadingDate(filename: string): string | undefined {
  const match = LEADING_DATE.exec(filename.trim());
  if (!match) return undefined;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));
  const real = date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
  return real ? `${match[1]}-${match[2]}-${match[3]}` : undefined;
}

/** `filename` with `date` leading it, replacing any date already there. */
export function withLeadingDate(filename: string, date: string): string {
  const rest = filename.trim().replace(/^\d{4}-\d{2}-\d{2}(?![0-9A-Za-z])[\s_-]*/, '');
  return rest ? `${date} ${rest}` : date;
}

/**
 * The extension a filename ends in, read the way the backend reads one
 * (Rust's `Path::extension`): the text after the last dot, `""` for a name
 * ending in a dot, and nothing for a name without one or a dotfile such as
 * ".pdf" on its own.
 */
export function filenameExtension(name: string): string | undefined {
  const dot = name.lastIndexOf('.');
  if (dot <= 0 || name === '..') return undefined;
  return name.slice(dot + 1);
}

/** ASCII-only case folding, as `eq_ignore_ascii_case` compares extensions. */
const asciiLower = (value: string) => value.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

/** The Windows-reserved characters, each named in the sentence that refuses it. */
const RESERVED = new Set(['<', '>', ':', '"', '|', '?', '*']);
const SEPARATORS = new Set(['/', '\\']);
/** Invisible marks that reorder how a name is displayed, so a name ending ".exe" can read as one ending ".pdf". */
const isBidiControl = (code: number) => code === 0x061c || (code >= 0x200e && code <= 0x200f) || (code >= 0x202a && code <= 0x202e) || (code >= 0x2066 && code <= 0x2069);
/** Unicode general category Cc, which is what Rust's `char::is_control` means. */
const isControl = (code: number) => code <= 0x1f || (code >= 0x7f && code <= 0x9f);
const codePoint = (code: number) => `U+${code.toString(16).toUpperCase().padStart(4, '0')}`;

/** The longest name, in UTF-8 bytes, the backend will write. */
export const MAX_FILENAME_BYTES = 512;

/**
 * Why `name` cannot be written as a filename, as a sentence a person can act
 * on, or undefined when it can. Mirrors the backend's `validate_leaf_filename`
 * and its extension rule (pipeline.rs), so a name the backend would refuse is
 * caught here before the round trip - and refused with the character that is
 * the problem named, not "filename must be one nonblank path component".
 *
 * The name is checked as it will be written. The backend trims surrounding
 * whitespace before it checks, and so does every caller here.
 */
export function validateFilename(name: string, sourceExtension: string | undefined): string | undefined {
  const problem = validateLeafFilename(name);
  if (problem) return problem;
  if (!sourceExtension) return 'This file has no extension, so Intern cannot rename it.';
  const extension = filenameExtension(name);
  if (extension === undefined || asciiLower(extension) !== asciiLower(sourceExtension)) {
    return `The filename must end with “.${sourceExtension}”, the file's own extension.`;
  }
  return undefined;
}

/** `validate_leaf_filename` on its own: one writable path component, whatever its extension. */
export function validateLeafFilename(name: string): string | undefined {
  if (!name) return 'Filename is required';
  for (const character of name) {
    const code = character.codePointAt(0)!;
    if (SEPARATORS.has(character)) return `“${character}” cannot be used in a filename; it separates folders.`;
    if (RESERVED.has(character)) return `“${character}” cannot be used in a Windows filename.`;
    if (character === '\t') return 'A tab cannot be used in a filename.';
    if (character === '\n' || character === '\r') return 'A line break cannot be used in a filename.';
    if (isControl(code)) return `The control character ${codePoint(code)} cannot be used in a filename.`;
    if (isBidiControl(code)) return `An invisible text-direction mark (${codePoint(code)}) cannot be used in a filename.`;
  }
  if (name === '.' || name === '..') return `“${name}” is not a filename.`;
  if (name.endsWith('.')) return 'A filename cannot end with a dot; Windows would drop it.';
  if (/\s$/u.test(name)) return 'A filename cannot end with a space; Windows would drop it.';
  const bytes = new TextEncoder().encode(name).length;
  // Counted in bytes, as the backend counts; "about" because a character
  // outside ASCII takes more than one.
  const over = bytes - MAX_FILENAME_BYTES;
  if (over > 0) return `This filename is too long. Shorten it by about ${over} ${over === 1 ? 'character' : 'characters'}.`;
  return undefined;
}

/**
 * A proposed name split for editing: the part a person may change, and the
 * extension they may not. The extension is the source file's - the backend
 * refuses any other - spelled as the proposal spells it when the two agree.
 */
export function splitFilename(proposed: string, sourceExtension: string | undefined): { stem: string; extension?: string } {
  if (!sourceExtension) return { stem: proposed };
  const extension = filenameExtension(proposed);
  if (extension !== undefined && asciiLower(extension) === asciiLower(sourceExtension)) {
    return { stem: proposed.slice(0, proposed.length - extension.length - 1), extension };
  }
  return { stem: proposed, extension: sourceExtension };
}

/** The name an edited stem and its locked extension make, with the stem's stray surrounding spaces dropped. */
export function joinFilename(stem: string, extension: string | undefined): string {
  const trimmed = stem.trim();
  return extension === undefined ? trimmed : `${trimmed}.${extension}`;
}
