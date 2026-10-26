import { useEffect, useRef, useState } from 'react';
import { describeAddReport } from '../lib/addReport';
import type { DesktopBridge, SelectionBoundary } from '../lib/bridge';
import { contains, folderLabel, outputBeside, rootName } from '../features/intake/folderNames';
import { folderSaveProblem, outputFolderProblem } from '../features/intake/folderProblems';
import type { AddReport, CloudLocation, CloudRoot } from '../types';

type Step = 'welcome' | 'pick' | 'output' | 'existing' | 'ready';

interface Props {
  bridge: DesktopBridge;
  selection?: SelectionBoundary;
  /** First run, or a launch with no input folder: start with the welcome screen. There is no skipping it. */
  welcome?: boolean;
  onDone(): Promise<void> | void;
  /** Leave without changing anything, when opened from Settings. */
  onCancel?(): void;
}

/** What the input folder is for, said the same way everywhere. */
export const INPUT_FOLDER_HINT = 'Put files here that you want Intern to process.';
/** What the output folder is for, said the same way everywhere. */
export const OUTPUT_FOLDER_HINT = 'Processed files end up here, renamed.';

/** A OneDrive or SharePoint location, where description records can fill SharePoint's Description column. */
export function syncedByOneDrive(location: CloudLocation | null | undefined): boolean {
  return location?.provider === 'onedrive_business' || location?.provider === 'onedrive_personal' || location?.provider === 'sharepoint';
}

function codeOf(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}

function describe(error: unknown): string {
  const plain = folderSaveProblem(codeOf(error));
  if (plain) return plain;
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  return 'That did not work. Try again.';
}

/**
 * The setup Intern cannot be used without: an input folder - put files here
 * that you want processed - and an output folder - processed files end up
 * here, renamed - then one question about what is already in the input
 * folder. Choosing the input folder is the proof that its documents are the
 * person's own, so there is no Microsoft sign-in here; the verified team
 * Inbox of an administrator's deployment is set up by the other flow.
 */
