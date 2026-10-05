import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';
import type { DesktopBridge, SelectionBoundary } from '../lib/bridge';
import { formatBytes } from '../lib/format';
import type { MicrosoftDevicePrompt, MicrosoftIntakeStatus } from '../features/intake/microsoft';
import { describeSharePointProblem } from '../features/sharepoint/sharePointProblems';
import { useSupportLink } from '../features/sharepoint/useSupportLink';
import type { SharePointProblem } from '../features/sharepoint/sharePointProblems';
import { modelReady, useModelSetup } from '../features/setup/useModelSetup';
import type { AppSettings, SetupState, SharePointSetupStatus } from '../types';
import { DownloadProgress } from './DownloadProgress';
import { SettingsDialog } from './SettingsDialog';

/** How often the sync step asks the backend to rescan OneDrive's registered libraries. */
const RESCAN_INTERVAL_MS = 5000;
/** Codes a rescan can report while OneDrive is still adding the library; the step keeps waiting through them. */
const TRANSIENT_WHILE_SYNCING = ['SHAREPOINT_SYNC_PENDING'];

type Step = 'welcome' | 'model' | 'microsoft' | 'sync' | 'activate' | 'finished';
const STEPS: Array<[Step, string]> = [
  ['welcome', 'Welcome'],
  ['model', 'Local model'],
  ['microsoft', 'Microsoft account'],
  ['sync', 'Library sync'],
  ['activate', 'Turn on filing'],
  ['finished', 'Finished'],
];

interface OnboardingFlowProps {
  bridge: DesktopBridge;
  selection?: SelectionBoundary;
  /** Reads App started alongside the onboarding check. */
  pendingSettings?: Promise<AppSettings>;
  pendingSetup?: Promise<SetupState>;
  initialSetup?: SetupState;
  /** Called only after completion has been recorded by the backend. */
  onComplete(): void;
}

/**
 * Guided setup for the fixed SharePoint deployment. Nothing here is stored in
 * the page: each step asks the backend where things stand, so closing Intern
 * part-way and opening it again resumes from the real state.
 */
export function OnboardingFlow({ bridge, selection, pendingSettings, pendingSetup, initialSetup, onComplete }: OnboardingFlowProps) {
  const [step, setStep] = useState<Step>('welcome');
  const [library, setLibrary] = useState<SharePointSetupStatus>();
  const heading = useRef<HTMLHeadingElement>(null);
  const model = useModelSetup(bridge, selection, { initial: initialSetup, pending: pendingSetup });
  const ready = modelReady(model.setup);

  // Every step change moves focus to the new step's heading, so keyboard and
  // screen reader users land at the start of what changed.
  useEffect(() => { heading.current?.focus(); }, [step]);
  useEffect(() => { if (step === 'model' && ready) setStep('microsoft'); }, [step, ready]);

  const showLibrary = (status: SharePointSetupStatus) => {
    setLibrary(status);
    setStep(status.phase === 'active' ? 'finished' : status.phase === 'ready_to_activate' ? 'activate' : 'sync');
  };
  const current = STEPS.findIndex(([id]) => id === step);

  return <main className="onboarding" aria-label="Intern setup">
    <aside className="onboarding-rail">
      <p className="onboarding-wordmark">Intern</p>
      <ol aria-label="Setup steps">
        {STEPS.map(([id, label], index) => <li key={id} className={index < current ? 'done' : index === current ? 'current' : undefined} aria-current={index === current ? 'step' : undefined}>
          <span className="onboarding-rail-mark" aria-hidden="true">{index + 1}</span>{label}{index < current && <span className="sr-only"> (done)</span>}
        </li>)}
      </ol>
    </aside>
    <section className="onboarding-panel">
      {step === 'welcome' && <>
        <h1 ref={heading} tabIndex={-1}>Your team's documents, filed for you</h1>
        <p>Intern watches your team's Inbox in SharePoint, reads each new document privately on this computer, gives it a clear name, and moves it to Filed.</p>
        <p>It handles only documents you upload yourself. Documents uploaded by anyone else are left alone.</p>
        <div className="onboarding-actions"><button type="button" className="primary" onClick={() => setStep(ready ? 'microsoft' : 'model')}>Set up Intern</button></div>
      </>}
      {step === 'model' && <ModelStep heading={heading} bridge={bridge} selection={selection} model={model} pendingSettings={pendingSettings} />}
      {step === 'microsoft' && <MicrosoftStep heading={heading} bridge={bridge} onConfirmed={() => setStep('sync')} />}
      {step === 'sync' && <SyncStep heading={heading} bridge={bridge} onLibrary={showLibrary} onSwitchAccount={() => setStep('microsoft')} />}
      {step === 'activate' && library && <ActivateStep heading={heading} bridge={bridge} onLibrary={showLibrary} />}
      {step === 'finished' && library && <FinishedStep heading={heading} bridge={bridge} library={library} onLibrary={showLibrary} onComplete={onComplete} />}
    </section>
  </main>;
}

