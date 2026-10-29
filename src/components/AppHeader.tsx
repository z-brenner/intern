import { CircleQuestionMark, FilePlus2, FolderOpen, Pause, Play, Settings } from 'lucide-react';
import { Icon } from './Icon';
import { PrivacyBadge } from './PrivacyBadge';
import type { PrivacyFacts } from './PrivacyBadge';

interface Props {
  paused: boolean;
  inert?: boolean;
  busy?: boolean;
  /** What the privacy badge says, and what it opens to. */
  privacy: PrivacyFacts;
  /** The input folder's own name ("Input"); empty until the settings are read. */
  inputFolder?: string;
  onAddFiles(): void;
  onAddFolder(): void;
  onTogglePause(): void;
  onSettings(trigger: HTMLButtonElement): void;
  onHelp(): void;
}

export function AppHeader({ paused, inert, busy, privacy, inputFolder = '', onAddFiles, onAddFolder, onTogglePause, onSettings, onHelp }: Props) {
  /*
    The badge is the product's promise, so it must not keep making it once a
    hosted model is on. Then it says the opposite, in the same place, so the
    one setting that sends document text off the machine is never invisible.
  */
  const into = inputFolder || 'your input folder';
  return <header className="app-header" inert={inert || undefined}>
    <div className="product"><strong>Intern</strong>
      <PrivacyBadge facts={privacy} onOpenSettings={onSettings} onOpenGuide={onHelp} /></div>
    {/*
      Adding is copying into the input folder, and the buttons say where to,
      so nobody takes them for a second way in that leaves the file where it
      is. The note is the buttons' description as well as visible text.
    */}
    <div className="header-actions">
      <span className="header-note" id="add-destination" title={`Files you add are copied into ${into}. The originals stay where they are.`}>Copied into <strong>{into}</strong></span>
      <button type="button" aria-describedby="add-destination" onClick={onAddFiles}><Icon icon={FilePlus2} /><span className="button-label">Add files</span></button>
      <button type="button" aria-describedby="add-destination" onClick={onAddFolder}><Icon icon={FolderOpen} /><span className="button-label">Add folder</span></button>
      {/*
        Pause says so in words. As a bare two-bar glyph it read as a layout
        control, and a paused queue looked the same as a running one.
      */}
      <button type="button" className={`pause-toggle${paused ? ' paused' : ''}`} disabled={busy} onClick={onTogglePause} aria-label={paused ? 'Resume queue' : 'Pause queue'}><Icon icon={paused ? Play : Pause} /><span className="button-label">{paused ? 'Resume' : 'Pause'}</span></button>
      <span className="header-divider" aria-hidden="true" />
      {/*
        Help sits beside Settings because that is the corner people already
        search when they are stuck, and it opens the published guide in their
        own browser rather than trying to render documentation inside the app.
      */}
      <button type="button" className="icon-button quiet" onClick={onHelp} aria-label="Help & support" title="Help & support"><Icon icon={CircleQuestionMark} /></button>
      <button type="button" className="icon-button quiet" onClick={(event) => onSettings(event.currentTarget)} aria-label="Settings" title="Settings"><Icon icon={Settings} /></button>
    </div>
  </header>;
}
