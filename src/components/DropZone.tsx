import { FileUp } from 'lucide-react';
import { useState } from 'react';
import type { DragEvent, ReactNode } from 'react';
import { SUPPORTED_FORMATS_LABEL } from '../lib/formats';
import { Icon } from './Icon';

const FORMATS = `Supports ${SUPPORTED_FORMATS_LABEL}`;

interface Props {
  variant?: 'panel' | 'hero';
  /** The input folder's own name ("Input"); empty until the settings are read. */
  inputFolder?: string;
  /** The output folder's own name ("Output"); empty until the settings are read. */
  outputFolder?: string;
  /** Whether high-confidence documents are renamed without asking. */
  automaticRename?: boolean;
  /** The input folder is the verified team Inbox, which takes a copy only once Microsoft confirms who uploaded it. */
  verifiedInbox?: boolean;
  onDrop(payload: unknown): void;
  /** The queue itself, for the `panel` size. */
  children?: ReactNode;
  /** The pickers, offered in the empty queue itself rather than pointed at. */
  onChooseFiles?(): void;
  onChooseFolder?(): void;
}

/**
 * The same drop target in two sizes.
 *
 * `panel` is a queue that already has work in it: the whole queue takes the
 * drop, and says so with an outline while something is dragged over it. It
 * used to be a dashed strip above the table that took two rows' height on
 * every screen to repeat what the empty queue says once. `hero` is what
 * a first-time user actually meets: an empty table with four column headings
 * and nothing under them said only that the app had loaded. It is the same
 * region, with the same accessible name and the same drop handler, so the
 * first thing a person sees is also the thing they can act on.
 *
 * A drop copies into the input folder - the one place Intern processes
 * documents from - and both sizes say so by the folder's name, so nobody
 * takes it for a separate way in, or wonders where the original went.
 */
export function DropZone({ variant = 'panel', inputFolder = '', outputFolder = '', automaticRename = false, verifiedInbox = false, onDrop, children, onChooseFiles, onChooseFolder }: Props) {
  const [over, setOver] = useState(false);
  const handleDrop = (event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setOver(false);
    onDrop(event.dataTransfer);
  };
  const input = inputFolder || 'your input folder';
  const label = `Drag files or folders here to copy them into ${input}`;
  return <div className={`drop-zone drop-zone--${variant}${over ? ' drag-over' : ''}`} aria-label={label} aria-describedby="supported-formats" role="region"
    onDragOver={(event) => { event.preventDefault(); if (!over) setOver(true); }}
    onDragLeave={(event) => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setOver(false); }}
    onDrop={handleDrop}>
    {variant === 'hero'
      ? <div className="drop-copy">
        <span className="drop-mark"><Icon icon={FileUp} /></span>
        <h2>Drop documents here</h2>
        <p>Intern copies them into your input folder{inputFolder ? <>, <strong>{inputFolder}</strong></> : ''}, reads each one on this device, and proposes a clear new name. Processed files end up in your output folder{outputFolder ? <>, <strong>{outputFolder}</strong></> : ''}.</p>
        <p>{automaticRename
          ? 'Documents Intern is sure about are renamed on their own; the rest wait for you to approve them.'
          : 'Nothing is renamed until you approve it.'} Your originals stay where they are.</p>
        {verifiedInbox && <p>A copy joins the queue once OneDrive has uploaded it and Microsoft confirms it is yours.</p>}
        {(onChooseFiles || onChooseFolder) && <div className="drop-actions">
          {onChooseFiles && <button type="button" className="primary" onClick={onChooseFiles}>Choose files…</button>}
          {onChooseFolder && <button type="button" onClick={onChooseFolder}>Choose a folder…</button>}
        </div>}
        <p className="drop-alternative">Or put files straight into {input}.</p>
        <small id="supported-formats">{FORMATS}</small>
      </div>
      : <>
        <small className="sr-only" id="supported-formats">{FORMATS}</small>
        {children}
      </>}
  </div>;
}