/** A failure: the plain-language action as an alert, the stable code beside it for support. */
export function OnboardingProblemNotice({ problem }: { problem: SharePointProblem }) {
  return <div className="onboarding-problem">
    <p role="alert">{problem.action}</p>
    <p className="onboarding-support">Support code <code>{problem.code}</code></p>
    {problem.detail && <details className="onboarding-support"><summary>Support details</summary><p>{problem.detail}</p></details>}
  </div>;
}

type HeadingRef = RefObject<HTMLHeadingElement | null>;

/** One action at a time: a second click while a call is in flight is ignored. */
function useAction() {
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<SharePointProblem>();
  const mounted = useRef(true);
  const inFlight = useRef(false);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const run = async (action: () => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true; setBusy(true); setProblem(undefined);
    try { await action(); }
    catch (cause) { if (mounted.current) setProblem(describeSharePointProblem(cause)); }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  };
  return { busy, problem, setProblem, run, mounted };
}

function ModelStep({ heading, bridge, selection, model, pendingSettings }: { heading: HeadingRef; bridge: DesktopBridge; selection?: SelectionBoundary; model: ReturnType<typeof useModelSetup>; pendingSettings?: Promise<AppSettings> }) {
  const [settings, setSettings] = useState<AppSettings>();
  const [hostedOpen, setHostedOpen] = useState(false);
  const hostedTrigger = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    let active = true;
    void (pendingSettings ?? bridge.getSettings()).then((loaded) => { if (active) setSettings(loaded); }).catch(() => {});
    return () => { active = false; };
  }, [bridge, pendingSettings]);
  const { setup, busy, operationError } = model;
  const downloading = setup?.state === 'downloading';
  const failed = setup?.state === 'failed';
  const canceled = setup?.state === 'required' && setup.error === 'MODEL_DOWNLOAD_CANCELED';
  const resumable = Boolean(setup && setup.downloadedBytes > 0 && setup.downloadedBytes < setup.totalBytes);
  const closeHosted = () => { setHostedOpen(false); hostedTrigger.current?.focus(); };
  return <>
    <h1 ref={heading} tabIndex={-1}>Get the local model</h1>
    <p>Intern reads documents with a model that runs on this computer, so your documents are never sent anywhere to be read. It downloads once{setup && setup.totalBytes > 0 ? ` (${formatBytes(setup.totalBytes)})` : ''}.</p>
    {!setup && !operationError && <p role="status" aria-live="polite" aria-label="Model setup status">Checking the local model…</p>}
    {operationError && <p className="onboarding-alert" role="alert">{operationError}</p>}
    {canceled && !operationError && <p role="status" aria-live="polite" aria-label="Model setup status">Download canceled. Your progress was saved.</p>}
    {setup && (downloading || resumable) && <div className="onboarding-progress">
      <DownloadProgress setup={setup} />
    </div>}
    {setup && <div className="onboarding-actions">
      <button type="button" className="primary" onClick={model.start} disabled={downloading || busy}>{downloading ? 'Downloading model…' : failed ? 'Try download again' : resumable ? 'Resume download' : 'Download model'}</button>
      {downloading && <button type="button" onClick={model.cancel} disabled={busy}>Cancel download</button>}
    </div>}
    <details className="onboarding-advanced">
      <summary>Other ways to get the model</summary>
      <p>For support or special setups. Most people should download the model above.</p>
      <div className="onboarding-actions">
        <button type="button" onClick={model.chooseExisting} disabled={downloading || busy || !model.canChooseExisting}>Choose existing model files</button>
      </div>
      <p>A hosted model needs no download, but the text of every document is sent to the service you choose, using your own API key.</p>
      <div className="onboarding-actions">
        <button type="button" ref={hostedTrigger} onClick={() => setHostedOpen(true)} disabled={downloading || busy || !settings}>Use a hosted model instead</button>
      </div>
    </details>
    {hostedOpen && settings && <SettingsDialog hideSharePointConnection settings={settings} bridge={bridge} selection={selection} onClose={closeHosted} onSave={async (next) => { await bridge.saveSettings(next); setSettings(next); closeHosted(); await model.refresh(); }} onCheckForUpdate={() => bridge.checkForUpdate()} onInstallUpdate={(onProgress) => bridge.installUpdate(onProgress)} />}
  </>;
}