export function FolderSetupFlow({ bridge, selection, welcome = false, onDone, onCancel }: Props) {
  const [step, setStep] = useState<Step>(welcome ? 'welcome' : 'pick');
  const [roots, setRoots] = useState<CloudRoot[]>();
  const [intake, setIntake] = useState('');
  const [destination, setDestination] = useState('');
  // The folders already saved, when there are any: an earlier release let a
  // person choose an output folder and skip the input folder, and that
  // choice is offered again rather than asked from scratch.
  const [saved, setSaved] = useState<{ input: string; output: string }>({ input: '', output: '' });
  const [existing, setExisting] = useState(0);
  const [problem, setProblem] = useState('');
  // What processing the documents already there came to, when some were
  // left out. The add no longer fails whole for one unreadable file - or for
  // a folder it cannot read - so without this the last step said they were
  // being processed while some, or all, were not.
  const [leftOut, setLeftOut] = useState<AddReport>();
  const [describesForSharePoint, setDescribesForSharePoint] = useState(false);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const heading = useRef<HTMLHeadingElement>(null);
  const onlyNew = useRef<HTMLButtonElement>(null);

  // Each step announces itself from its heading, except the question about
  // existing documents, which starts on the safer answer.
  useEffect(() => { (step === 'existing' ? onlyNew.current : heading.current)?.focus(); }, [step]);
  useEffect(() => { if (step === 'pick' && !roots) void findRoots(); }, [step]);
  useEffect(() => {
    let active = true;
    void Promise.resolve().then(() => bridge.getSettings())
      .then((settings) => { if (active) setSaved({ input: settings.intakeFolder.trim(), output: settings.destination.trim() }); })
      .catch(() => { /* Nothing to offer again; every choice is still there. */ });
    return () => { active = false; };
  }, [bridge]);

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
  const choose = (path: string) => { setProblem(''); setIntake(path); setStep('output'); };
  const save = async (watched: string, filed: string, processExisting: boolean) => {
    const settings = await bridge.getSettings();
    const cloud = await bridge.classifyFolder(watched);
    // Description records fill SharePoint's Description column through a
    // flow the guide ships; they only mean something in a library that
    // OneDrive syncs, so they are switched on for an output folder there and
    // left as they were anywhere else.
    const output = await bridge.classifyFolder(filed).catch(() => null);
    const describeForSharePoint = syncedByOneDrive(output);
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
      ...(describeForSharePoint ? { recordDescriptions: true } : {}),
    });
    setDescribesForSharePoint(describeForSharePoint);
    const report = processExisting ? await bridge.queueInputFolder() : undefined;
    setLeftOut(report && report.skipped.length > 0 ? report : undefined);
    setStep('ready');
  };
  const fileInto = async (watched: string, filed: string) => {
    const refused = outputFolderProblem(watched, filed);
    if (refused) { setProblem(refused); return; }
    setIntake(watched);
    setDestination(filed);
    const count = await bridge.intakeFolderDocuments(watched);
    setExisting(count);
    if (count > 0) setStep('existing');
    else await save(watched, filed, false);
  };
  // A synced location's top folder holds everything in it, so Intern makes
  // an Input folder to watch there and an Output folder beside that, both
  // synced.
  const makeInputAndOutput = async () => {
    const input = await bridge.createInputFolder(intake);
    await fileInto(input, await bridge.createOutputFolder(input));
  };

  const known = roots ?? [];
  // An output folder beside a synced location's own top folder would sit
  // outside it, where OneDrive does not reach.
  const besideLeavesSync = known.some((root) => contains(intake, root.path));
  // A drive's or a share's top folder has nothing beside it: the backend
  // refuses to make an output folder there, so the button that asked it to
  // could only ever fail.
  const beside = besideLeavesSync ? undefined : outputBeside(intake);
  const nothingBeside = !besideLeavesSync && beside === undefined;
  // The output folder already saved, when it still works with this input.
  const keep = saved.output && !besideLeavesSync && !outputFolderProblem(intake, saved.output) ? saved.output : '';
  // The input folder saved before watching was turned off, offered again -
  // unless it is one of the synced locations listed anyway.
  const previousInput = saved.input && !known.some((root) => contains(root.path, saved.input) && contains(saved.input, root.path)) ? saved.input : '';
  const steps: Array<[Step, string]> = [
    ...(welcome ? [['welcome', 'Welcome'] as [Step, string]] : []),
    ['pick', 'Input folder'], ['output', 'Output folder'], ['existing', 'Files already there'], ['ready', 'Ready'],
  ];
  const current = steps.findIndex(([id]) => id === step);
  const alert = problem && <p className="onboarding-alert" role="alert">{problem}</p>;
  const label = (path: string) => folderLabel(known, path);
  const noSuggestions = roots !== undefined && roots.length === 0 && !previousInput;

  return <main className="onboarding" aria-label="Intern setup">
    <aside className="onboarding-rail">
      <p className="onboarding-wordmark">Intern</p>
      <ol aria-label="Setup steps">
        {steps.map(([id, text], index) => <li key={id} className={index < current ? 'done' : index === current ? 'current' : undefined} aria-current={index === current ? 'step' : undefined}>
          <span className="onboarding-rail-mark" aria-hidden="true">{index + 1}</span>{text}{index < current && <span className="sr-only"> (done)</span>}
        </li>)}
      </ol>
    </aside>
    <section className="onboarding-panel">
      {step === 'welcome' && <>
        <h1 ref={heading} tabIndex={-1}>Your documents, renamed and filed</h1>
        <p>Intern works with two folders. Choose them now; you can change them later in Settings.</p>
        <dl className="setup-folders">
          <div><dt>Input folder</dt><dd><span>{INPUT_FOLDER_HINT}</span></dd></div>
          <div><dt>Output folder</dt><dd><span>{OUTPUT_FOLDER_HINT}</span></dd></div>
        </dl>
        {alert}
        <div className="onboarding-actions">
          <button type="button" className="primary" disabled={busy} onClick={() => setStep('pick')}>Choose folders</button>
        </div>
      </>}
      {step === 'pick' && <>
        <h1 ref={heading} tabIndex={-1}>Choose your input folder</h1>
        <p><strong>{INPUT_FOLDER_HINT}</strong> Pick a folder only you add files to. Files you drop on Intern’s window are copied into it too.</p>
        {!roots && <p role="status" aria-live="polite">Looking for OneDrive folders on this computer…</p>}
        {roots && (roots.length > 0 || previousInput) && <ul className="folder-choices" aria-label="Suggested folders">
          {previousInput && <li>
            <span className="folder-choice-name">{label(previousInput)}</span>
            <span className="folder-choice-hint">The folder you chose before</span>
            <button type="button" aria-label={`Use ${label(previousInput)}`} onClick={() => choose(previousInput)}>Use this folder</button>
          </li>}
          {roots.map((root) => <li key={root.path}>
            <span className="folder-choice-name">{rootName(root)}</span>
            <span className="folder-choice-hint">Synced by OneDrive</span>
            <button type="button" aria-label={`Use ${rootName(root)}`} onClick={() => choose(root.path)}>Use this folder</button>
          </li>)}
        </ul>}
        {roots && roots.length === 0 && <p>No OneDrive or SharePoint folders were found on this computer. Any folder on this computer works too.</p>}
        {alert}
        <div className="onboarding-actions">
          {selection && <button type="button" className={noSuggestions ? 'primary' : undefined} disabled={busy} onClick={() => void run(async () => { const path = await browse(); if (path) choose(path); })}>{noSuggestions ? 'Choose a folder…' : 'Choose another folder…'}</button>}
          {roots && roots.length === 0 && <button type="button" disabled={busy} onClick={() => { setRoots(undefined); void findRoots(); }}>Look for OneDrive folders again</button>}
          {onCancel && <button type="button" onClick={onCancel}>Cancel</button>}
        </div>
      </>}
      {step === 'output' && <>
        <h1 ref={heading} tabIndex={-1}>Choose your output folder</h1>
        <p><strong>{OUTPUT_FOLDER_HINT}</strong> It has to be outside your input folder.</p>
        {besideLeavesSync
          ? <p>You chose all of <strong>{label(intake)}</strong>. Intern will make two folders in it: <strong>Input</strong>, for files to process, and <strong>Output</strong>, where processed files end up.</p>
          : <>
            <p>Input folder: <strong>{label(intake)}</strong></p>
            {keep && <p>Your output folder until now: <strong>{label(keep)}</strong></p>}
            {!keep && nothingBeside && <p>A drive’s top folder has nothing beside it. Choose a folder for processed files.</p>}
          </>}
        {alert}
        <div className="onboarding-actions">
          {besideLeavesSync
            ? <button type="button" className="primary" disabled={busy} onClick={() => void run(makeInputAndOutput)}>Create Input and Output folders</button>
            : <>
              {keep && <button type="button" className="primary" disabled={busy} onClick={() => void run(() => fileInto(intake, keep))}>Keep using this folder</button>}
              {!nothingBeside && <button type="button" className={keep ? undefined : 'primary'} disabled={busy} onClick={() => void run(async () => { await fileInto(intake, await bridge.createOutputFolder(intake)); })}>Create an “Output” folder next to it</button>}
              {selection && <button type="button" className={nothingBeside && !keep ? 'primary' : undefined} disabled={busy} onClick={() => void run(async () => { const path = await browse(); if (path) await fileInto(intake, path); })}>Choose another folder…</button>}
            </>}
          <button type="button" disabled={busy} onClick={() => { setProblem(''); setStep('pick'); }}>Back</button>
        </div>
        {beside !== undefined && <p className="onboarding-support">New Output folder: {beside}</p>}
      </>}
      {step === 'existing' && <>
        <h1 ref={heading} tabIndex={-1}>Process what is already there?</h1>
        <p>Your input folder already has {existing === 1 ? '1 file' : `${existing} files`} in it. Should Intern process {existing === 1 ? 'it' : 'them'} too, or only files you add from now on?</p>
        {alert}
        <div className="onboarding-actions">
          <button type="button" disabled={busy} onClick={() => void run(() => save(intake, destination, true))}>Process {existing === 1 ? 'it' : 'them'} too</button>
          <button type="button" ref={onlyNew} className="primary" disabled={busy} onClick={() => void run(() => save(intake, destination, false))}>Only new files</button>
        </div>
      </>}
      {step === 'ready' && <>
        <h1 ref={heading} tabIndex={-1} className="onboarding-finished">You’re all set</h1>
        <dl className="setup-folders">
          <div><dt>Input folder</dt><dd><strong>{label(intake)}</strong><span>{INPUT_FOLDER_HINT} You can also drop them on Intern’s window.</span></dd></div>
          <div><dt>Output folder</dt><dd><strong>{label(destination)}</strong><span>{OUTPUT_FOLDER_HINT}</span></dd></div>
        </dl>
        <p>You can change either folder any time in Settings.</p>
        <p>Intern keeps running in the system tray and starts when you sign in, so files are processed while its window is closed. You can change this in Settings.</p>
        {describesForSharePoint && <p>Your output folder is in OneDrive, so Intern also leaves a short description of each document there for SharePoint’s Description column. Settings shows how to fill that column.</p>}
        {leftOut && <div className="note note--review" role="note" aria-label="Files not added">
          <p>{describeAddReport(leftOut)} They stay in your input folder.</p>
        </div>}
        {alert}
        <div className="onboarding-actions">
          <button type="button" className="primary" disabled={busy} onClick={() => void run(async () => { await onDone(); })}>Done</button>
        </div>
      </>}
    </section>
  </main>;
}
