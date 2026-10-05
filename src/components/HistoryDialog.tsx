import { ExternalLink, FolderOpen, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { describeActionError } from '../lib/actionErrors';
import type { DesktopBridge, SelectionBoundary } from '../lib/bridge';
import type { HistoryEntry } from '../types';
import { Icon } from './Icon';

interface Props {
  bridge: DesktopBridge;
  selection?: SelectionBoundary;
  /**
   * Queue items that are filed and still in the queue. Their documents can be
   * opened or shown from the history; one cleared from the queue has no
   * record left to say where it is now.
   */
  filedItems?: ReadonlySet<string>;
  onClose(): void;
}

function describeHistoryError(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  if (typeof error === 'string' && error.trim()) return error.trim();
  return 'The rename history could not be read.';
}

function actionLabel(entry: HistoryEntry): string {
  if (entry.stage === 'rolled_back') return entry.direction === 'undo' ? 'Undo rolled back' : 'Rename rolled back';
  if (entry.direction === 'undo') return 'Undone';
  return entry.kind === 'verified_copy' ? 'Renamed (copied across drives)' : 'Renamed';
}

function pathLeaf(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function formatWhen(at: number): string {
  return new Date(at * 1000).toLocaleString();
}

/**
 * Modal history of finished renames and undos, with CSV export. Same
 * backdrop/focus-trap conventions as SettingsDialog: Escape closes, Tab
 * cycles within the dialog.
 */
export function HistoryDialog({ bridge, selection, filedItems, onClose }: Props) {
  const [entries, setEntries] = useState<HistoryEntry[]>();
  const [loadError, setLoadError] = useState('');
  const [openError, setOpenError] = useState('');
  const [exporting, setExporting] = useState(false);
  const [exportMessage, setExportMessage] = useState('');
  const [exportError, setExportError] = useState('');
  const dialog = useRef<HTMLElement>(null);
  const initialFocus = useRef<HTMLButtonElement>(null);
  // Escape reaches the latest onClose through a ref, so the focus effect below
  // does not have to depend on its identity.
  const close = useRef(onClose);
  useEffect(() => { close.current = onClose; }, [onClose]);

  useEffect(() => {
    let active = true;
    bridge.historyList()
      .then((listed) => { if (active) setEntries(listed); })
      .catch((error) => { if (active) { setEntries([]); setLoadError(describeHistoryError(error)); } });
    return () => { active = false; };
  }, [bridge]);

  // Focus Close once, when the dialog opens. Depending on `onClose` here meant
  // refocusing on every App render, and App rerenders on every queue event.
  useEffect(() => {
    initialFocus.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { event.preventDefault(); close.current(); return; }
      if (event.key !== 'Tab') return;
      const focusable = Array.from(dialog.current?.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled])') ?? []);
      if (!focusable.length) return;
      const current = document.activeElement;
      const index = focusable.indexOf(current as HTMLElement);
      const nextIndex = event.shiftKey ? (index <= 0 ? focusable.length - 1 : index - 1) : (index === focusable.length - 1 ? 0 : index + 1);
      if (index !== -1) { event.preventDefault(); focusable[nextIndex].focus(); }
    };
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, []);

  const runExport = async () => {
    if (exporting) return;
    setExporting(true);
    setExportError('');
    setExportMessage('');
    try {
      // The native save dialog lives at the selection boundary like every
      // other picker; without one (browser dev, tests) the in-memory export
      // ignores the path anyway.
      const path = selection?.pickHistoryExportPath
        ? await selection.pickHistoryExportPath()
        : 'intern-history.csv';
      if (!path) return;
      const count = await bridge.historyExport(path);
      setExportMessage(`Exported ${count} ${count === 1 ? 'operation' : 'operations'}.`);
    } catch (error) {
      setExportError(describeHistoryError(error));
    } finally {
      setExporting(false);
    }
  };

  // Open and Show in folder reach the item's document where it is now, so
  // they belong on its newest row only: an older row describes a rename the
  // document has since moved on from.
  const newest = new Set<string>();
  const openable = new Set<string>();
  for (const entry of entries ?? []) {
    if (newest.has(entry.queueItemId)) continue;
    newest.add(entry.queueItemId);
    if (filedItems?.has(entry.queueItemId)) openable.add(entry.receiptId);
  }
  const openDocument = async (id: string, reveal: boolean) => {
    setOpenError('');
    try { await (reveal ? bridge.revealItem(id) : bridge.openItem(id)); }
    catch (error) { setOpenError(describeActionError(error)); }
  };

  return <div className="dialog-backdrop" role="presentation"><section ref={dialog} className="settings-dialog history-dialog" role="dialog" aria-modal="true" aria-label="Rename history">
    <div className="dialog-head"><h2>Rename history</h2><button type="button" className="icon-button" onClick={onClose} aria-label="Close history"><Icon icon={X} /></button></div>
    <p className="update-note">Every rename and undo Intern has applied, newest first. The CSV export carries each document's description in its last column.</p>
    {loadError && <p className="form-error" role="alert">{loadError}</p>}
    {entries && entries.length === 0 && !loadError && <p className="history-empty">No renames yet.</p>}
    {entries && entries.length > 0 && <div className="history-table-wrap"><table>
      <thead><tr><th scope="col">When</th><th scope="col">Action</th><th scope="col">Original name</th><th scope="col">New name</th></tr></thead>
      <tbody>{entries.map((entry) => <tr key={entry.receiptId}>
        <td>{formatWhen(entry.at)}</td>
        <td>{actionLabel(entry)}</td>
        <td title={entry.originalPath}>{pathLeaf(entry.originalPath)}</td>
        <td title={entry.newPath}>{pathLeaf(entry.newPath)}{entry.description && <span className="row-description">{entry.description}</span>}
          {openable.has(entry.receiptId) && <span className="history-document-actions">
            <button type="button" className="link-button" aria-label={`Open ${pathLeaf(entry.newPath)}`} onClick={() => void openDocument(entry.queueItemId, false)}><Icon icon={ExternalLink} />Open</button>
            <button type="button" className="link-button" aria-label={`Show ${pathLeaf(entry.newPath)} in its folder`} onClick={() => void openDocument(entry.queueItemId, true)}><Icon icon={FolderOpen} />Show in folder</button>
          </span>}
        </td>
      </tr>)}</tbody>
    </table></div>}
    {openError && <p className="form-error" role="alert" aria-label="Open error">{openError}</p>}
    {exportMessage && <p role="status" aria-label="Export status" aria-live="polite">{exportMessage}</p>}
    {exportError && <p className="form-error" role="alert">{exportError}</p>}
    <div className="history-actions">
      <button type="button" disabled={exporting || !entries?.length} onClick={() => void runExport()}>{exporting ? 'Exporting…' : 'Export CSV…'}</button>
      <button ref={initialFocus} type="button" className="primary" onClick={onClose}>Close</button>
    </div>
  </section></div>;
}
