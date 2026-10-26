import { FilePlus2, FolderOpen, Pause, Play } from 'lucide-react';
import { Icon } from './Icon';

interface Props {
  paused: boolean;
  inert?: boolean;
  busy?: boolean;
  hosted?: boolean;
  /** The input folder's own name ("Input"); empty until the settings are read. */
  inputFolder?: string;
  onAddFiles(): void;
  onAddFolder(): void;
  onTogglePause(): void;
}

export function AppHeader({ paused, inert, busy, hosted, inputFolder = '', onAddFiles, onAddFolder, onTogglePause }: Props) {
  /*
    The badge is the product's promise, so it must not keep making it once a
    hosted model is on. Then it says the opposite, in the same place, so the
    one setting that sends document text off the machine is never invisible.
  */
  const into = inputFolder || 'your input folder';
  return <header className="app-header" inert={inert || undefined}>
    <div className="product"><strong>Intern</strong>{hosted
      ? <span className="private hosted" title="A hosted model is reading your documents. Change it in Settings."><i />Hosted model · Text leaves this device</span>
      : <span className="private"><i />Private · On this device</span>}</div>
    {/*
      Adding is copying into the input folder, and the buttons say where to,
      so nobody takes them for a second way in that leaves the file where it
      is. The note is the buttons' description as well as visible text.
    */}
    <div className="header-actions">
      <span className="header-note" id="add-destination" title={`Files you add are copied into ${into}. The originals stay where they are.`}>Copied into <strong>{into}</strong></span>
      <button type="button" aria-describedby="add-destination" onClick={onAddFiles}><Icon icon={FilePlus2} />Add files</button>
      <button type="button" aria-describedby="add-destination" onClick={onAddFolder}><Icon icon={FolderOpen} />Add folder</button>
      <button type="button" className="icon-button" disabled={busy} onClick={onTogglePause} aria-label={paused ? 'Resume queue' : 'Pause queue'}><Icon icon={paused ? Play : Pause} /></button>
    </div>
  </header>;
}
