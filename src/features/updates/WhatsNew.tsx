import { useEffect, useRef, useState } from 'react';
import { changesSince, compareVersions } from './releaseNotes';
import './whatsNew.css';

export const LAST_SEEN_VERSION_KEY = 'intern.lastSeenVersion';

async function installedVersion(): Promise<string> {
  const { getVersion } = await import('@tauri-apps/api/app');
  return getVersion();
}

/** Bundled notes work offline; acknowledgement is saved only when the person dismisses them. */
export function WhatsNew({ getVersion = installedVersion }: { getVersion?: () => Promise<string> }) {
  const [summary, setSummary] = useState<{ version: string; changes: string[] }>();
  useEffect(() => {
    let active = true;
    void getVersion().then((version) => {
      if (!active) return;
      let previous: string | undefined;
      try { previous = window.localStorage.getItem(LAST_SEEN_VERSION_KEY) || undefined; } catch { /* Still show this release when storage is unavailable. */ }
      if (previous && compareVersions(version, previous) <= 0) return;
      const changes = changesSince(previous, version);
      if (changes.length) setSummary({ version, changes });
    }).catch(() => { /* A version lookup failure must not interfere with documents or acknowledge unseen notes. */ });
    return () => { active = false; };
  }, [getVersion]);

  if (!summary) return null;
  const dismiss = () => {
    try { window.localStorage.setItem(LAST_SEEN_VERSION_KEY, summary.version); } catch { /* Dismiss for this launch even when storage is unavailable. */ }
    setSummary(undefined);
  };
  return <SummaryDialog changes={summary.changes} onDismiss={dismiss} />;
}

function SummaryDialog({ changes, onDismiss }: { changes: string[]; onDismiss(): void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const node = dialog.current;
    node?.showModal();
    return () => node?.close();
  }, []);
  return <dialog ref={dialog} className="settings-dialog whats-new" aria-labelledby="whats-new-title"
    onKeyDown={(event) => event.stopPropagation()}
    onCancel={(event) => { event.preventDefault(); onDismiss(); }}>
    <h2 id="whats-new-title">What’s new in Intern</h2>
    <ul>{changes.map((change) => <li key={change}>{change}</li>)}</ul>
    <button type="button" className="primary" autoFocus onClick={onDismiss}>Continue</button>
  </dialog>;
}
