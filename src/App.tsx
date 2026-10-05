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
import { ViewEmpty } from './components/ViewEmpty';
import { GUIDE_URL } from './lib/bridge';
import { installingLabel } from './lib/format';
import { humanizeReason } from './lib/reasons';
import type { DesktopBridge, SelectionBoundary, SelectionResult, UpdateProgressListener, UpdateStatus } from './lib/bridge';
import { createInMemoryBridge } from './lib/inMemoryBridge';
import type { TauriSelectionBoundary } from './lib/tauriBridge';
import { useMediaQuery } from './lib/useMediaQuery';
import { describeSharePointProblem } from './features/sharepoint/sharePointProblems';
import type { SharePointProblem } from './features/sharepoint/sharePointProblems';
import { useQueue } from './features/queue/useQueue';
import { modelReady, useModelSetup } from './features/setup/useModelSetup';
import type { AppSettings, QueueItem, QueueView, SetupState } from './types';

type Gate =
  | { kind: 'loading' }
  | { kind: 'failed'; problem: SharePointProblem }
  | { kind: 'onboarding' | 'folder-setup' | 'app'; pendingSettings?: Promise<AppSettings>; pendingSetup?: Promise<SetupState>; initialSetup?: SetupState };

export function App({ bridge: suppliedBridge, selection }: { bridge?: DesktopBridge; selection?: SelectionBoundary }) {
  const bridgeRef = useRef<DesktopBridge>(suppliedBridge ?? createInMemoryBridge());
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
  const [actionError, setActionError] = useState('');
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>();
  // The version dismissed from the banner, so "Not now" does not reappear on
  // every later poll - but a newer release than the one dismissed still does.
  const [updateDismissed, setUpdateDismissed] = useState<string>();
  const [updateInstalling, setUpdateInstalling] = useState(false);
  const [updateProgress, setUpdateProgress] = useState<{ fraction: number | undefined }>();
  const [updateError, setUpdateError] = useState('');
  const narrowInspector = useMediaQuery('(max-width: 1100px)');

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
  const filtered = items.filter((item) => view === 'queue' ? item.status !== 'completed' : view === 'review' ? item.status === 'review' : item.status === 'completed');
  // A folder of four hundred documents is a wall of rows. The filter narrows
  // the current view by anything a person is likely to remember: the name the
  // file arrived with, the name it was given, or a word from its description.
  const query = filter.trim().toLowerCase();
  const visible = query ? filtered.filter((item) => matchesQuery(item, query)) : filtered;
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
  const select = (item: QueueItem, trigger: HTMLButtonElement) => { seededSelection.current = true; focusRestoreVersion.current += 1; reviewTrigger.current = { element: trigger, itemId: item.id }; setSelectedId(item.id); setSelectedByPerson(true); };
  const restoreQueueFocus = () => {
    const invocation = reviewTrigger.current;
    const version = ++focusRestoreVersion.current;
    reviewTrigger.current = null;
    queueMicrotask(() => {
      if (focusRestoreVersion.current !== version) return;
      const refreshedTrigger = [...document.querySelectorAll<HTMLButtonElement>('.row-select')]
        .find((button) => button.dataset.itemId === invocation?.itemId);
      // Prefer the primary action by name rather than "the first button in the
      // panel". That positional fallback silently moved the moment the toolbar
      // gained a second control, sending focus to a destructive Discard button
      // instead of Apply all ready.
      const target = invocation?.element.isConnected
        ? invocation.element
        : refreshedTrigger
          ?? document.querySelector<HTMLButtonElement>('.queue-panel .queue-actions button.primary')
          ?? document.querySelector<HTMLButtonElement>('.queue-panel button');
      target?.focus();
    });
  };
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
  // resolves with a count - and the result is not needed here.
  const runQueueAction = async (run: () => Promise<unknown>, success: string) => {
    if (actionInFlight.current) return false;
    actionInFlight.current = true;
    setActionPending(true);
    setActionError('');
    setActionMessage('');
    try {
      try { await run(); }
      catch (error) {
        try { await refresh(); } catch { /* Preserve the original command error. */ }
        setActionError(describeActionError(error));
        return false;
      }
      // The command has already happened. A reread that fails afterwards is
      // reported by the queue's own connection banner, and calling the command
      // failed would send someone looking for a file under its old name.
      try { await refresh(); } catch { /* Reported as a queue connection error. */ }
      setActionMessage(success);
      return true;
    } finally {
      actionInFlight.current = false;
      setActionPending(false);
    }
  };
  const refreshAndClear = async (run: () => Promise<void>, success: string) => {
    const selectionVersion = focusRestoreVersion.current;
    if (!await runQueueAction(run, success)) return;
    if (focusRestoreVersion.current !== selectionVersion) return;
    setSelectedId(undefined);
    restoreQueueFocus();
  };
  const applyAllReady = async () => {
    if (actionInFlight.current) return;
    actionInFlight.current = true;
    const selectionVersion = focusRestoreVersion.current;
    const selectedAtStart = selected;
    setActionPending(true);
    setActionError('');
    setActionMessage('');
    const failed = new Map<string, unknown>();
    let applied = 0;
    try {
      for (const item of readyItems) {
        try { await bridge.approve(item.id, item.proposedFilename!, item.description ?? ''); applied += 1; }
        catch (error) { failed.set(item.id, error); }
      }
      await refresh();
      if (failed.size) {
        const firstError = failed.values().next().value;
        setActionError(`${applied} ${applied === 1 ? 'rename' : 'renames'} applied. ${failed.size} could not be applied. ${describeActionError(firstError)}`);
      } else {
        setActionMessage(`${applied} ${applied === 1 ? 'rename' : 'renames'} applied.`);
      }
      if (focusRestoreVersion.current === selectionVersion && selectedAtStart?.status === 'ready' && !failed.has(selectedAtStart.id)) {
        setSelectedId(undefined);
        restoreQueueFocus();
      }
    } catch (error) {
      setActionError(`The queue could not refresh. ${describeActionError(error)}`);
    } finally {
      actionInFlight.current = false;
      setActionPending(false);
    }
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
    const imported = await runQueueAction(async () => {
      const result = await choose();
      let displayName: string;
      if (result.folder) {
        displayName = result.folder.files?.at(-1)?.displayName ?? `${result.folder.displayName}/`;
        await bridge.addFolder(result.folder);
      } else if (result.files?.length) {
        displayName = result.files[result.files.length - 1].displayName;
        await bridge.addFiles(result.files);
      } else {
        return; // Canceling a picker is not an import and needs no success notice.
      }
      const refreshed = await bridge.listItems();
      targetId = [...refreshed].reverse().find((item) => item.originalFilename === displayName)?.id;
    }, '');
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
    {/*
      An alert, not a polite status: this paragraph is created with its
      sentence already in it, and a live region that arrives complete is not
      reliably spoken. Every other error banner in the app is an alert too.
    */}
    {actionError && <p className="operation-feedback" role="alert" aria-label="Action error">{actionError}</p>}
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
        <p>The queue stopped taking new work. {humanizeReason(pipelineError)}</p>
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
        <button type="button" disabled={actionPending} onClick={() => void (async () => { if (await runQueueAction(() => bridge.clearHistory(), 'History cleared.')) queueMicrotask(() => document.querySelector<HTMLButtonElement>('.sidebar button[aria-label="Completed"]')?.focus()); })()}>Clear history</button>
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
      {selected && <ReviewInspector busy={actionPending} drawer={drawerOpen} item={selected} onClose={closeReview} onApprove={(filename, description) => void refreshAndClear(() => bridge.approve(selected.id, filename, description), 'Rename applied.')} onKeep={() => void refreshAndClear(() => bridge.keepOriginal(selected.id), 'Original filename kept.')} onCancel={() => void refreshAndClear(() => bridge.cancel(selected.id), 'Processing canceled.')} onRetry={() => void refreshAndClear(() => bridge.retry(selected.id), 'Item queued for retry.')} onRemove={() => void refreshAndClear(() => bridge.remove(selected.id), 'Item removed.')} onUndo={() => void refreshAndClear(() => bridge.undo(selected.id), 'Operation undone.')} />}
    </div>
    {historyOpen && <HistoryDialog bridge={bridge} selection={selection} onClose={closeHistory} />}
    {settingsOpen && <SettingsDialog settings={settings} bridge={bridge} selection={selection} onClose={closeSettings} onChooseFolder={() => { setSettingsOpen(false); setFolderSetupOpen(true); }} onSave={async (next) => { await saveSettings(next); closeSettings(); }} onCheckForUpdate={() => bridge.checkForUpdate()} onInstallUpdate={installUpdateBetweenRenames} renameApplying={renameApplying} />}
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

function describeActionError(error: unknown) {
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  return 'The operation could not be completed.';
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