import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { AppHeader } from './components/AppHeader';
import { DropZone } from './components/DropZone';
import { FolderSetupFlow } from './components/FolderSetupFlow';
import { HistoryDialog } from './components/HistoryDialog';
import { OnboardingFlow, OnboardingProblemNotice } from './components/OnboardingFlow';
import { QueueTable } from './components/QueueTable';
import { ReviewInspector } from './components/ReviewInspector';
import { SettingsDialog } from './components/SettingsDialog';
import { SetupScreen } from './components/SetupScreen';
import { Sidebar } from './components/Sidebar';
import { Toast } from './components/Toast';
import { ViewEmpty } from './components/ViewEmpty';
import { GUIDE_URL } from './lib/bridge';
import { describeActionError } from './lib/actionErrors';
import type { RefusedAction } from './lib/actionErrors';
import { describeAddReport } from './lib/addReport';
import { installingLabel } from './lib/format';
import { describeQueueStop } from './lib/reasons';
import type { DesktopBridge, LaunchReportSource, SelectionBoundary, SelectionResult, UpdateProgressListener, UpdateStatus } from './lib/bridge';
import { createInMemoryBridge } from './lib/inMemoryBridge';
import type { DragState, TauriSelectionBoundary } from './lib/tauriBridge';
import { useMediaQuery } from './lib/useMediaQuery';
import { describeSharePointProblem } from './features/sharepoint/sharePointProblems';
import type { SharePointProblem } from './features/sharepoint/sharePointProblems';
import { useQueue } from './features/queue/useQueue';
import { isParked, itemActions, nextUndecided, undecidedOrder } from './features/review/actions';
import { useReviewShortcuts } from './features/review/useReviewShortcuts';
import type { ReviewInspectorHandle } from './features/review/useReviewShortcuts';
import { modelReady, useModelSetup } from './features/setup/useModelSetup';
import type { AddReport, AppSettings, QueueItem, QueueView, SetupState } from './types';

type Gate =
  | { kind: 'loading' }
  | { kind: 'failed'; problem: SharePointProblem }
  | { kind: 'onboarding' | 'folder-setup' | 'app'; pendingSettings?: Promise<AppSettings>; pendingSetup?: Promise<SetupState>; initialSetup?: SetupState };

export function App({ bridge: suppliedBridge, selection }: { bridge?: DesktopBridge; selection?: SelectionBoundary }) {
  // The demo pushes its own changes, as the desktop backend does, so an item
  // sent back to be analyzed again is seen coming back.
  const bridgeRef = useRef<DesktopBridge>(suppliedBridge ?? createInMemoryBridge({ liveEvents: true }));
  const bridge = suppliedBridge ?? bridgeRef.current;
  // The built-in demo bridge carries no SharePoint deployment, so it opens
  // straight into the app exactly as it always has.
  const [gate, setGate] = useState<Gate>(suppliedBridge ? { kind: 'loading' } : { kind: 'app' });
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    if (!suppliedBridge) return;
    let active = true;
    // Settings and setup are read alongside the onboarding check rather than
    // after it; whichever screen opens picks up the same reads.
    const pendingSettings = bridge.getSettings();
    const pendingSetup = bridge.getSetup();
    // Each consumer reports its own failure; settling here only waits for it,
    // so the first screen shown already knows the model's state.
    const settled = Promise.allSettled([pendingSettings, pendingSetup]);
    void Promise.all([bridge.getOnboarding(), settled]).then(([status, [settings, setup]]) => {
      if (!active) return;
      // SharePoint onboarding exists only for an enabled deployment; a
      // required flag alone must not strand anyone. Without one, a first run
      // that watches no folder yet is offered folder setup once - and only
      // when the settings could be read, so nobody is asked to redo a folder
      // Intern merely failed to load.
      const folderSetup = !status.sharePointAvailable && status.completedVersion < status.currentVersion
        && settings.status === 'fulfilled' && !settings.value.intakeEnabled;
      setGate({ kind: status.required && status.sharePointAvailable ? 'onboarding' : folderSetup ? 'folder-setup' : 'app', pendingSettings, pendingSetup, initialSetup: setup.status === 'fulfilled' ? setup.value : undefined });
    }).catch((error: unknown) => {
      if (active) setGate({ kind: 'failed', problem: describeSharePointProblem(error) });
    });
    return () => { active = false; };
  }, [bridge, suppliedBridge, attempt]);

  // Its own landmark name: the screen that follows may be setup or the app, and
  // this placeholder must not be mistaken for either.
  if (gate.kind === 'loading') return <main className="setup-screen" aria-label="Starting Intern"><section><p role="status" aria-label="Loading setup" aria-live="polite">Loading setup</p></section></main>;
  if (gate.kind === 'failed') return <main className="setup-screen" aria-label="Intern setup"><section>
    <h1>Intern could not open</h1>
    <OnboardingProblemNotice problem={gate.problem} />
    <div className="setup-actions"><button type="button" className="primary" onClick={() => { setGate({ kind: 'loading' }); setAttempt((count) => count + 1); }}>Try again</button></div>
  </section></main>;
  // The queue and its subscriptions are not mounted until onboarding is recorded as complete.
  // Done or skipped, it is recorded so it is offered only once; Settings can open it again.
  if (gate.kind === 'folder-setup') {
    const finish = async () => { await bridge.completeOnboarding(); setGate({ ...gate, kind: 'app' }); };
    return <FolderSetupFlow welcome bridge={bridge} selection={selection} onDone={finish} onSkip={finish} />;
  }
  if (gate.kind === 'onboarding') return <OnboardingFlow bridge={bridge} selection={selection} pendingSettings={gate.pendingSettings} pendingSetup={gate.pendingSetup} initialSetup={gate.initialSetup} onComplete={() => setGate({ kind: 'app' })} />;
  // A model that still needs setup is shown at once. A ready one is read again
  // by the app itself, as it always was, so the queue's first snapshot arrives
  // before the queue screen replaces the loading state.
  return <MainApp bridge={bridge} selection={selection} demo={!suppliedBridge} pendingSettings={gate.pendingSettings} initialSetup={modelReady(gate.initialSetup) ? undefined : gate.initialSetup} />;
}

