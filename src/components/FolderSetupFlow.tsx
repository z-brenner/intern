import { useEffect, useRef, useState } from 'react';
import type { DesktopBridge, SelectionBoundary } from '../lib/bridge';
import { contains, filedBeside, folderLabel, folderName, rootName } from '../features/intake/folderNames';
import type { CloudRoot } from '../types';

type Step = 'welcome' | 'pick' | 'filed' | 'existing' | 'ready';

interface Props {
  bridge: DesktopBridge;
  selection?: SelectionBoundary;
  /** First run: start with the welcome screen, which can also be skipped. */
  welcome?: boolean;
  onDone(): Promise<void> | void;
  onSkip?(): Promise<void> | void;
  /** Leave without changing anything, when opened from the app. */
  onCancel?(): void;
}

function describe(error: unknown): string {
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  return 'That did not work. Try again.';
}

/**
 * Three clicks from nothing to a watched folder: choose a folder only you add
 * documents to, say where renamed documents go, and decide once about what is
 * already there. Choosing the folder is the proof that its documents are
 * yours, so there is no Microsoft sign-in here; the verified team Inbox of an
 * administrator's deployment is set up by the other flow.
 */
export function FolderSetupFlow({ bridge, selection, welcome = false, onDone, onSkip, onCancel }: Props) {
  const [step, setStep] = useState<Step>(welcome ? 'welcome' : 'pick');
  const [roots, setRoots] = useState<CloudRoot[]>();
  const [intake, setIntake] = useState('');
  const [destination, setDestination] = useState('');
  const [existing, setExisting] = useState(0);
  const [problem, setProblem] = useState('');
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const heading = useRef<HTMLHeadingElement>(null);
  const onlyNew = useRef<HTMLButtonElement>(null);

  // Each step announces itself from its heading, except the question about
  // existing documents, which starts on the safer answer.
  useEffect(() => { (step === 'existing' ? onlyNew.current : heading.current)?.focus(); }, [step]);
  useEffect(() => { if (step === 'pick' && !roots) void findRoots(); }, [step]);

  const run = async (action: () => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true; setBusy(true); setProblem('');
    try { await action(); }
    catch (error) { setProblem(describe(error)); }
    finally { inFlight.current = false; setBusy(false); }
  };
  const findRoots = async () => {
    try { setRoots((await bridge.cloudRoots()).filter((root) => root.provider !== 'network_share')); }
    catch { setRoots([]); }
  };
  const browse = async () => (await selection?.pickFolder())?.path;
  const choose = (path: string) => { setIntake(path); setStep('filed'); };
  const save = async (watched: string, filed: string, renameExisting: boolean) => {
    const settings = await bridge.getSettings();
    const cloud = await bridge.classifyFolder(watched);
    await bridge.saveSettings({
      ...settings,
      intakeFolder: watched,
      destination: filed,
      intakeEnabled: true,
      // A synced folder is admitted as the person's own; a network share is
      // refused by the backend with its own explanation.
      intakeMyFolder: cloud !== null,
      intakeLocalOnly: cloud === null,
      processOthersUploads: false,
      runInBackground: true,
      startAtLogin: true,
    });
    if (renameExisting) await bridge.addFolder({ path: watched, displayName: folderName(watched) });
    setStep('ready');
  };
  const fileInto = async (watched: string, filed: string) => {
    setIntake(watched);
    setDestination(filed);
    const count = await bridge.intakeFolderDocuments(watched);
    setExisting(count);
    if (count > 0) setStep('existing');
    else await save(watched, filed, false);
  };
  // A synced location's top folder holds everything in it, so Intern makes
  // an Inbox to watch there and a Filed folder beside that, both synced.
  const makeInboxAndFiled = async () => {
    const inbox = await bridge.createInboxFolder(intake);
    await fileInto(inbox, await bridge.createFiledFolder(inbox));
  };

  const known = roots ?? [];
  // A Filed folder beside a synced location's own top folder would sit
  // outside it, where OneDrive does not reach.
  const besideLeavesSync = known.some((root) => contains(intake, root.path));
  const steps: Array<[Step, string]> = [
    ...(welcome ? [['welcome', 'Welcome'] as [Step, string]] : []),
    ['pick', 'Folder'], ['filed', 'Filed'], ['existing', 'Existing documents'], ['ready', 'Ready'],
  ];
  const current = steps.findIndex(([id]) => id === step);
  const alert = problem && <p className="onboarding-alert" role="alert">{problem}</p>;

  return <main className="onboarding" aria-label="Intern setup">
    <aside className="onboarding-rail">
      <p className="onboarding-wordmark">Intern</p>
      <ol aria-label="Setup steps">
        {steps.map(([id, label], index) => <li key={id} className={index < current ? 'done' : index === current ? 'current' : undefined} aria-current={index === current ? 'step' : undefined}>
          <span className="onboarding-rail-mark" aria-hidden="true">{index + 1}</span>{label}{index < current && <span className="sr-only"> (done)</span>}
        </li>)}
      </ol>
    </aside>
    <section className="onboarding-panel">
      {step === 'welcome' && <>
        <h1 ref={heading} tabIndex={-1}>Your documents, renamed and filed</h1>
        <p>Intern renames documents that land in a folder you choose, then files them. Pick a folder that only you add documents to.</p>
        {alert}
        <div className="onboarding-actions">
          <button type="button" className="primary" disabled={busy} onClick={() => setStep('pick')}>Choose a folder</button>
          {onSkip && <button type="button" disabled={busy} onClick={() => void run(async () => { await onSkip(); })}>Not now</button>}
        </div>
      </>}
      {step === 'pick' && <>
        <h1 ref={heading} tabIndex={-1}>Choose a folder</h1>
        {!roots && <p role="status" aria-live="polite">Looking for synced folders on this computer…</p>}
        {roots && roots.length > 0 && <ul className="folder-choices" aria-label="Synced folders">
          {roots.map((root) => <li key={root.path}>
            <span className="folder-choice-name">{rootName(root)}</span>
            <span className="folder-choice-hint">Synced by OneDrive</span>
            <button type="button" aria-label={`Use ${rootName(root)}`} onClick={() => choose(root.path)}>Use this folder</button>
          </li>)}
        </ul>}
        {roots && roots.length === 0 && <p>We didn't find any SharePoint folders on this computer. In SharePoint, open the folder and click Sync (or Add shortcut to My files), then come back.</p>}
        {alert}
        <div className="onboarding-actions">
          {roots && roots.length === 0 && <button type="button" className="primary" onClick={() => { setRoots(undefined); void findRoots(); }}>Check again</button>}
          {selection && <button type="button" disabled={busy} onClick={() => void run(async () => { const path = await browse(); if (path) choose(path); })}>{roots && roots.length === 0 ? 'Browse…' : 'Browse for another folder…'}</button>}
          {onCancel && <button type="button" onClick={onCancel}>Cancel</button>}
        </div>
      </>}
      {step === 'filed' && <>
        <h1 ref={heading} tabIndex={-1}>Where should renamed documents go?</h1>
        {besideLeavesSync
          ? <p>You chose all of <strong>{folderLabel(known, intake)}</strong>. Intern will make two folders in it: <strong>Inbox</strong>, where you put documents to rename, and <strong>Filed</strong>, where they go once renamed.</p>
          : <>
            <p>Intern will watch <strong>{folderLabel(known, intake)}</strong>.</p>
            <p>Renamed documents can go to a new folder called Filed, next to the one you chose.</p>
          </>}
        {alert}
        <div className="onboarding-actions">
          {besideLeavesSync
            ? <button type="button" className="primary" disabled={busy} onClick={() => void run(makeInboxAndFiled)}>Create Inbox and Filed folders</button>
            : <>
              <button type="button" className="primary" disabled={busy} onClick={() => void run(async () => { await fileInto(intake, await bridge.createFiledFolder(intake)); })}>Create a “Filed” folder here</button>
              {selection && <button type="button" disabled={busy} onClick={() => void run(async () => { const path = await browse(); if (path) await fileInto(intake, path); })}>Choose another folder…</button>}
            </>}
          <button type="button" disabled={busy} onClick={() => { setProblem(''); setStep('pick'); }}>Back</button>
        </div>
        {!besideLeavesSync && <p className="onboarding-support">Filed folder: {filedBeside(intake)}</p>}
      </>}
      {step === 'existing' && <>
        <h1 ref={heading} tabIndex={-1}>Rename what is already there?</h1>
        <p>There {existing === 1 ? 'is 1 document' : `are ${existing} documents`} already in this folder. Rename them too?</p>
        {alert}
        <div className="onboarding-actions">
          <button type="button" disabled={busy} onClick={() => void run(() => save(intake, destination, true))}>Rename them</button>
          <button type="button" ref={onlyNew} className="primary" disabled={busy} onClick={() => void run(() => save(intake, destination, false))}>Only new documents</button>
        </div>
      </>}
      {step === 'ready' && <>
        <h1 ref={heading} tabIndex={-1} className="onboarding-finished">Watching {folderLabel(known, intake)}.</h1>
        <p>Renamed documents go to {folderLabel(known, destination)}.</p>
        <p>Intern keeps running in the system tray and starts when you sign in, so documents are filed while its window is closed. You can change this in Settings.</p>
        {alert}
        <div className="onboarding-actions">
          <button type="button" className="primary" disabled={busy} onClick={() => void run(async () => { await onDone(); })}>Done</button>
        </div>
      </>}
    </section>
  </main>;
}