function MicrosoftStep({ heading, bridge, onConfirmed }: { heading: HeadingRef; bridge: DesktopBridge; onConfirmed(): void }) {
  const [status, setStatus] = useState<MicrosoftIntakeStatus>();
  const [prompt, setPrompt] = useState<MicrosoftDevicePrompt>();
  const { busy, problem, setProblem, run, mounted } = useAction();
  const generation = useRef(0);
  const signingIn = useRef(false);
  const available = Boolean(bridge.microsoftIntakeStatus && bridge.microsoftSignInStart && bridge.microsoftSignInPoll && bridge.microsoftDisconnect);

  const refresh = async () => {
    const next = await bridge.microsoftIntakeStatus?.();
    if (!next || !mounted.current) return;
    setStatus(next);
    if (next.error) setProblem(describeSharePointProblem(next.error));
  };
  useEffect(() => {
    let active = true;
    void bridge.microsoftIntakeStatus?.().then((next) => {
      if (!active) return;
      setStatus(next);
      if (next.error) setProblem(describeSharePointProblem(next.error));
    }).catch((cause) => { if (active) setProblem(describeSharePointProblem(cause)); });
    return () => {
      active = false; generation.current += 1;
      // Leaving during a pending sign-in must not silently connect later.
      if (signingIn.current) { signingIn.current = false; void bridge.microsoftDisconnect?.().catch(() => {}); }
    };
  }, [bridge]);

  const begin = () => void run(async () => {
    const version = ++generation.current;
    signingIn.current = true;
    try {
      const next = await bridge.microsoftSignInStart!();
      if (mounted.current && generation.current === version) setPrompt(next);
    } catch (cause) { signingIn.current = false; throw cause; }
  });
  const disconnect = () => void run(async () => {
    generation.current += 1; signingIn.current = false; setPrompt(undefined);
    await bridge.microsoftDisconnect?.();
    await refresh();
    heading.current?.focus();
  });

  useEffect(() => {
    if (!prompt) return;
    const version = generation.current;
    let active = true;
    let timer: number | undefined;
    const stop = (next?: SharePointProblem) => { signingIn.current = false; setPrompt(undefined); if (next) setProblem(next); };
    const poll = async () => {
      if (!active || generation.current !== version) return;
      if (Date.now() >= prompt.expiresAt * 1000) {
        stop(describeSharePointProblem({ code: 'MICROSOFT_SIGN_IN_EXPIRED' }));
        void bridge.microsoftDisconnect?.().catch(() => {});
        return;
      }
      try {
        const result = await bridge.microsoftSignInPoll!();
        if (!active || generation.current !== version) return;
        if (result.state === 'connected') { stop(); await refresh(); }
        else timer = window.setTimeout(() => { void poll(); }, Math.max(5, result.intervalSeconds) * 1000);
      } catch (cause) {
        if (active) stop(describeSharePointProblem(cause));
      }
    };
    timer = window.setTimeout(() => { void poll(); }, Math.max(5, prompt.intervalSeconds) * 1000);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, prompt]);

  const account = status?.connected ? status.account : null;
  return <>
    <h1 ref={heading} tabIndex={-1}>Connect your Microsoft account</h1>
    <p>Intern uses your work Microsoft account to tell which documents in the Inbox you uploaded. Through Microsoft it checks only file details, such as who uploaded a document; it reads documents only from this computer.</p>
    {problem && <OnboardingProblemNotice problem={problem} />}
    {account ? <>
      <div className="onboarding-account">
        <strong>{account.displayName}</strong>
        <span>{account.email}</span>
      </div>
      <p>Is this the account you use to upload documents to the Inbox?</p>
      <div className="onboarding-actions">
        <button type="button" className="primary" disabled={busy} onClick={onConfirmed}>Yes, this is my account</button>
        <button type="button" disabled={busy} onClick={disconnect}>Use a different account</button>
      </div>
    </> : prompt ? <>
      <div className="onboarding-signin">
        <p>Open Microsoft's sign-in page, enter this code, and sign in with your work account:</p>
        <strong className="onboarding-code">{prompt.userCode}</strong>
      </div>
      <p className="onboarding-waiting" role="status" aria-live="polite" aria-label="Microsoft sign-in">Waiting for you to finish signing in with Microsoft…</p>
      <div className="onboarding-actions">
        <button type="button" className="primary" disabled={busy || !bridge.microsoftOpenSignIn} onClick={() => void run(async () => { await bridge.microsoftOpenSignIn!(); })}>Open Microsoft sign-in</button>
        <button type="button" disabled={busy} onClick={disconnect}>Cancel sign-in</button>
      </div>
    </> : <div className="onboarding-actions">
      <button type="button" className="primary" disabled={!available || busy || !status || Boolean(status.error)} onClick={begin}>Connect Microsoft account</button>
    </div>}
  </>;
}

