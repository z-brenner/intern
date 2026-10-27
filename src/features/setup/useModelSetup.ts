import { useEffect, useState } from 'react';
import type { DesktopBridge, SelectionBoundary } from '../../lib/bridge';
import type { SetupEventSource } from '../../lib/tauriBridge';
import type { ModelTier, SetupState, TierSetup } from '../../types';

/**
 * The local model's setup state and actions, shared by the standalone setup
 * screen and the guided onboarding step so both keep the same resumable,
 * verified download behavior.
 *
 * `pending` is a read already started elsewhere (App starts it alongside the
 * onboarding check); without one the hook reads setup itself.
 */
export function useModelSetup(bridge: DesktopBridge, selection: SelectionBoundary | undefined, { initial, pending }: { initial?: SetupState; pending?: Promise<SetupState> } = {}) {
  const [setup, setSetup] = useState<SetupState | undefined>(initial);
  const [setupAction, setSetupAction] = useState<'start' | 'cancel' | 'choose'>();
  const [setupError, setSetupError] = useState('');

  useEffect(() => {
    let active = true;
    void Promise.resolve().then(() => pending ?? bridge.getSetup()).then((next) => { if (active) setSetup(next); }).catch((error) => { if (active) setSetupError(describeSetupError(error)); });
    return () => { active = false; };
  }, [bridge, pending]);
  useEffect(() => {
    const source = bridge as DesktopBridge & Partial<SetupEventSource>;
    if (!source.subscribeSetup) return;
    let active = true;
    let stop: (() => void) | undefined;
    void source.subscribeSetup((next) => { if (active) setSetup(next); }).then((unsubscribe) => {
      if (active) stop = unsubscribe;
      else unsubscribe();
    });
    return () => { active = false; stop?.(); };
  }, [bridge]);
  // The first model's download, or the accurate one's behind the window:
  // either is followed here, so the main window's status line and Settings
  // move with it where no events arrive.
  const moving = isDownloading(setup);
  useEffect(() => {
    if (!moving) return;
    let active = true;
    let timer: number | undefined;
    const poll = async () => {
      try {
        const next = await bridge.getSetup();
        if (!active) return;
        setSetup(next);
        if (isDownloading(next)) timer = window.setTimeout(() => { void poll(); }, 250);
      } catch (error) {
        if (active) setSetupError(describeSetupError(error));
      }
    };
    timer = window.setTimeout(() => { void poll(); }, 250);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, moving]);

  const runSetupAction = async (action: 'start' | 'cancel' | 'choose', run: () => Promise<boolean | void>) => {
    if (setupAction) return;
    setSetupAction(action);
    setSetupError('');
    try {
      if (await run() === false) return;
      setSetup(await bridge.getSetup());
    } catch (error) {
      setSetupError(describeSetupError(error));
    } finally {
      setSetupAction(undefined);
    }
  };

  return {
    setup,
    setSetup,
    busy: setupAction !== undefined,
    /** The operation's own failure, or the failure the setup state reports. */
    operationError: setupError || (setup?.state === 'failed' ? describeSetupError(setup.error) : undefined),
    canChooseExisting: Boolean(selection),
    start: () => void runSetupAction('start', () => bridge.startModelDownload()),
    cancel: () => void runSetupAction('cancel', () => bridge.setupCancel()),
    /** One model's download behind the window: start, resume, retry or download again. */
    startTier: (tier: ModelTier) => void runSetupAction('start', () => bridge.startModelDownload(tier)),
    /** Pauses one model's download behind the window. */
    pauseTier: (tier: ModelTier) => void runSetupAction('cancel', () => bridge.setupCancel(tier)),
    chooseExisting: () => void runSetupAction('choose', async () => {
      const files = await selection?.pickExistingModelFiles();
      if (!files) return false;
      await bridge.setupChooseExisting(files);
    }),
    refresh: async () => setSetup(await bridge.getSetup()),
  };
}

function isDownloading(setup: SetupState | undefined) {
  return setup?.state === 'downloading' || Boolean(setup?.tiers?.some((tier) => tier.state === 'downloading'));
}

/** The accurate model's setup, when this backend has one. */
export function accurateTier(setup: SetupState | undefined): TierSetup | undefined {
  return setup?.tiers?.find((tier) => tier.tier === 'accurate');
}

/** A hosted model, once chosen and configured, stands in for the local one. */
export function modelReady(setup: SetupState | undefined) {
  return Boolean(setup && (setup.state === 'ready' || setup.hostedModelReady));
}

/**
 * Where llama-server and the parser worker keep what they print: the app's
 * local data folder (Tauri's identifier under %LOCALAPPDATA%).
 */
const LOG_FOLDER = '%LOCALAPPDATA%\\com.intern.app\\logs';

export function describeSetupError(error: unknown) {
  const code = typeof error === 'string'
    ? error
    : typeof error === 'object' && error && 'code' in error && typeof error.code === 'string'
      ? error.code
      : undefined;
  switch (code) {
    case 'SETUP_BUSY': return 'Another model setup operation is already active. Wait for it to finish or cancel it. (SETUP_BUSY)';
    // These used to name files this build does not use: "the matching Q4 or Q8
    // model and mmproj GGUF files", and an "image self-test". Intern has no
    // vision projector, so the advice sent people looking for something that
    // does not exist. Name the files the manifest actually pins instead.
    case 'MODEL_FILE_INVALID':
    case 'MODEL_MANIFEST_INVALID': return 'The selected file did not match a model Intern pins. Choose the exact Ternary-Bonsai-2-2B-PQ2_0.gguf or Ternary-Bonsai-2-4B-PQ2_0.gguf file, or let Intern download them. (MODEL_FILE_INVALID)';
    // The runtime failures below happen after the model file has passed its
    // checksum, so a fresh download re-hashes the same file and fails the
    // same way. They point at the runtime's own log instead.
    case 'MODEL_SELF_TEST_FAILED': return `Intern installed the model, but its local self-test failed. The model file itself checked out, so downloading it again will not help; what the model runtime reported is in llama-server.log in ${LOG_FOLDER}. (MODEL_SELF_TEST_FAILED)`;
    case 'MODEL_SERVER_START_FAILED': return `The local model is installed, but its runtime would not start on this computer. Security software blocking one of its files is a common cause; the reason is in llama-server.log in ${LOG_FOLDER}. Downloading the model again will not fix this. (MODEL_SERVER_START_FAILED)`;
    case 'MODEL_SERVER_UNHEALTHY': return `The local model's runtime started but did not become ready in time. Close other heavy programs and try again; if it keeps happening, llama-server.log in ${LOG_FOLDER} says why. (MODEL_SERVER_UNHEALTHY)`;
    case 'INSUFFICIENT_DISK': return 'There is not enough free disk space to install the local model. Free space and try again. (INSUFFICIENT_DISK)';
  }
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  if (typeof error === 'string' && error.trim()) return error.trim();
  return 'The local model could not be prepared.';
}
