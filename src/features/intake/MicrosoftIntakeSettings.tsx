import type { DesktopBridge } from '../../lib/bridge';
import { useMicrosoftSignIn } from './useMicrosoftSignIn';

export function MicrosoftIntakeSettings({ bridge, savedFolder, unsavedFolder }: { bridge: DesktopBridge; savedFolder: string; unsavedFolder: boolean }) {
  const { status, prompt, busy, error, run, refresh, begin, disconnect } = useMicrosoftSignIn(bridge);
  const available = Boolean(bridge.microsoftIntakeStatus && bridge.microsoftSignInStart && bridge.microsoftSignInPoll && bridge.microsoftDisconnect && bridge.microsoftBindIntake && !status?.error);

  const documents = status?.documents ?? [];
  const counts = {
    verified: documents.filter((item) => ['verified', 'processed', 'filed'].includes(item.state)).length,
    other: documents.filter((item) => item.state === 'other').length,
    unknown: documents.filter((item) => item.state === 'unknown').length,
  };
  return <div className="microsoft-intake" role="group" aria-label="Microsoft upload identity">
    <div className="identity-heading"><h4>Microsoft upload identity</h4><span className="identity-policy">Unknown uploader = held</span></div>
    <p className="section-lead">Unverified uploads are never processed. Intern checks Microsoft's own file metadata for who created the document and who last modified it, never a typed name or the computer that synced it first.</p>
    {!available && <p className="check-hint">Microsoft account connection is available in the installed desktop app. This browser preview cannot verify any uploads.</p>}
    {status?.error && <p className="form-error" role="status" aria-label="Microsoft connection status">{status.error}</p>}
    {status?.connected ? <div className="identity-account">
      <p className="field-label">Processing for</p>
      <strong>{status.account?.displayName ?? 'Microsoft account awaiting revalidation'}</strong>
      {status.account && <span>{status.account.email}</span>}
      <div className="update-actions"><button type="button" disabled={busy} onClick={() => void run(disconnect)}>Disconnect Microsoft</button></div>
    </div> : <>
      <p className="identity-permissions" role="note">This connection reads your Microsoft profile and file metadata and keeps sign-in available after restart. Intern never downloads SharePoint document content through Microsoft Graph; document analysis stays local unless you separately choose a hosted model.</p>
      {!prompt && <div className="update-actions"><button type="button" className="primary" disabled={!available || busy} onClick={begin}>Connect my Microsoft account</button></div>}
    </>}
    {prompt && <div className="identity-signin" role="status" aria-label="Microsoft sign-in">
      <p>Open Microsoft's sign-in page and enter this code:</p><strong className="identity-code">{prompt.userCode}</strong>
      <p>microsoft.com/devicelogin</p>
      <div className="update-actions"><button type="button" disabled={busy || !bridge.microsoftOpenSignIn} onClick={() => void run(async () => { await bridge.microsoftOpenSignIn?.(); })}>Open Microsoft sign-in</button><button type="button" disabled={busy} onClick={() => void run(disconnect)}>Cancel sign-in</button></div>
      <p className="check-hint">Waiting for Microsoft. Files remain held until sign-in and upload verification both succeed.</p>
    </div>}
    <div className="identity-folder">
      <h4>2. Pair the intake folder</h4>
      <p className="check-hint">Save the local intake folder first. Intern verifies it against the provisioned Files/Inbox library; matching folder names alone are not proof.</p>
      {unsavedFolder && <p className="check-hint">Save your changed intake folder before pairing it.</p>}
      <div className="update-actions"><button type="button" disabled={!available || busy || !status?.connected || !savedFolder || unsavedFolder} onClick={() => void run(async () => { await bridge.microsoftBindIntake?.(); await refresh(); })}>Verify folder pairing</button></div>
      {status?.binding && <p className="check-hint" role="status" aria-label="Microsoft folder pairing">Paired: {status.binding.localFolder}<br /><span>{status.binding.webUrl}</span></p>}
      <p className="check-hint">Work or school accounts only. New, unchanged uploads are supported; moved, edited, conflicting, and unverified files stay held.</p>
    </div>
    {documents.length > 0 && <section className="identity-documents" aria-label="Upload verification activity">
      <h4>3. Uploads and processing</h4>
      <p role="status" aria-label="Uploader counts">Recent checks: {counts.verified} verified · {counts.other} belonging to others · {counts.unknown} uploader unknown</p>
      <ul>{documents.map((item) => <li key={item.path}>
        <strong>{item.filename}</strong><span className="identity-state">{item.state === 'unknown' ? 'Held: uploader unknown' : item.state === 'other' ? 'Held for another account' : item.state === 'filed' ? 'Filed' : item.state === 'processed' ? 'Processed' : 'Uploader verified'}</span>
        <p>{item.reason}</p>
        {item.uploader && <p>Uploaded by <strong>{item.uploader.displayName}</strong>{item.uploader.email ? ` (${item.uploader.email})` : ''}</p>}
        <p>{item.processedBy ? `Processed by ${item.processedBy.displayName}${item.processedBy.email ? ` (${item.processedBy.email})` : ''}` : 'Not processed by this installation.'}</p>
        {item.filedAs && <p>Filed as {item.filedAs}</p>}
      </li>)}</ul>
    </section>}
    {error && <p className="form-error" role="alert">{error}</p>}
  </div>;
}