function SyncStep({ heading, bridge, onLibrary, onSwitchAccount }: { heading: HeadingRef; bridge: DesktopBridge; onLibrary(status: SharePointSetupStatus): void; onSwitchAccount(): void }) {
  const [needsSync, setNeedsSync] = useState(false);
  const [waiting, setWaiting] = useState(false);
  // Why the library is still unconfirmed, when the backend knows: a OneDrive
  // record problem reported alongside the pending status.
  const [pendingProblem, setPendingProblem] = useState<SharePointProblem>();
  const { busy, problem: failure, setProblem, run, mounted } = useAction();
  const support = useSupportLink(bridge);

  // Pending libraries stay on this step; anything further along moves on.
  const route = (status: SharePointSetupStatus) => {
    const reported = status.phase === 'enrollment_pending' && status.problem ? status.problem : undefined;
    // Rescans repeat the same problem every few seconds; keep the shown one
    // unless it changed, so the alert is not announced again each time.
    setPendingProblem((shown) => !reported ? undefined
      : shown?.code === reported.code && shown.detail === (reported.message.trim() || undefined) ? shown
        : describeSharePointProblem(reported));
    if (status.phase === 'enrollment_pending') { setNeedsSync(true); return false; }
    onLibrary(status);
    return true;
  };
  const check = () => void run(async () => {
    setWaiting(false);
    const status = await bridge.getSharePointSetup();
    if (mounted.current) route(status);
  });
  const requestSync = () => void run(async () => {
    setWaiting(false);
    const status = await bridge.startSharePointSync();
    if (mounted.current && !route(status)) setWaiting(true);
  });
  useEffect(() => { check(); }, [bridge]);

  useEffect(() => {
    if (!waiting) return;
    let active = true;
    let timer: number | undefined;
    const rescan = async () => {
      try {
        const status = await bridge.getSharePointSetup();
        if (!active) return;
        if (route(status)) return;
      } catch (cause) {
        if (!active) return;
        const next = describeSharePointProblem(cause);
        // Still syncing is not a failure; keep waiting. OneDrive writes the
        // library's records a piece at a time, so a rescan that lands partway
        // through can fail to confirm a library that is about to appear.
        if (!TRANSIENT_WHILE_SYNCING.includes(next.code)) { setWaiting(false); setProblem(next); return; }
      }
      timer = window.setTimeout(() => { void rescan(); }, RESCAN_INTERVAL_MS);
    };
    timer = window.setTimeout(() => { void rescan(); }, RESCAN_INTERVAL_MS);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, waiting]);

  const switchAccount = () => void run(async () => {
    await bridge.microsoftDisconnect?.();
    if (mounted.current) onSwitchAccount();
  });
  const problem = failure ?? pendingProblem;
  const recovery = waiting || problem;
  // Asking OneDrive to sync is one click away while the library is pending,
  // unless a problem shows that another request would fail the same way.
  const offerSync = !waiting && (problem ? Boolean(problem.offerSync) : needsSync);
  // With nothing else to press, checking again is the way forward.
  const retryFirst = Boolean(problem) && !problem?.switchAccount && !problem?.getOneDrive && !offerSync;
  return <>
    <h1 ref={heading} tabIndex={-1}>Sync the Files library</h1>
    <p>Intern works on your team's Files library through OneDrive, so documents stay on this computer while they are read. Intern asks OneDrive to sync the InternTestSite Files library for you.</p>
    {problem && <OnboardingProblemNotice problem={problem} />}
    {waiting && <>
      <p>OneDrive may ask you to confirm. If it does, choose Sync.</p>
      <p className="onboarding-waiting" role="status" aria-live="polite" aria-label="Library sync">Waiting for OneDrive to add the Files library. Intern checks again every few seconds. You can close Intern and finish later.</p>
    </>}
    {support.error && <p className="onboarding-alert" role="alert">{support.error}</p>}
    {!waiting && !problem && !needsSync && <p role="status" aria-live="polite" aria-label="Library sync">Looking for the Files library on this computer…</p>}
    <div className="onboarding-actions">
      {problem?.switchAccount && <button type="button" className="primary" disabled={busy} onClick={switchAccount}>Use a different account</button>}
      {offerSync && <button type="button" className={problem?.switchAccount ? undefined : 'primary'} disabled={busy} onClick={requestSync}>Sync Files with OneDrive</button>}
      {recovery && <button type="button" className={retryFirst ? 'primary' : undefined} disabled={busy} onClick={waiting ? requestSync : check}>Try again</button>}
      {problem?.otherAccount && <button type="button" disabled={busy} onClick={switchAccount}>Use a different account</button>}
      {recovery && <button type="button" onClick={() => support.open('sharepoint-site')}>Open SharePoint</button>}
      {problem?.getOneDrive && <button type="button" onClick={() => support.open('onedrive-download')}>Get OneDrive</button>}
    </div>
  </>;
}

