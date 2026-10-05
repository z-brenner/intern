import { File, FileImage, FileSpreadsheet, FileText, FileType2, Mail, Presentation } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { type FileKind, fileKind } from '../lib/formats';
import { Icon } from './Icon';

const icons: Record<FileKind, LucideIcon> = {
  pdf: FileText,
  document: FileType2,
  spreadsheet: FileSpreadsheet,
  presentation: Presentation,
  email: Mail,
  text: FileText,
  image: FileImage,
};

export function FileKindIcon({ filename }: { filename: string }) {
  const kind = fileKind(filename);
  const icon = kind ? icons[kind] : File;
  return <span className={`file-kind file-kind--${kind ?? 'other'}`} aria-hidden="true"><Icon icon={icon} /></span>;
}
