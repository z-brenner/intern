import { useMemo, useRef } from 'react';
import { App } from './App';
import { createBrowserSelectionBoundary, createFixtureBatchBridge, createInMemoryBridge } from './lib/inMemoryBridge';
import { TauriBridge, createTauriSelectionBoundary, isTauriRuntime } from './lib/tauriBridge';

/**
 * `?sharePoint=fake` swaps in the in-memory bridge's simulated SharePoint
 * deployment so browser development and the Playwright journey can walk guided
 * onboarding. It is a query parameter rather than a default so that `/` - the
 * demo app, the exploratory run, and the reviewed QA capture - stays exactly as
 * it was. It is read only in a Vite dev server (`import.meta.env.DEV`, which a
 * production build compiles to false and drops), and never in the Tauri
 * runtime, which always talks to the real backend below.
 */
function browserBridge() {
  const params = new URLSearchParams(window.location.search);
  if (params.get('fixtureBatch') === '1') return createFixtureBatchBridge();
  if (import.meta.env.DEV && params.get('sharePoint') === 'fake') return createInMemoryBridge({ sharePoint: 'fake' });
  // A first run without the deployment, for walking folder setup.
  if (import.meta.env.DEV && params.get('folderSetup') === 'fake') return createInMemoryBridge({ completedOnboardingVersion: 0 });
  // An update on offer, for checking where its banner sits in the window.
  if (import.meta.env.DEV && params.get('update') === 'available') return createInMemoryBridge({ update: { state: 'available', currentVersion: '0.1.0', version: '0.2.0' } });
  return undefined;
}

export function BrowserApp() {
  const bridge = useRef(isTauriRuntime() ? undefined : browserBridge()).current;
  if (isTauriRuntime()) {
    return <TauriApp />;
  }
  return <App bridge={bridge} selection={createBrowserSelectionBoundary()} />;
}

function TauriApp() {
  const bridge = useMemo(() => new TauriBridge(), []);
  // The window's drag-drop events are subscribed to by App, which routes them
  // through the same import path as the file pickers.
  const selection = useMemo(() => createTauriSelectionBoundary(), []);
  return <App bridge={bridge} selection={selection} />;
}