function ActivateStep({ heading, bridge, onLibrary }: { heading: HeadingRef; bridge: DesktopBridge; onLibrary(status: SharePointSetupStatus): void }) {
  const { busy, problem, run, mounted } = useAction();
  const activate = () => void run(async () => {
    const status = await bridge.activateOnboarding();
    if (mounted.current) onLibrary(status);
  });
  return <>
    <h1 ref={heading} tabIndex={-1}>Turn on filing</h1>
    <p>OneDrive has the Files library on this computer. When you turn on filing, Intern will:</p>
    <ul className="onboarding-list">
      <li>watch Files/Inbox for documents you upload</li>
      <li>give each one a clear name and move it to Files/Filed</li>
      <li>leave documents uploaded by anyone else alone</li>
      <li>start when you sign in to Windows and keep running in the system tray</li>
    </ul>
    <p>Upload documents directly into Files/Inbox. Documents that were copied, moved, or edited there are held and not read.</p>
    {problem && <OnboardingProblemNotice problem={problem} />}
    <div className="onboarding-actions">
      <button type="button" className="primary" disabled={busy} onClick={activate}>{busy ? 'Turning on filing…' : 'Turn on filing'}</button>
    </div>
  </>;
}

function FinishedStep({ heading, bridge, library, onLibrary, onComplete }: { heading: HeadingRef; bridge: DesktopBridge; library: SharePointSetupStatus; onLibrary(status: SharePointSetupStatus): void; onComplete(): void }) {
  const { busy, problem, run, mounted } = useAction();
  // Completion is recorded before the app opens; if it cannot be, the app
  // stays closed rather than pretending setup finished.
  const finish = () => void run(async () => {
    try { await bridge.completeOnboarding(); }
    catch (cause) {
      // Filing stopped being on since this step appeared (an account switch,
      // say). Asking again takes the person back to the step that fixes it.
      if (describeSharePointProblem(cause).code !== 'ONBOARDING_SETUP_INCOMPLETE') throw cause;
      const status = await bridge.getSharePointSetup();
      if (status.phase === 'active') throw cause;
      if (mounted.current) onLibrary(status);
      return;
    }
    if (mounted.current) onComplete();
  });
  return <>
    <h1 ref={heading} tabIndex={-1} className="onboarding-finished">Watching Files/Inbox. Filing your documents into Files/Filed.</h1>
    <div className="onboarding-account">
      <span>Connected Microsoft account</span>
      <strong>{library.account.displayName}</strong>
      <span>{library.account.email}</span>
    </div>
    <p>Intern starts automatically when you sign in to Windows and keeps running in the system tray when its window is closed.</p>
    <p>Only documents you upload directly into Files/Inbox are filed. Everyone else's documents are left alone.</p>
    {problem && <OnboardingProblemNotice problem={problem} />}
    <div className="onboarding-actions">
      <button type="button" className="primary" disabled={busy} onClick={finish}>Open Intern</button>
    </div>
  </>;
}
