import { useEffect, useState } from 'react';
import type { DescriptionsEventSource, DesktopBridge } from '../../lib/bridge';
import type { DescriptionsStatus } from '../../types';
import { useSupportLink } from '../sharepoint/useSupportLink';

interface Props {
  bridge: DesktopBridge;
  /** The setting as saved: what the backend is doing now, and what backfill requires. */
  savedEnabled: boolean;
  /** The setting in the unsaved draft, which the checkbox edits. */
  draftEnabled: boolean;
  onDraftEnabledChange: (enabled: boolean) => void;
}

function formatRecordedTime(at: number | null): string {
  return at === null ? 'never' : new Date(at * 1000).toLocaleString();
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

function statusLine(status: DescriptionsStatus): string {
  if (!status.enabled) return 'Off · no records are being written';
  return [
    'On',
    status.folder ? `writing to ${status.folder}` : 'no output folder yet',
    `${plural(status.recordedThisSession, 'record', 'records')} written since Intern started`,
    `Last recorded: ${formatRecordedTime(status.lastRecordedAt)}`,
  ].join(' · ');
}

function describeDescriptionsError(error: string): string {
  // The backend keeps the code and the operating system's own words; the
  // first is for the guide's troubleshooting table, the second says what
  // actually happened (a folder that is read-only, a share that went away).
  return error.startsWith('DESCRIPTION_RETRACT_FAILED')
    ? `The record for an undone rename could not be removed. ${error}`
    : `The last description record could not be written. ${error}`;
}

/** Why a backfill was refused, in a sentence that says what to do first. */
function backfillFailure(error: unknown): string {
  const code = typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code.trim() : undefined;
  if (code === 'DESCRIPTIONS_DISABLED') return 'Turn on description records and save settings first; then records can be written for documents already filed. (DESCRIPTIONS_DISABLED)';
  if (code === 'DESCRIPTIONS_NEED_DESTINATION') return 'Description records live in the output folder, so choose an output folder first. (DESCRIPTIONS_NEED_DESTINATION)';
  const message = error instanceof Error ? error.message : typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' ? error.message : typeof error === 'string' ? error : '';
  return message.trim() ? `Records could not be written: ${message.trim()}${code ? ` (${code})` : ''}` : 'Records could not be written.';
}

/**
 * Settings → Descriptions for SharePoint.
 *
 * The sentence Intern writes about a document used to live only in the queue.
 * A SharePoint library has a column for it, so Intern leaves a small record
 * beside each filed document, the OneDrive sync client uploads it like any
 * other file, and a Power Automate flow - running in the person's own
 * Microsoft 365 account, under their own SharePoint connection - copies the
 * sentence into the column. Intern itself makes no network request for any of
 * it, which is the reason for the whole arrangement: a direct Microsoft Graph
 * writer would mean a sign-in, a stored token, and a request nobody asked for.
 *
 * So this section is the setting, a status line saying whether records are
 * being written and where, and four steps to get the flow running. The flow is
 * downloaded from the guide, opened in the person's browser through the same
 * fixed-link path every other help link takes; the app never fetches it.
 *
 * The section owns its own status subscription and backfill so that it can
 * sit in the dialog as one self-contained piece.
 */
export function SharePointDescriptions({ bridge, savedEnabled, draftEnabled, onDraftEnabledChange }: Props) {
  const [status, setStatus] = useState<DescriptionsStatus>();
  const [backfilling, setBackfilling] = useState(false);
  const [backfillMessage, setBackfillMessage] = useState('');
  const [backfillError, setBackfillError] = useState('');
  const flowLink = useSupportLink(bridge);

  useEffect(() => {
    let active = true;
    // A convenience: a bridge without it (an older desktop build, a test
    // double) leaves the status line out and the steps still work.
    void Promise.resolve().then(() => bridge.descriptionsStatus())
      .then((current) => { if (active) setStatus(current); })
      .catch(() => { /* The section still renders without a status. */ });
    const source = bridge as DesktopBridge & Partial<DescriptionsEventSource>;
    const stop = source.subscribeDescriptions?.((current) => { if (active) setStatus(current); });
    return () => { active = false; stop?.(); };
  }, [bridge]);

  const runBackfill = async () => {
    if (backfilling) return;
    setBackfilling(true);
    setBackfillError('');
    setBackfillMessage('');
    try {
      const result = await bridge.descriptionsBackfill();
      setBackfillMessage(result.failed
        ? `${plural(result.written, 'record', 'records')} written; ${result.failed} could not be written.`
        : `${plural(result.written, 'record', 'records')} written.`);
      setStatus(await bridge.descriptionsStatus());
    } catch (error) {
      setBackfillError(backfillFailure(error));
    } finally {
      setBackfilling(false);
    }
  };

  return (
    <section className="settings-group" aria-labelledby="sharepoint-descriptions-heading">
      <h3 id="sharepoint-descriptions-heading">Descriptions for SharePoint</h3>
      <p className="section-lead">Puts Intern's one-sentence description of each filed document in your SharePoint library's Description column.</p>
      <label className="check-label"><input type="checkbox" checked={draftEnabled} onChange={(event) => onDraftEnabledChange(event.target.checked)} />Write a description record for each filed document</label>
      <p className="check-hint">Each record is a small file in <code>.intern\descriptions</code> inside the output folder. OneDrive uploads it with the document; Intern itself sends nothing.</p>
      {status && <div className="descriptions-status">
        <p role="status" aria-label="Description records status" aria-live="polite">{statusLine(status)}</p>
        {status.lastError && <p className="form-error" role="alert">{describeDescriptionsError(status.lastError)}</p>}
      </div>}
      <h4 className="descriptions-steps-title">Fill your library's Description column automatically</h4>
      <ol className="descriptions-steps" aria-label="Fill your library's Description column automatically">
        <li>In the library, add a column named <strong>Description</strong> (Multiple lines of text).</li>
        <li>
          Get the flow.{' '}
          <button type="button" onClick={() => flowLink.open('description-flow')}>Open the flow download</button>
          <span className="check-hint">Opens the Intern guide in your browser; the flow is a small .zip file there.</span>
          {flowLink.error && <span className="form-error" role="alert">{flowLink.error}</span>}
        </li>
        <li>In Power Automate, choose <strong>My flows → Import → Import Package (Legacy)</strong>, upload it, and pick your SharePoint connection. Then open the flow, choose your site and library in its steps, and save. It runs in your Microsoft 365 account, not in Intern.</li>
        <li>
          Fill in documents filed before records were on.{' '}
          {/*
            Only once the saved settings have records on: the backend refuses
            otherwise, and an enabled button that always fails is worse than a
            hint saying what to do first.
          */}
          <button type="button" disabled={backfilling || !savedEnabled} onClick={() => void runBackfill()}>{backfilling ? 'Writing…' : 'Write records for documents already filed'}</button>
          {!savedEnabled && <span className="check-hint">Save with records on first.</span>}
          {backfillMessage && <span role="status" aria-label="Backfill status" aria-live="polite">{backfillMessage}</span>}
          {backfillError && <span className="form-error" role="alert">{backfillError}</span>}
        </li>
      </ol>
    </section>
  );
}
