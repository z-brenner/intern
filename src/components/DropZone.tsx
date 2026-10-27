import { FileUp } from 'lucide-react';
import type { DragEvent } from 'react';
import { SUPPORTED_FORMATS_LABEL } from '../lib/formats';
import { Icon } from './Icon';

const FORMATS = `Supports ${SUPPORTED_FORMATS_LABEL}`;

interface Props {
  variant?: 'bar' | 'hero';
  /** The input folder's own name ("Input"); empty until the settings are read. */
  inputFolder?: string;
  /** The output folder's own name ("Output"); empty until the settings are read. */
  outputFolder?: string;
  /** Whether high-confidence documents are renamed without asking. */
  automaticRename?: boolean;
  /** The input folder is the verified team Inbox, which takes a copy only once Microsoft confirms who uploaded it. */
  verifiedInbox?: boolean;
  onDrop(payload: unknown): void;
}

/**
 * The same drop target in two sizes.
 *
 * `bar` is the strip above a queue that already has work in it. `hero` is what
 * a first-time user actually meets: an empty table with four column headings
 * and nothing under them said only that the app had loaded. It is the same
 * region, with the same accessible name and the same drop handler, so the
 * first thing a person sees is also the thing they can act on.
 *
 * A drop copies into the input folder - the one place Intern processes
 * documents from - and both sizes say so by the folder's name, so nobody
 * takes it for a separate way in, or wonders where the original went.
 */
export function DropZone({ variant = 'bar', inputFolder = '', outputFolder = '', automaticRename = false, verifiedInbox = false, onDrop }: Props) {
  const handleDrop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    onDrop(event.dataTransfer);
  };
  const input = inputFolder || 'your input folder';
  const label = `Drag files or folders here to copy them into ${input}`;
  return <div className={`drop-zone drop-zone--${variant}`} aria-label={label} aria-describedby="supported-formats" role="region" onDragOver={(event) => event.preventDefault()} onDrop={handleDrop}>
    {variant === 'hero'
      ? <div className="drop-copy">
        <span className="drop-mark"><Icon icon={FileUp} /></span>
        <h2>Drop documents here</h2>
        <p>Intern copies them into your input folder{inputFolder ? <>, <strong>{inputFolder}</strong></> : ''}, reads each one on this device, and proposes a clear new name. Processed files end up in your output folder{outputFolder ? <>, <strong>{outputFolder}</strong></> : ''}.</p>
        <p>{automaticRename
          ? 'Documents Intern is sure about are renamed on their own; the rest wait for you to approve them.'
          : 'Nothing is renamed until you approve it.'} Your originals stay where they are.</p>
        {verifiedInbox && <p>A copy joins the queue once OneDrive has uploaded it and Microsoft confirms it is yours.</p>}
        <p className="drop-alternative">Or use Add files and Add folder above, or put files straight into {input}.</p>
        <small id="supported-formats">{FORMATS}</small>
      </div>
      : <>
        <Icon icon={FileUp} /><div className="drop-copy"><span>{label}</span><small id="supported-formats">{FORMATS}</small></div>
      </>}
  </div>;
}
