/**
 * What Intern reads, said once for every place that says it.
 *
 * The backend admits a file by intern-core's `SUPPORTED_EXTENSIONS`, and the
 * worker routes exactly that list. `FILE_KINDS` mirrors it here, and a test
 * holds the two to the same set: they drifted once, and the drop zone went on
 * telling people that Outlook messages and PowerPoint decks were not read
 * long after they were.
 */
export type FileKind = 'pdf' | 'document' | 'spreadsheet' | 'presentation' | 'email' | 'text' | 'image';

/** Every admitted extension, lowercase and without the dot, by its kind. */
export const FILE_KINDS: Readonly<Record<string, FileKind>> = {
  pdf: 'pdf',
  docx: 'document',
  docm: 'document',
  doc: 'document',
  odt: 'document',
  rtf: 'text',
  xlsx: 'spreadsheet',
  xlsm: 'spreadsheet',
  xls: 'spreadsheet',
  ods: 'spreadsheet',
  csv: 'text',
  pptx: 'presentation',
  pptm: 'presentation',
  ppsx: 'presentation',
  ppt: 'presentation',
  odp: 'presentation',
  eml: 'email',
  msg: 'email',
  txt: 'text',
  md: 'text',
  markdown: 'text',
  png: 'image',
  jpg: 'image',
  jpeg: 'image',
  tif: 'image',
  tiff: 'image',
};

/**
 * Every admitted extension, for the file picker's Documents filter and for
 * what the in-memory bridge skips, as the backend does.
 */
export const SUPPORTED_EXTENSIONS: readonly string[] = Object.freeze(Object.keys(FILE_KINDS));

/** The formats as a person names them, for the drop zone. */
export const SUPPORTED_FORMATS_LABEL = 'PDF, Word (.docx, .doc, .rtf, .odt), Excel (.xlsx, .xls, .ods, .csv), PowerPoint (.pptx, .ppt, .odp), Outlook .msg and .eml email, text, Markdown, and scanned images (PNG, JPEG, TIFF)';

/** The kind of a file by its extension, or `undefined` for one Intern does not read. */
export function fileKind(filename: string): FileKind | undefined {
  const dot = filename.lastIndexOf('.');
  if (dot < 0) return undefined;
  const extension = filename.slice(dot + 1).toLowerCase();
  return Object.hasOwn(FILE_KINDS, extension) ? FILE_KINDS[extension] : undefined;
}
