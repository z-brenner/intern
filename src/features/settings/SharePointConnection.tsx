import { useCallback, useEffect, useRef, useState } from 'react';
import type { DesktopBridge } from '../../lib/bridge';
import type { AppSettings, IntakeStatus, SharePointSetupStatus } from '../../types';
import { useMicrosoftSignIn } from '../intake/useMicrosoftSignIn';
import { describeSharePointProblem } from '../sharepoint/sharePointProblems';
import type { SharePointProblem } from '../sharepoint/sharePointProblems';
import { useSupportLink } from '../sharepoint/useSupportLink';

/*
  The library is fixed for this deployment, so these are facts to show rather
  than choices to offer. They match the literal types the backend reports.
*/
const SITE: SharePointSetupStatus['site'] = 'InternTestSite';
const LIBRARY: SharePointSetupStatus['library'] = 'Files';
const INBOX: SharePointSetupStatus['intake'] = 'Inbox';
const FILED: SharePointSetupStatus['destination'] = 'Filed';

function setupSentence(setup: SharePointSetupStatus | undefined, failure: SharePointProblem | undefined): string {
  if (failure) return `Needs attention. ${failure.action}`;
  if (!setup) return 'Checking the SharePoint connection…';
  switch (setup.phase) {
    case 'active': return `Active. Intern watches ${INBOX} and files into ${FILED}.`;
    case 'ready_to_activate': return 'Needs attention. Filing is not on for this computer and Microsoft account. Turn on filing to finish setup.';
    case 'enrollment_pending': return `Needs attention. The ${LIBRARY} library is not synced on this computer yet. Sync it with OneDrive, keep OneDrive running while it syncs, then check again.`;
  }
}

function watcherSentence(intake: IntakeStatus | undefined): string {
  if (!intake) return 'Checking the Inbox watcher…';
  if (intake.error) return 'Needs attention. The Inbox watcher reported a problem; check again, and see Support details if it continues.';
  if (!intake.watching) return `Not watching ${INBOX}. Check again, or reconnect Microsoft if this continues.`;
  return `Watching ${INBOX} · Last scan ${intake.lastScanAt === null ? 'not yet' : new Date(intake.lastScanAt * 1000).toLocaleTimeString()}`;
}

