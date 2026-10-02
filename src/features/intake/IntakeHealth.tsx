import type { DesktopBridge } from '../../lib/bridge';
import { useState } from 'react';
import type { IntakeStatus } from '../../types';

export type IntakeHealth = 'synced' | 'waiting' | 'onedrive_stopped' | 'not_synced';

/**
 * How a watched OneDrive or SharePoint folder is doing, in the terms a person
 * can act on. OneDrive being paused is not something Intern can see; a paused
 * OneDrive simply leaves documents waiting to download, so it reads as
 * waiting. Nothing is reported for a folder OneDrive does not keep, unless it
 * was chosen as a synced folder and OneDrive stopped keeping it.
 */
export function intakeHealth(status: IntakeStatus, myFolder: boolean): IntakeHealth | undefined {
  if (!status.enabled) return undefined;
  const synced = status.cloud !== null && status.cloud.provider !== 'network_share';
  if (!synced) return myFolder ? 'not_synced' : undefined;
  if (status.oneDriveRunning === false) return 'onedrive_stopped';
  if (status.awaitingHydration > 0) return 'waiting';
  return 'synced';
}

/** The words for each state, and the one thing to press. Support codes stay out of these. */
export const HEALTH_COPY: Record<IntakeHealth, { label: string; message(waiting: number): string; action?: string }> = {
  synced: { label: 'Synced', message: () => 'Up to date.' },
  waiting: {
    label: 'Waiting for OneDrive',
    message: (waiting) => `${waiting === 1 ? '1 document is' : `${waiting} documents are`} still downloading from OneDrive.`,
    action: 'Open OneDrive',
  },
  onedrive_stopped: { label: 'OneDrive is not running', message: () => 'OneDrive isn\'t running, so new documents can\'t arrive.', action: 'Start OneDrive' },
  not_synced: { label: 'Not this folder', message: () => 'This folder isn\'t synced by OneDrive anymore.', action: 'Choose folder again' },
};

interface Props {
  status: IntakeStatus;
  myFolder: boolean;
  bridge: DesktopBridge;
  onChooseFolder?(): void;
}

export function IntakeHealthNotice({ status, myFolder, bridge, onChooseFolder }: Props) {
  const [failure, setFailure] = useState('');
  const health = intakeHealth(status, myFolder);
  if (!health) return null;
  const copy = HEALTH_COPY[health];
  const act = () => {
    setFailure('');
    if (health === 'not_synced') { onChooseFolder?.(); return; }
    void bridge.openOneDrive().catch((error: unknown) => {
      const code = typeof error === 'object' && error && 'code' in error ? String(error.code) : '';
      setFailure(code === 'ONEDRIVE_MISSING' ? 'OneDrive is not installed on this computer.' : 'OneDrive could not be started. Open it from the Start menu.');
    });
  };
  return <div className={`intake-health intake-health--${health}`}>
    <p role="status" aria-label="Folder health"><strong>{copy.label}</strong> {copy.message(status.awaitingHydration)}</p>
    {copy.action && (health !== 'not_synced' || onChooseFolder) && <button type="button" onClick={act}>{copy.action}</button>}
    {failure && <p className="form-error" role="alert">{failure}</p>}
  </div>;
}