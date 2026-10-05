import type { CloudRoot } from '../../types';

/** The last folder name in a Windows or POSIX path. */
export function folderName(path: string): string {
  const parts = path.split(/[\\/]+/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function components(path: string): string[] {
  return path.replace(/^\\\\\?\\/, '').split(/[\\/]+/).filter(Boolean).map((part) => part.toLowerCase());
}

/** Whether `root` is `path` or one of the folders above it, compared the way Windows does: by whole names, ignoring case. */
export function contains(root: string, path: string): boolean {
  const outer = components(root);
  const inner = components(path);
  return outer.length > 0 && outer.length <= inner.length && outer.every((part, index) => part === inner[index]);
}

/**
 * What a person calls a synced location: a SharePoint library by its own
 * folder name and the organization it belongs to ("Legal - Documents
 * (Contoso SharePoint)"), a OneDrive account by the name OneDrive gives it.
 */
export function rootName(root: CloudRoot): string {
  if (root.provider === 'sharepoint') return `${folderName(root.path)} (${root.displayName} SharePoint)`;
  return root.displayName;
}

/** The synced location holding `path`; the deepest one wins, as on the backend. */
export function rootFor(roots: CloudRoot[], path: string): CloudRoot | undefined {
  return roots
    .filter((root) => root.provider !== 'network_share' && contains(root.path, path))
    .sort((left, right) => components(right.path).length - components(left.path).length)[0];
}

/** A folder as "<synced location> › <folders below it>", or its own path when nothing syncs it. */
export function folderLabel(roots: CloudRoot[], path: string): string {
  const root = rootFor(roots, path);
  if (!root) return path;
  const below = path.split(/[\\/]+/).filter(Boolean).slice(components(root.path).length);
  return [rootName(root), ...below].join(' › ');
}

/**
 * A folder with nothing above it: a drive ("E:\"), a share ("\\server\share",
 * also spelled "\\?\UNC\server\share"), or "/".
 */
const ROOTS = [
  /^(?:\\\\\?\\)?[A-Za-z]:[\\/]?$/,
  /^(?:\\\\\?\\UNC\\|\\\\)[^\\]+\\[^\\]+\\?$/i,
  /^\/$/,
];

/**
 * The "Filed" folder setup offers beside `folder`, spelled with the folder's
 * own separator - or undefined for a root, which has nothing beside it. The
 * backend refuses to make one there, and "E:\" used to come out as "E/Filed".
 */
export function filedBeside(folder: string): string | undefined {
  if (ROOTS.some((root) => root.test(folder))) return undefined;
  const trimmed = folder.replace(/[\\/]+$/, '');
  const cut = Math.max(trimmed.lastIndexOf('\\'), trimmed.lastIndexOf('/'));
  if (cut < 0) return undefined;
  const separator = trimmed.includes('\\') ? '\\' : '/';
  return `${trimmed.slice(0, cut)}${separator}Filed`;
}