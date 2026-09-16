import { useEffect, useState } from 'react';
import type { DesktopBridge, SelectionBoundary } from '../../lib/bridge';
import type { SetupEventSource } from '../../lib/tauriBridge';
import type { SetupState } from '../../types';

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
    void (pending ?? bridge.getSetup()).then((next) => { if (active) setSetup(next); }).catch((error) => { if (active) setSetupError(describeSetupError(error)); });
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
  useEffect(() => {
    if (setup?.state !== 'downloading') return;
    let active = true;
    let timer: number | undefined;
    const poll = async () => {
      try {
        const next = await bridge.getSetup();
        if (!active) return;
        setSetup(next);
        if (next.state === 'downloading') timer = window.setTimeout(() => { void poll(); }, 250);
      } catch (error) {
        if (active) setSetupError(describeSetupError(error));
      }
    };
    timer = window.setTimeout(() => { void poll(); }, 250);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, setup?.state]);

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
    chooseExisting: () => void runSetupAction('choose', async () => {
      const files = await selection?.pickExistingModelFiles();
      if (!files) return false;
      await bridge.setupChooseExisting(files);
    }),
    refresh: async () => setSetup(await bridge.getSetup()),
  };
}

/** A hosted model, once chosen and configured, stands in for the local one. */
export function modelReady(setup: SetupState | undefined) {
  return Boolean(setup && (setup.state === 'ready' || setup.hostedModelReady));
}

export function describeSetupError(error: unknown) {
  const code = typeof error === 'string'
    ? error
    : typeof error === 'object' && error && 'code' in error && typeof error.code === 'string'
      ? error.code
      : undefined;
  switch (code) {
    case 'SETUP_BUSY': return 'Another model setup operation is already active. Wait for it to finish or cancel it. (SETUP_BUSY)';
    // These used to name files this build does not use: "the matching Q4 or Q8
    // model and mmproj GGUF files", and an "image self-test". Intern pins one
    // model file and has no vision projector, so the advice sent people looking
    // for something that does not exist. Name the file the manifest actually
    // pins instead.
    case 'MODEL_FILE_INVALID':
    case 'MODEL_MANIFEST_INVALID': return 'The selected file did not match the model Intern pins. Choose the exact Qwen3.5-2B-Q4_K_M.gguf file, or let Intern download it. (MODEL_FILE_INVALID)';
    case 'MODEL_SELF_TEST_FAILED': return 'Intern installed the model, but its local self-test failed. Try the download again or choose a verified model file. (MODEL_SELF_TEST_FAILED)';
    case 'INSUFFICIENT_DISK': return 'There is not enough free disk space to install the local model. Free space and try again. (INSUFFICIENT_DISK)';
  }
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' && error.message.trim()) return error.message.trim();
  if (typeof error === 'string' && error.trim()) return error.trim();
  return 'The local model could not be prepared.';
}