function onOff(value: boolean): string {
  return value ? 'On' : 'Off';
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/**
 * The read-only SharePoint connection card shown in Settings when this build
 * carries the fixed deployment. It offers no way to point Intern elsewhere:
 * the actions re-check the connection, sign in to Microsoft again, or finish
 * the fixed setup (ask OneDrive to sync, turn on filing) when the backend
 * says it is unfinished. A reconnect is always followed by the backend's own
 * setup check, so a different organization's account is reported rather than
 * adopted, and a different account from the same one is asked to turn filing
 * on for itself.
 */
export function SharePointConnection({ bridge, settings }: { bridge: DesktopBridge; settings: AppSettings }) {
  const [setup, setSetup] = useState<SharePointSetupStatus>();
  const [failure, setFailure] = useState<SharePointProblem>();
  const [intake, setIntake] = useState<IntakeStatus>();
  const [checking, setChecking] = useState(false);
  const [checked, setChecked] = useState('');
  const [finishing, setFinishing] = useState<'sync' | 'activate'>();
  const [finishFailure, setFinishFailure] = useState<SharePointProblem>();
  const support = useSupportLink(bridge);
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);

  // Each check replaces the last; the latest one started is the one shown.
  const checkGeneration = useRef(0);
  const check = useCallback(async (): Promise<SharePointProblem | undefined> => {
    const version = ++checkGeneration.current;
    const [setupResult, intakeResult] = await Promise.allSettled([bridge.getSharePointSetup(), bridge.intakeStatus()]);
    if (!mounted.current || checkGeneration.current !== version) return undefined;
    if (intakeResult.status === 'fulfilled') setIntake(intakeResult.value);
    if (setupResult.status === 'fulfilled') { setSetup(setupResult.value); setFailure(undefined); return undefined; }
    const next = describeSharePointProblem(setupResult.reason);
    setSetup(undefined); setFailure(next);
    return next;
  }, [bridge]);

  const microsoft = useMicrosoftSignIn(bridge, async () => {
    const next = await check();
    // The account just signed in belongs to another organization: disconnect
    // it rather than leave it connected, and let the alert ask for the right one.
    if (next?.code === 'MICROSOFT_ACCOUNT_WRONG_TENANT') { await bridge.microsoftDisconnect?.(); await microsoft.refresh(); }
    else if (!next && mounted.current) setChecked('Reconnected. The SharePoint connection was checked again.');
  });

  useEffect(() => { void check(); }, [check]);

  const runCheck = async () => {
    if (checking) return;
    setChecking(true); setChecked(''); setFinishFailure(undefined);
    try {
      await Promise.all([check(), microsoft.refresh().catch(() => {})]);
      if (mounted.current) setChecked('Checked just now.');
    } finally { if (mounted.current) setChecking(false); }
  };
  // Finishing setup here runs the same backend steps onboarding does, then
  // asks again, so the card shows where setup really stands afterwards.
  const finishInFlight = useRef(false);
  const finish = async (kind: 'sync' | 'activate') => {
    if (finishInFlight.current || busy) return;
    finishInFlight.current = true;
    setFinishing(kind); setFinishFailure(undefined); setChecked('');
    try { await (kind === 'sync' ? bridge.startSharePointSync() : bridge.activateOnboarding()); }
    catch (cause) { if (mounted.current) setFinishFailure(describeSharePointProblem(cause)); }
    finally {
      await check();
      finishInFlight.current = false;
      if (mounted.current) setFinishing(undefined);
    }
  };
  const reconnect = () => { setChecked(''); setFinishFailure(undefined); microsoft.begin(); };
  const cancel = () => void microsoft.run(async () => { await microsoft.disconnect(); await check(); });

  const account = microsoft.status?.connected ? microsoft.status.account : null;
  const documents = microsoft.status?.documents ?? [];
  const verified = documents.filter((item) => ['verified', 'processed', 'filed'].includes(item.state)).length;
  const held = documents.filter((item) => item.state === 'other' || item.state === 'unknown').length;
  const busy = checking || microsoft.busy || Boolean(finishing);
  // SharePoint's own Sync button is the manual way to finish a library that has not synced.
  const offerSharePoint = Boolean(failure) || setup?.phase === 'enrollment_pending';
  const problem = failure ?? finishFailure;

  return <section className="settings-group sharepoint-connection" aria-labelledby="sharepoint-connection-heading">
    <h3 id="sharepoint-connection-heading">SharePoint connection</h3>
    <p className="section-lead">Intern files documents for this organization's SharePoint library. The location is set for you and cannot be changed here.</p>
    <dl className="connection-facts">
      <div><dt>Site</dt><dd>{SITE}</dd></div>
      <div><dt>Library</dt><dd>{LIBRARY}</dd></div>
      <div><dt>Watches</dt><dd>{INBOX}</dd></div>
      <div><dt>Files into</dt><dd>{FILED}</dd></div>
      <div className="connection-account"><dt>Microsoft account</dt><dd>{account
        ? <><strong>{account.displayName}</strong><span>{account.email}</span></>
        : microsoft.status ? 'Not connected' : 'Checking…'}</dd></div>
    </dl>
    <div className="connection-health">
      <p role="status" aria-label="SharePoint setup" aria-live="polite">{setupSentence(setup, failure)}</p>
      <p role="status" aria-label="Watcher health" aria-live="polite">{watcherSentence(intake)}</p>
      <p role="status" aria-label="Background status">Runs in the background: {onOff(settings.runInBackground)} · Starts when you sign in: {onOff(settings.startAtLogin)}</p>
    </div>
    {failure && <p className="form-error" role="alert">{failure.action}</p>}
    {finishFailure && <p className="form-error" role="alert">{finishFailure.action}</p>}
    {microsoft.error && <p className="form-error" role="alert">{microsoft.error}</p>}
    {support.error && <p className="form-error" role="alert">{support.error}</p>}
    {checked && <p className="check-hint" role="status" aria-label="Connection check">{checked}</p>}
    {microsoft.prompt && <div className="identity-signin" role="status" aria-label="Microsoft sign-in">
      <p>Open Microsoft's sign-in page and enter this code:</p><strong className="identity-code">{microsoft.prompt.userCode}</strong>
      <p>microsoft.com/devicelogin</p>
      <div className="update-actions">
        <button type="button" disabled={microsoft.busy || !bridge.microsoftOpenSignIn} onClick={() => void microsoft.run(async () => { await bridge.microsoftOpenSignIn?.(); })}>Open Microsoft sign-in</button>
        <button type="button" disabled={microsoft.busy} onClick={cancel}>Cancel sign-in</button>
      </div>
      <p className="check-hint">Sign in with your work account. Documents stay untouched until the sign-in finishes and the connection is checked.</p>
    </div>}
    <div className="update-actions">
      {setup?.phase === 'enrollment_pending' && !microsoft.prompt && <button type="button" className="primary" disabled={busy} onClick={() => void finish('sync')}>{finishing === 'sync' ? 'Asking OneDrive…' : 'Sync Files with OneDrive'}</button>}
      {setup?.phase === 'ready_to_activate' && !microsoft.prompt && <button type="button" className="primary" disabled={busy} onClick={() => void finish('activate')}>{finishing === 'activate' ? 'Turning on filing…' : 'Turn on filing'}</button>}
      <button type="button" disabled={busy} onClick={() => void runCheck()}>{checking ? 'Checking…' : 'Check again'}</button>
      {!microsoft.prompt && <button type="button" disabled={busy || !bridge.microsoftSignInStart} onClick={reconnect}>Reconnect Microsoft</button>}
      {offerSharePoint && <button type="button" onClick={() => support.open('sharepoint-site')}>Open SharePoint</button>}
      {problem?.getOneDrive && <button type="button" onClick={() => support.open('onedrive-download')}>Get OneDrive</button>}
    </div>
    {!microsoft.prompt && <p className="check-hint">Reconnecting signs out the current Microsoft account until the new sign-in finishes.</p>}
    <details className="support-details" role="group" aria-label="Support details">
      <summary>Support details</summary>
      <dl>
        <div><dt>{failure ? 'Error code' : 'Setup phase'}</dt><dd><code>{failure ? failure.code : setup?.phase ?? 'checking'}</code></dd></div>
        {failure?.detail && <div><dt>Setup message</dt><dd>{failure.detail}</dd></div>}
        {finishFailure && <div><dt>Last setup action</dt><dd><code>{finishFailure.code}</code>{finishFailure.detail && <> · {finishFailure.detail}</>}</dd></div>}
        {microsoft.status?.error && <div><dt>Microsoft status</dt><dd>{microsoft.status.error}</dd></div>}
        {account && <div><dt>Account ID</dt><dd><code>{account.id}</code></dd></div>}
        {account && <div><dt>Tenant ID</dt><dd><code>{account.tenantId}</code></dd></div>}
        {microsoft.status?.binding && <div><dt>Paired folder</dt><dd>{microsoft.status.binding.localFolder}<br />{microsoft.status.binding.webUrl}</dd></div>}
        {intake && <>
          <div><dt>Watched folder</dt><dd>{intake.folder || 'none'}</dd></div>
          <div><dt>This computer</dt><dd>{intake.machineName}</dd></div>
          <div><dt>Last scan</dt><dd>{intake.lastScanAt === null ? 'not yet' : new Date(intake.lastScanAt * 1000).toLocaleString()}</dd></div>
          <div><dt>Scan results</dt><dd>{[
            `${intake.processedHere} processed here`,
            `${intake.heldForOthers} held for others`,
            `${intake.uploaderUnknown ?? 0} uploader unknown`,
            plural(intake.syncConflicts, 'sync conflict', 'sync conflicts'),
            `${intake.awaitingHydration} waiting for OneDrive to download`,
            plural(intake.unreadableFolders, 'unreadable subfolder', 'unreadable subfolders'),
          ].join(' · ')}</dd></div>
          {intake.error && <div><dt>Watcher error</dt><dd><code>{intake.error}</code></dd></div>}
        </>}
        <div><dt>Recent upload checks</dt><dd>{verified} verified · {held} held</dd></div>
      </dl>
      {documents.length > 0 && <ul className="support-documents" aria-label="Recent upload checks">{documents.map((item) => <li key={item.path}>
        <strong>{item.filename}</strong> <span className="identity-state">{item.state}</span>
        <span>{item.reason}</span>
      </li>)}</ul>}
    </details>
  </section>;
}