function MainApp({ bridge, selection, demo, pendingSettings, initialSetup }: { bridge: DesktopBridge; selection?: SelectionBoundary; demo: boolean; pendingSettings?: Promise<AppSettings>; initialSetup?: SetupState }) {
  const seededSelection = useRef(false);
  const settingsTrigger = useRef<HTMLElement | null>(null);
  const historyTrigger = useRef<HTMLElement | null>(null);
  const reviewTrigger = useRef<{ element: HTMLButtonElement; itemId: string } | null>(null);
  const focusRestoreVersion = useRef(0);
  const { items, paused, setPaused, refresh, error: queueError, pipelineError, reconnect } = useQueue(bridge);
  const inspectorHandle = useRef<ReviewInspectorHandle>(null);
  // Where focus goes once the item it is meant for is on screen: its row, or
  // its name or heading in the review panel.
  const [focusRequest, setFocusRequest] = useState<{ id: string; target: 'row' | 'filename' | 'heading' }>();
  const [view, setView] = useState<QueueView>('queue');
  const [filter, setFilter] = useState('');
  const [selectedId, setSelectedId] = useState<string>();
  // Whether the selection is one a person made. Narrow, the inspector is a
  // modal drawer, and the selection seeded below is not a person's: opening
  // that drawer for it launched Intern inside a dialog nobody had asked for,
  // over an inert queue, with the caret in a proposed filename.
  const [selectedByPerson, setSelectedByPerson] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [folderSetupOpen, setFolderSetupOpen] = useState(false);
  // Until the real settings arrive these are placeholders, not the person's
  // configuration, and saving them would overwrite a destination, a watched
  // folder, and a machine name with defaults.
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const [settings, setSettings] = useState<AppSettings>({ destination: '', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: '', intakeEnabled: false, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' });
  const model = useModelSetup(bridge, selection, { initial: demo ? DEMO_SETUP : initialSetup });
  const [actionPending, setActionPending] = useState(false);
  const actionInFlight = useRef(false);
  const [actionMessage, setActionMessage] = useState('');
  // The last add's report, kept on screen while it names files that were left
  // out: the status above is read out but never seen, and a person who dropped
  // twenty-five files and sees twenty-four rows needs to see which and why.
  const [skippedNotice, setSkippedNotice] = useState('');
  const [actionError, setActionError] = useState('');
  // The toast at the foot of the queue: a batch under way, or one finished,
  // with the renames it made for Undo to put back. Each new one has a new key,
  // so its ten seconds start again rather than running on from the last.
  const [toast, setToast] = useState<{ key: number; kind: 'progress' | 'done'; text: string; undo?: string[] }>();
  const toastKey = useRef(0);
  const showToast = (kind: 'progress' | 'done', text: string, undo?: string[]) => setToast({ key: ++toastKey.current, kind, text, undo });
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>();
  // The version dismissed from the banner, so "Not now" does not reappear on
  // every later poll - but a newer release than the one dismissed still does.
  const [updateDismissed, setUpdateDismissed] = useState<string>();
  const [updateInstalling, setUpdateInstalling] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<{ fraction: number | undefined }>();
  const [updateError, setUpdateError] = useState('');
  const narrowInspector = useMediaQuery('(max-width: 1100px)');
  // Files dragged over the desktop window. The webview never sees them - the
  // runtime takes the drop - so without this nothing on screen said a drop
  // would land anywhere.
  const [drag, setDrag] = useState<DragState>({ dragging: false, count: 0 });

  useEffect(() => {
    void (pendingSettings ?? bridge.getSettings()).then((loaded) => { setSettings(loaded); setSettingsLoaded(true); }).catch(() => setSettingsLoaded(false));
  }, [bridge, pendingSettings]);
  // Once when Intern starts, and again on this timer for as long as it keeps
  // running - not a person digging into Settings, which is how a machine that
  // is never restarted stayed on a build from months ago. Nothing here
  // installs anything: a found update only ever shows a banner, and only the
  // click on it downloads and installs - still signed, still refused if it
  // is not.
  //
  // Not before the settings are read, because they can say not to: an office
  // that allows no unrequested traffic switches this off, and a check sent in
  // the moment before its own settings file loaded would be exactly that.
  // Settings that cannot be read start nothing either, since the switch could
  // be in them; the button in Settings still checks on demand. Turning the
  // switch back on checks at once and re-arms the timer.
  const automaticUpdateChecks = settingsLoaded && !settings.skipUpdateChecks;
  useEffect(() => {
    if (!automaticUpdateChecks) return;
    let active = true;
    let timer: number | undefined;
    const check = async () => {
      try {
        const status = await bridge.checkForUpdate();
        if (active) setUpdateStatus(status);
      } catch {
        // A missed check says nothing about whether an update exists; the
        // manual button in Settings still works, and the next scheduled
        // check tries again on its own.
      }
      if (active) timer = window.setTimeout(() => { void check(); }, UPDATE_POLL_INTERVAL_MS);
    };
    void check();
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, automaticUpdateChecks]);
  useEffect(() => {
    if (!seededSelection.current && items.length) {
      seededSelection.current = true;
      const firstReview = items.find((item) => item.status === 'review');
      if (firstReview) setSelectedId(firstReview.id);
    }
  }, [items]);
  const inCurrentView = (item: QueueItem) => view === 'queue' ? item.status !== 'completed' : view === 'review' ? item.status === 'review' : item.status === 'completed';
  const filtered = items.filter(inCurrentView);
  // A folder of four hundred documents is a wall of rows. The filter narrows
  // the current view by anything a person is likely to remember: the name the
  // file arrived with, the name it was given, or a word from its description.
  const query = filter.trim().toLowerCase();
  const shown = (list: QueueItem[]) => list.filter((item) => inCurrentView(item) && (!query || matchesQuery(item, query)));
  const visible = shown(items);
  const filterShown = filtered.length > FILTER_THRESHOLD || query.length > 0;
  const selected = items.find((item) => item.id === selectedId);
  const drawerOpen = Boolean(selected && selectedByPerson && narrowInspector);
  const readyItems = items.filter((item) => item.status === 'ready' && item.proposedFilename);
  // Installing an update hands Intern to the installer, which closes it. A
  // rename in its applying stage is the one piece of work that cannot be
  // canceled, and closing underneath it leaves the journal to finish the move
  // on the next launch - recoverable, but not something to start on purpose
  // while a person is watching the file move. Install waits for it instead:
  // here, before the click, and in installUpdateBetweenRenames after it.
  const renameApplying = items.some(applyingRename);
  // Only items that have not started. Anything mid-flight, awaiting a decision,
  // or already renamed is deliberately out of reach of the discard action.
  const waitingItems = items.filter((item) => item.status === 'waiting');
  const queueStatus = queueStatusAnnouncement(items, paused);
  const rowButton = (id: string) => [...document.querySelectorAll<HTMLButtonElement>('.row-select')].find((button) => button.dataset.itemId === id);
  const rowTrigger = (id: string) => { const element = rowButton(id); return element ? { element, itemId: id } : null; };
  // Wide, the panel sits after the whole queue in the tab order, so a click
  // brings focus to its heading rather than leaving it a row count of Tab
  // presses away. Narrow, the drawer takes focus itself.
  const select = (item: QueueItem, trigger: HTMLButtonElement) => {
    seededSelection.current = true; focusRestoreVersion.current += 1; reviewTrigger.current = { element: trigger, itemId: item.id }; setSelectedId(item.id); setSelectedByPerson(true);
    if (!narrowInspector) setFocusRequest({ id: item.id, target: 'heading' });
  };
  // J/K and the arrow keys: the selection moves and focus goes with it, onto
  // the row - unless the drawer is open over the rows, where it moves to the
  // item's name instead.
  const moveSelection = (item: QueueItem) => {
    seededSelection.current = true; focusRestoreVersion.current += 1; reviewTrigger.current = rowTrigger(item.id); setSelectedId(item.id);
    if (!drawerOpen) setFocusRequest({ id: item.id, target: 'row' });
  };
  // Enter on a row, and Next undecided: select it and go straight to its name.
  const openItem = (item: QueueItem) => {
    seededSelection.current = true; focusRestoreVersion.current += 1; reviewTrigger.current = rowTrigger(item.id); setSelectedId(item.id); setSelectedByPerson(true);
    setFocusRequest({ id: item.id, target: itemActions(item).approve ? 'filename' : 'heading' });
  };
  useEffect(() => {
    if (!focusRequest) return;
    setFocusRequest(undefined);
    if (focusRequest.target === 'row') rowButton(focusRequest.id)?.focus();
    else if (selected?.id === focusRequest.id) inspectorHandle.current?.focus(focusRequest.target);
  }, [focusRequest, selected?.id]);
  // Review works through what is left to decide: needing review first, then
  // ready, in table order.
  const undecided = undecidedOrder(visible);
  const nextToDecide = selected ? nextUndecided(undecided, selected.id, visible) : undefined;
  // Where focus goes back to in the queue is decided once the queue shows
  // what the action did. Decided straight after the command, as it was, the
  // row being filed was still on screen a moment before it left the view, and
  // every toolbar button was still disabled for the action in flight - so
  // focus went to <body> whenever there was no next item to go to.
  const [queueFocus, setQueueFocus] = useState<{ version: number; element?: HTMLButtonElement; itemId?: string }>();
  // `itemId`: a row to go to instead, once it is on screen - the document an
  // undo has just put back in the queue.
  const restoreQueueFocus = (itemId?: string) => {
    const invocation = reviewTrigger.current;
    reviewTrigger.current = null;
    setQueueFocus({ version: ++focusRestoreVersion.current, ...(itemId ? { itemId } : { element: invocation?.element, itemId: invocation?.itemId }) });
  };
  useEffect(() => {
    if (!queueFocus) return;
    setQueueFocus(undefined);
    const { version, element, itemId } = queueFocus;
    if (focusRestoreVersion.current !== version) return;
    const refreshedTrigger = itemId ? rowButton(itemId) : undefined;
    // Prefer the primary action by name rather than "the first button in the
    // panel". That positional fallback silently moved the moment the toolbar
    // gained a second control, sending focus to a destructive Discard button
    // instead of Apply all ready.
    const target = element?.isConnected
      ? element
      : refreshedTrigger
        ?? document.querySelector<HTMLButtonElement>('.queue-panel .queue-actions button.primary:not(:disabled)')
        ?? document.querySelector<HTMLButtonElement>('.queue-panel button:not(:disabled)');
    target?.focus();
  }, [queueFocus]);
  const closeReview = () => {
    setSelectedId(undefined);
    restoreQueueFocus();
  };
  useEffect(() => {
    if (!selectedId || selected) return;
    setSelectedId(undefined);
    restoreQueueFocus();
  }, [selected, selectedId]);
  // Promise<unknown>: some commands report what they did - discardWaiting
  // resolves with a count - and the result is not needed here. False when
  // the command failed; otherwise the queue as read after it, for a caller
  // that says what the command did - undefined when that reread failed.
  // `success` can be worked out from that read, for a command whose outcome
  // only the queue afterwards can say; `refused` says which command it was,
  // where a code means different things to different commands.
  const runQueueAction = async (run: () => Promise<unknown>, success: string | ((read?: QueueItem[]) => string), refused?: RefusedAction): Promise<false | { read?: QueueItem[] }> => {
    if (actionInFlight.current) return false;
    actionInFlight.current = true;
    setActionPending(true);
    setActionError('');
    setActionMessage('');
    // An Undo still on screen would read as undoing this action instead.
    setToast(undefined);
    try {
      try { await run(); }
      catch (error) {
        try { await refresh(); } catch { /* Preserve the original command error. */ }
        setActionError(describeActionError(error, refused));
        return false;
      }
      // The command has already happened. A reread that fails afterwards is
      // reported by the queue's own connection banner, and calling the command
      // failed would send someone looking for a file under its old name.
      let read: QueueItem[] | undefined;
      try { read = await refresh(); } catch { /* Reported as a queue connection error. */ }
      setActionMessage(typeof success === 'string' ? success : success(read));
      return { read };
    } finally {
      actionInFlight.current = false;
      setActionPending(false);
    }
  };
  // Approve, keep and remove decide an item, and review moves on to the next
  // one still to decide - selected, its name focused, and announced - rather
  // than sending focus back to the toolbar to find it again (FRONTEND_UX-10).
  // When none is left, focus goes back to the queue as before.
  const decide = async (item: QueueItem, decision: 'approve' | 'keep' | 'remove', run: () => Promise<void>) => {
    const before = undecidedOrder(visible);
    const selectionVersion = focusRestoreVersion.current;
    // Approving a parked item checks its files first, and that check can be
    // what refuses it.
    const outcome = await runQueueAction(run, '', decision === 'approve' ? { approve: true, checkedFiles: isParked(item) } : undefined);
    if (!outcome) return;
    // Only a list read after the command says what it did; when that reread
    // failed, the command still happened and the queue as last read stands.
    const fresh = outcome.read;
    const after = decision === 'approve' ? fresh?.find((entry) => entry.id === item.id)?.status : undefined;
    // An approval the backend accepted can still leave the document unrenamed:
    // it files the name between documents while the queue is busy, and sends
    // the item back to review when the file changed since it was read. Each is
    // said as it is, and an item sent back stays on screen with its reason.
    // Only review is "back": an approval being filed as the queue was read
    // shows as processing, and that is a rename under way.
    if (after === 'review') {
      setActionMessage(`${item.originalFilename} was not renamed. It needs review again.`);
      return;
    }
    const done = decision === 'keep' ? `Kept ${item.originalFilename} under its own name.`
      : decision === 'remove' ? `Removed ${item.originalFilename} from the queue.`
        : after === 'ready' ? `${item.originalFilename} will be renamed when the queue is free.`
          : after === 'processing' ? `${item.originalFilename} is being renamed.` : `Renamed ${item.originalFilename}.`;
    setActionMessage(done);
    // A rename the queue shows as filed can be put back from the toast.
    if (after === 'completed' && fresh?.find((entry) => entry.id === item.id)?.undoable) showToast('done', renamedCount(1), [item.id]);
    if (focusRestoreVersion.current !== selectionVersion) return;
    const next = nextUndecided(before, item.id, shown(fresh ?? items));
    if (!next) {
      setSelectedId(undefined);
      restoreQueueFocus();
      return;
    }
    // A selection change of its own, so the focus restore that a disappearing
    // row would otherwise start does not pull focus back to the toolbar.
    focusRestoreVersion.current += 1;
    reviewTrigger.current = rowTrigger(next.id);
    setSelectedId(next.id);
    setFocusRequest({ id: next.id, target: 'filename' });
    setActionMessage(`${done} Next: ${next.originalFilename}.`);
  };
  const refreshAndClear = async (run: () => Promise<void>, success: string | ((read?: QueueItem[]) => string), refused?: RefusedAction) => {
    const selectionVersion = focusRestoreVersion.current;
    if (!await runQueueAction(run, success, refused)) return;
    if (focusRestoreVersion.current !== selectionVersion) return;
    setSelectedId(undefined);
    restoreQueueFocus();
  };
  // Retry queues a failed item, a duplicate or an unverified upload to be
  // read again. A parked item's is Check again, which queues nothing: it
  // finishes the stopped rename or rolls it back, and only the queue after
  // it says which; it can also fail for the reason the item was parked.
  const retryItem = (item: QueueItem) => isParked(item)
    ? refreshAndClear(() => bridge.retry(item.id), (read) => checkedOutcome(item, read), { checkedFiles: true })
    : refreshAndClear(() => bridge.retry(item.id), 'Item queued for retry.');
  const applyAllReady = async () => {
    if (actionInFlight.current) return;
    actionInFlight.current = true;
    const selectionVersion = focusRestoreVersion.current;
    const selectedAtStart = selected;
    const batch = readyItems;
    setActionPending(true);
    setActionError('');
    // Said once, not at every step: the toast shows how far along it is.
    setActionMessage(`Applying ${batch.length} ${batch.length === 1 ? 'rename' : 'renames'}…`);
    const failed = new Map<string, unknown>();
    const applied: string[] = [];
    try {
      for (const [index, item] of batch.entries()) {
        // Forty renames take a while, and the button alone said nothing.
        showToast('progress', `Applying ${index + 1} of ${batch.length}…`);
        try { await bridge.approve(item.id, item.proposedFilename!, item.description ?? ''); applied.push(item.id); }
        catch (error) { failed.set(item.id, error); }
      }
      // Said from the batch's own read. The one on screen can be older: a
      // queue event the renames raised starts a newer read, and the read
      // made for the batch is then never shown.
      const outcome = batchOutcome(applied, await refresh() ?? await bridge.listItems());
      if (applied.length) showToast('done', outcome.text, outcome.undoable.length ? outcome.undoable : undefined);
      else setToast(undefined);
      if (failed.size) {
        const firstError = failed.values().next().value;
        setActionMessage('');
        setActionError(`${applied.length} ${applied.length === 1 ? 'rename' : 'renames'} applied. ${failed.size} could not be applied. ${describeActionError(firstError, { approve: true })}`);
      } else {
        setActionMessage(outcome.text);
      }
      if (focusRestoreVersion.current === selectionVersion && selectedAtStart?.status === 'ready' && !failed.has(selectedAtStart.id)) {
        setSelectedId(undefined);
        restoreQueueFocus();
      }
    } catch (error) {
      setToast(undefined);
      setActionMessage('');
      setActionError(`The queue could not refresh. ${describeActionError(error)}`);
    } finally {
      actionInFlight.current = false;
      setActionPending(false);
    }
  };
  // Undo from the toast puts back every rename that batch made, one at a time
  // as the backend undoes them; one it refuses is reported and the rest are
  // still undone.
  const undoRenames = async (ids: string[]) => {
    if (actionInFlight.current) return;
    actionInFlight.current = true;
    setActionPending(true);
    setActionError('');
    setActionMessage('');
    const failed: unknown[] = [];
    try {
      for (const [index, id] of ids.entries()) {
        showToast('progress', `Undoing ${index + 1} of ${ids.length}…`);
        try { await bridge.undo(id); }
        catch (error) { failed.push(error); }
      }
      // Undone or not, each command has happened; a reread that fails is
      // reported by the queue's own connection banner.
      try { await refresh(); } catch { /* Reported as a queue connection error. */ }
      const undone = ids.length - failed.length;
      const text = `Undid ${undone} ${undone === 1 ? 'rename' : 'renames'}.`;
      if (failed.length) {
        setToast(undefined);
        setActionError(`${text} ${failed.length} could not be undone. ${describeActionError(failed[0])}`);
      } else {
        showToast('done', text);
        setActionMessage(text);
      }
    } finally {
      actionInFlight.current = false;
      setActionPending(false);
    }
    // Undo went with its toast. Focus goes to the first document put back,
    // waiting in review again, rather than being left nowhere.
    if (!document.activeElement || document.activeElement === document.body) restoreQueueFocus(ids[0]);
  };
  // Opening a document changes nothing in the queue, so it takes no part in
  // the one-action-at-a-time guard; it only reports a refusal.
  const openDocument = async (id: string, reveal: boolean) => {
    setActionError('');
    try { await (reveal ? bridge.revealItem(id) : bridge.openItem(id)); }
    catch (error) { setActionError(describeActionError(error)); }
  };
  // Help leaves the app on purpose. Inside Tauri the webview has nowhere to
  // put a new tab, so the bridge hands the address to the system browser; if
  // that hand-off is refused the address itself is shown, because a person can
  // always type it.
  const openGuide = async () => {
    setActionError('');
    try { await bridge.openGuide(); }
    catch { setActionError(`The guide could not be opened. You can reach it at ${GUIDE_URL}.`); }
  };
  const saveSettings = async (next: AppSettings) => {
    if (!settingsLoaded) throw new Error('Intern could not read your settings, so saving now would replace them with defaults. Restart Intern and try again.');
    await bridge.saveSettings(next);
    setSettings(next);
  };
  // Naming the organisation renames what is still waiting. The desktop
  // backend says so with a queue event; reading the queue once more shows it
  // on a bridge that has no events, at the cost of one extra read on one that
  // does. The settings are saved either way, so a failed read is left to the
  // queue's own connection banner.
  const refreshAfterSettings = async () => {
    try { await refresh(); } catch { /* Reported as a queue connection error. */ }
  };
  // The click is not the moment Intern closes. The download comes first and
  // can take minutes, and a rename can begin its move meanwhile, which the
  // disabled button cannot see. So once every byte is in and verified, the
  // queue stops starting new work and the installer waits for any rename
  // still part-way through its move, read from the queue itself rather than
  // from the last render. A pause made here is undone if installing then
  // fails; a pause the person made is left as it was.
  // Updated in the same commit as the screen, not after paint: an Install
  // clicked the moment a pause shows must already see that pause, or it
  // pauses again and later undoes a pause the person made.
  const pausedNow = useRef(paused);
  useLayoutEffect(() => { pausedNow.current = paused; }, [paused]);
  const installUpdateBetweenRenames = async (onProgress?: UpdateProgressListener) => {
    let pausedForInstall = false;
    const settleRenames = async () => {
      if (!pausedNow.current) {
        await bridge.pauseQueue();
        pausedForInstall = true;
        setPaused(true);
      }
      while ((await bridge.listItems()).some(applyingRename)) {
        await new Promise((resolve) => window.setTimeout(resolve, RENAME_SETTLE_POLL_MS));
      }
    };
    try {
      await bridge.installUpdate(onProgress, settleRenames);
    } catch (error) {
      if (pausedForInstall) await bridge.resumeQueue().then(() => setPaused(false), () => {});
      throw error;
    }
  };
  const installUpdate = async () => {
    setUpdateInstalling(true);
    setUpdateError('');
    setUpdateProgress(undefined);
    try { await installUpdateBetweenRenames((fraction) => setUpdateProgress({ fraction })); }
    // On success this hands off to the installer and Intern is closed from
    // outside; there is nothing left to un-set `updateInstalling` for.
    catch (error) { setUpdateError(describeActionError(error)); setUpdateInstalling(false); }
  };
  const openSettings = (trigger: HTMLButtonElement) => { focusRestoreVersion.current += 1; settingsTrigger.current = trigger; setSettingsOpen(true); };
  const closeSettings = () => { setSettingsOpen(false); settingsTrigger.current?.focus(); };
  const openHistory = (trigger: HTMLButtonElement) => { focusRestoreVersion.current += 1; historyTrigger.current = trigger; setHistoryOpen(true); };
  const closeHistory = () => { setHistoryOpen(false); historyTrigger.current?.focus(); };
  // Keep picker, drop resolution, import, and refresh in the same error and
  // busy boundary. A state-only lock misses two events before React rerenders.
  const importSelection = async (choose: () => Promise<SelectionResult>) => {
    if (actionInFlight.current) {
      setActionError('Another queue action is still running. Add these files again when it finishes.');
      return;
    }
    const selectionVersion = focusRestoreVersion.current;
    let targetId: string | undefined;
    let report: AddReport | undefined;
    const imported = await runQueueAction(async () => {
      const result = await choose();
      let displayName: string;
      if (result.folder) {
        displayName = result.folder.files?.at(-1)?.displayName ?? `${result.folder.displayName}/`;
        report = await bridge.addFolder(result.folder);
      } else if (result.files?.length) {
        displayName = result.files[result.files.length - 1].displayName;
        report = await bridge.addFiles(result.files);
      } else {
        return; // Canceling a picker is not an import and needs no success notice.
      }
      setSkippedNotice('');
      const refreshed = await bridge.listItems();
      targetId = [...refreshed].reverse().find((item) => item.originalFilename === displayName)?.id;
    }, '');
    // "Added 24 documents. Skipped 1: notes.zip (not a supported format)."
    if (imported && report) {
      const message = describeAddReport(report);
      setActionMessage(message);
      if (report.skipped.length > 0) setSkippedNotice(message);
    }
    // Select only after the queue contains the imported row, and never over
    // a different document the reviewer chose while the import was running.
    if (!imported || !targetId || focusRestoreVersion.current !== selectionVersion) return;
    seededSelection.current = true;
    focusRestoreVersion.current += 1;
    reviewTrigger.current = null;
    setSelectedId(targetId);
    setSelectedByPerson(true);
  };
  // Tauri's own drag-drop is on, so on the desktop a dropped file never
  // reaches the drop zone's HTML5 handler: the paths arrive as a window event
  // instead. That path used to call the bridge straight from BrowserApp, with
  // no error to show and no busy guard, so a refused drop simply vanished.
  // It goes through the same import as the pickers now.
  const importDrop = useRef(importSelection);
  useEffect(() => { importDrop.current = importSelection; });
  useEffect(() => {
    const source = selection as (SelectionBoundary & Partial<TauriSelectionBoundary>) | undefined;
    if (!source?.subscribeDrops) return;
    let active = true;
    let stop: (() => void) | undefined;
    void source.subscribeDrops((result) => { if (active) void importDrop.current(async () => result); }).then((unsubscribe) => {
      if (active) stop = unsubscribe;
      else unsubscribe();
    }).catch(() => { /* No drop stream in this runtime; the pickers still work. */ });
    return () => { active = false; stop?.(); };
  }, [selection]);
  useEffect(() => {
    const source = selection as (SelectionBoundary & Partial<TauriSelectionBoundary>) | undefined;
    if (!source?.subscribeDragState) return;
    let active = true;
    let stop: (() => void) | undefined;
    void source.subscribeDragState((state) => { if (active) setDrag(state); }).then((unsubscribe) => {
      if (active) stop = unsubscribe;
      else unsubscribe();
    }).catch(() => { /* No drag events in this runtime; drops still land, unannounced. */ });
    return () => { active = false; stop?.(); setDrag({ dragging: false, count: 0 }); };
  }, [selection]);
  useReviewShortcuts({
    enabled: modelReady(model.setup) && !folderSetupOpen && !settingsOpen && !historyOpen,
    rows: visible,
    selected,
    inspector: inspectorHandle,
    onMove: moveSelection,
    onOpen: openItem,
    onFilter: () => {
      const box = document.querySelector<HTMLInputElement>('.queue-filter input');
      box?.focus();
      return Boolean(box);
    },
  });
  // "Send to > Intern" and a document opened with Intern add outside the
  // window, and had nowhere to say what they left out: the same note as an
  // add made here, said as soon as the window can say it.
  useEffect(() => {
    const source = bridge as DesktopBridge & Partial<LaunchReportSource>;
    if (!source.subscribeLaunchReports) return;
    return source.subscribeLaunchReports((report) => {
      const message = describeAddReport(report);
      setActionMessage(message);
      if (report.skipped.length > 0) setSkippedNotice(message);
    });
  }, [bridge]);
  // Opened from Settings. The queue stays subscribed underneath, and the
  // saved settings are read back afterwards so Settings shows the new folder.
  if (folderSetupOpen) return <FolderSetupFlow bridge={bridge} selection={selection} onCancel={() => setFolderSetupOpen(false)} onDone={async () => {
    const saved = await bridge.getSettings();
    setSettings(saved); setSettingsLoaded(true); setFolderSetupOpen(false);
  }} />;
  // A hosted model, once chosen and configured, stands in for the local one:
  // the download can be skipped entirely, or finished later from Settings.
  if (!modelReady(model.setup)) return <>
    <SetupScreen
      setup={model.setup}
      busy={model.busy}
      canChooseExisting={model.canChooseExisting}
      operationError={model.operationError}
      onStart={model.start}
      onCancel={model.cancel}
      onChooseExisting={model.chooseExisting}
      onUseHostedModel={() => setSettingsOpen(true)}
    />
    {settingsOpen && <SettingsDialog settings={settings} bridge={bridge} selection={selection} onClose={() => setSettingsOpen(false)} onSave={async (next) => { await saveSettings(next); setSettingsOpen(false); await model.refresh(); }} onCheckForUpdate={() => bridge.checkForUpdate()} onInstallUpdate={installUpdateBetweenRenames} renameApplying={renameApplying} />}
  </>;
  return <main className="app-shell" aria-label="Intern">
    <p className="sr-only" role="status" aria-label="Queue status" aria-live="polite" aria-atomic="true">{queueStatus}</p>
    <p className="sr-only" role="status" aria-label="Action status" aria-live="polite" aria-atomic="true">{actionMessage}</p>
    {drag.dragging && <div className="drop-overlay" aria-hidden="true"><p>{drag.count > 0 ? `Drop to add ${drag.count} ${drag.count === 1 ? 'file' : 'files'}` : 'Drop to add files'}</p></div>}
    {/* Settings has its own Updates section with the same information and its
       own Install button; showing both at once would be the same choice
       offered twice. */}
    {!settingsOpen && updateStatus?.state === 'available' && updateStatus.version !== updateDismissed && <div className="note note--update" role="status" aria-label="Update available">
      <p>Intern {updateStatus.version} is available. You have {updateStatus.currentVersion}.</p>
      {updateError && <p role="alert">{updateError}</p>}
      <div className="update-actions">
        <button type="button" className="primary" disabled={updateInstalling || renameApplying} onClick={() => void installUpdate()}>{updateInstalling ? installingLabel(updateProgress) : `Install ${updateStatus.version} and restart`}</button>
        <button type="button" disabled={updateInstalling} onClick={() => setUpdateDismissed(updateStatus.version)}>Not now</button>
      </div>
      {renameApplying && (!updateInstalling || updateProgress?.fraction === 1) && <p className="check-hint">Waiting for a rename to finish</p>}
    </div>}
    <AppHeader inert={drawerOpen} busy={actionPending} paused={paused} hosted={settings.modelSource === 'hosted'} onAddFiles={() => { if (selection) void importSelection(async () => ({ files: await selection.pickFiles() })); }} onAddFolder={() => { if (selection) void importSelection(async () => ({ folder: await selection.pickFolder() })); }} onTogglePause={() => void (async () => { if (await runQueueAction(() => paused ? bridge.resumeQueue() : bridge.pauseQueue(), `Queue ${paused ? 'resumed' : 'paused'}.`)) setPaused(!paused); })()} />
    <Sidebar inert={drawerOpen} active={view} items={items} onChange={(next) => { focusRestoreVersion.current += 1; reviewTrigger.current = null; setView(next); setSelectedId(undefined); }} onSettings={openSettings} onHelp={() => void openGuide()} />
    <div className="workspace"><section className="queue-panel" aria-label="Queue items" inert={drawerOpen || undefined}>
      {queueError && <div className="note note--failed" role="alert" aria-label="Queue connection error">
        <p>{queueError.kind === 'subscription' ? 'Live queue updates are unavailable.' : 'The queue could not be refreshed.'} {describeActionError(queueError.cause)} {items.length > 0 ? 'Showing the last loaded items.' : 'Queue contents may not be available yet.'}</p>
        <button type="button" disabled={actionPending} onClick={reconnect}>Retry queue connection</button>
      </div>}
      {/*
        The queue stops itself when a failure would repeat for every document -
        a model that cannot be reached, a key that was refused. Without this it
        simply went quiet, and the reason it reported was thrown away.
      */}
      {pipelineError && <div className="note note--failed" role="alert" aria-label="Queue stopped">
        <p>The queue stopped taking new work. {describeQueueStop(pipelineError)}</p>
      </div>}
      {/* A note, not a second live region: the status above already reads it out. */}
      {skippedNotice && <div className="note note--review" role="note" aria-label="Files not added">
        <p>{skippedNotice}</p>
        <button type="button" onClick={() => setSkippedNotice('')}>Dismiss</button>
      </div>}
      {/*
        An empty queue is the first thing a new user sees, and it used to be
        four column headings with nothing under them. The same drop target
        grows into the whole panel and says what to do with it.
      */}
      <DropZone variant={items.length === 0 ? 'hero' : 'bar'} onDrop={(payload) => { if (selection) void importSelection(() => selection.resolveDrop(payload)); }} />
      {/*
        The way out of a folder chosen by mistake. Pointing the queue at a large
        directory used to be unrecoverable from inside the app: pausing stops it
        taking new work but leaves the backlog, Clear history only touches
        finished items, and dropping a waiting item was one click each. Four
        hundred items meant four hundred clicks, so the count is shown here to
        make the scale of what is being dropped explicit.
      */}
      {view === 'queue' && (readyItems.length > 0 || waitingItems.length > 0) && <div className="queue-actions">
        {waitingItems.length > 0 && <button type="button" aria-label="Discard waiting items" disabled={actionPending} onClick={() => void (async () => { const dropped = waitingItems.length; await runQueueAction(() => bridge.discardWaiting(), `Discarded ${dropped} waiting ${dropped === 1 ? 'item' : 'items'}.`); })()}>Discard waiting <span>{waitingItems.length}</span></button>}
        {readyItems.length > 0 && <button type="button" className="primary" aria-label="Apply all ready" disabled={actionPending} onClick={() => void applyAllReady()}>Apply all ready <span>{readyItems.length}</span></button>}
      </div>}
      {view === 'completed' && filtered.length > 0 && <div className="queue-actions">
        <button type="button" disabled={actionPending} onClick={(event) => openHistory(event.currentTarget)}>History</button>
        <button type="button" disabled={actionPending} onClick={() => void (async () => { if (await runQueueAction(() => bridge.clearHistory(), 'History cleared.')) queueMicrotask(() => document.querySelector<HTMLButtonElement>('.sidebar button[data-view="completed"]')?.focus()); })()}>Clear history</button>
      </div>}
      {filterShown && <div className="queue-filter" role="search">
        <input type="search" aria-label="Filter queue" placeholder="Filter by filename or description" value={filter} onChange={(event) => setFilter(event.target.value)} onKeyDown={(event) => {
          // Escape clears the filter first; only an already-empty box lets the
          // key through to whatever else listens for it.
          if (event.key === 'Escape' && filter) { event.stopPropagation(); setFilter(''); }
        }} />
      </div>}
      {items.length > 0 && (visible.length > 0
        ? <QueueTable items={visible} selectedId={selectedId} onSelect={select} />
        : query
          ? <p className="queue-filter-empty" role="status">No items match “{filter.trim()}”.</p>
          : <ViewEmpty view={view} />)}
      <p className="item-count">{query ? `${visible.length} of ${filtered.length} ${filtered.length === 1 ? 'item' : 'items'}` : `${filtered.length} ${filtered.length === 1 ? 'item' : 'items'}`}</p></section>
      {/*
        At the foot of the queue, beside the panel rather than in it: the
        panel is inert under the narrow drawer, and an error in an inert
        subtree is never announced. The drawer sits above it, so it never
        covers the drawer's actions, as the centred error banner did.
      */}
      {(toast || actionError) && <div className="toasts">
        {toast && <Toast key={toast.key} tone={toast.kind === 'progress' ? 'progress' : 'success'} label={toast.kind === 'progress' ? 'Action progress' : 'Action result'}
          action={toast.undo ? { label: 'Undo', name: toast.undo.length === 1 ? 'Undo this rename' : `Undo these ${toast.undo.length} renames`, disabled: actionPending, onClick: () => void undoRenames(toast.undo!) } : undefined}
          onDismiss={toast.kind === 'progress' ? undefined : () => setToast(undefined)}>{toast.text}</Toast>}
        {actionError && <Toast tone="error" label="Action error" onDismiss={() => setActionError('')}>{actionError}</Toast>}
      </div>}
      {selected && <ReviewInspector ref={inspectorHandle} busy={actionPending} drawer={drawerOpen} item={selected}
        position={undecided.length ? { index: undecided.findIndex((item) => item.id === selected.id) + 1 || undefined, total: undecided.length } : undefined}
        onNext={nextToDecide && nextToDecide.id !== selected.id ? () => openItem(nextToDecide) : undefined}
        onClose={closeReview} onApprove={(filename, description) => void decide(selected, 'approve', () => bridge.approve(selected.id, filename, description))} onKeep={() => void decide(selected, 'keep', () => bridge.keepOriginal(selected.id))} onCancel={() => void refreshAndClear(() => bridge.cancel(selected.id), 'Processing canceled.')} onRetry={() => void retryItem(selected)} onReanalyze={() => void refreshAndClear(() => bridge.reanalyze(selected.id), 'Sent back to be analyzed again.')} onRemove={(confirmed) => void decide(selected, 'remove', () => confirmed ? bridge.remove(selected.id, { confirmed: true }) : bridge.remove(selected.id))} onUndo={() => void refreshAndClear(() => bridge.undo(selected.id), 'Operation undone.')} onOpen={() => void openDocument(selected.id, false)} onReveal={() => void openDocument(selected.id, true)} />}
    </div>
    {historyOpen && <HistoryDialog bridge={bridge} selection={selection} filedItems={new Set(items.filter((item) => item.status === 'completed').map((item) => item.id))} onClose={closeHistory} />}
    {settingsOpen && <SettingsDialog settings={settings} bridge={bridge} selection={selection} onClose={closeSettings} onChooseFolder={() => { setSettingsOpen(false); setFolderSetupOpen(true); }} onSave={async (next) => { await saveSettings(next); closeSettings(); void refreshAfterSettings(); }} onCheckForUpdate={() => bridge.checkForUpdate()} onInstallUpdate={installUpdateBetweenRenames} renameApplying={renameApplying} />}
  </main>;
}

/** Views with this many items or fewer are short enough to read; the filter box appears above longer ones. */
const FILTER_THRESHOLD = 6;

/** The demo bridge's model is ready from the first render. */
const DEMO_SETUP: SetupState = { state: 'ready', downloadedBytes: 0, totalBytes: 0 };

/** How often Intern asks GitHub for the release manifest while it keeps running, on top of the check at launch. */
export const UPDATE_POLL_INTERVAL_MS = 6 * 60 * 60 * 1000;

/** A rename in its applying stage: the file is moving, and that cannot be canceled. */
const applyingRename = (item: QueueItem) => item.status === 'processing' && item.cancelable === false;
/** How often an install that is waiting on a rename asks the queue again. A move takes moments. */
const RENAME_SETTLE_POLL_MS = 250;

function matchesQuery(item: QueueItem, query: string) {
  return [item.originalFilename, item.proposedFilename, item.description]
    .some((text) => text !== undefined && text.toLowerCase().includes(query));
}

/** "Renamed 3 documents." */
function renamedCount(count: number) {
  return `Renamed ${count} ${count === 1 ? 'document' : 'documents'}.`;
}

/**
 * What a batch of approvals the backend accepted did, read from the queue
 * after it. Accepted is not the same as renamed: while the queue is busy the
 * backend files an approved name between documents, and it sends an item back
 * to review when the file changed since it was read. Each is said as it is,
 * and only what was filed is offered to Undo.
 */
function batchOutcome(ids: string[], queue: QueueItem[]) {
  const now = (id: string) => queue.find((item) => item.id === id);
  const filed = ids.filter((id) => now(id)?.status === 'completed');
  const later = ids.filter((id) => now(id)?.status === 'ready').length;
  const back = ids.filter((id) => now(id)?.status === 'review').length;
  const text = [
    filed.length > 0 || (!later && !back) ? renamedCount(filed.length) : '',
    later ? `${later} will be renamed when the queue is free.` : '',
    back ? `${back} ${back === 1 ? 'needs' : 'need'} review again.` : '',
  ].filter(Boolean).join(' ');
  return { text, undoable: filed.filter((id) => now(id)?.undoable === true) };
}

/**
 * What Check again did, read from the queue after it: the rename had
 * finished and the document is filed, or it had not happened and the
 * document waits for a decision again.
 */
function checkedOutcome(item: QueueItem, queue?: QueueItem[]) {
  const now = queue?.find((entry) => entry.id === item.id);
  if (now?.status === 'completed') return `Checked ${item.originalFilename}: it is filed.`;
  if (now?.status === 'review') return `Checked ${item.originalFilename}: it was not renamed, and waits for your decision.`;
  return `Checked ${item.originalFilename}.`;
}

function queueStatusAnnouncement(items: QueueItem[], paused: boolean) {
  const labels: Array<[QueueItem['status'], string]> = [
    ['processing', 'processing'],
    ['ready', 'ready'],
    ['review', 'needs review'],
    ['waiting', 'waiting'],
    ['completed', 'completed'],
    ['failed', 'failed'],
  ];
  const counts = labels.flatMap(([status, label]) => {
    const count = items.filter((item) => item.status === status).length;
    return count ? [`${count} ${label}`] : [];
  });
  return `Queue ${paused ? 'paused' : 'active'}. ${counts.length ? counts.join(', ') : 'No items'}.`;
}