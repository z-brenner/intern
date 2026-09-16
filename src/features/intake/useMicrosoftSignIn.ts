import { useEffect, useRef, useState } from 'react';
import type { DesktopBridge } from '../../lib/bridge';
import { describeSharePointProblem } from '../sharepoint/sharePointProblems';
import type { MicrosoftDevicePrompt, MicrosoftIntakeStatus } from './microsoft';

/** A refusal whose meaning is the same wherever SharePoint setup is explained. */
const SHARED_CODES = ['MICROSOFT_MANUAL_PAIRING_DISABLED'];

export function explainMicrosoftError(error: unknown): string {
  const message = typeof error === 'string' ? error : error instanceof Error ? error.message : typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' ? error.message : '';
  const code = typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
  const shared = SHARED_CODES.find((known) => known === code || message.includes(`(${known})`));
  if (shared) return `${describeSharePointProblem(shared).action} (${shared})`;
  if (typeof error === 'string' && error.trim()) return error;
  if (error instanceof Error && error.message) return error.message;
  return 'Microsoft verification could not complete. Your files remain held.';
}

/**
 * The Microsoft device sign-in, shared by the manual intake settings and the
 * managed SharePoint connection card. Every guard lives here once: a single
 * request at a time, a generation counter so a canceled or superseded sign-in
 * can never land later, and a disconnect when the view closes mid-sign-in.
 *
 * `onConnected` runs after a sign-in completes and the status is refreshed,
 * still inside the poll's generation check.
 */
export function useMicrosoftSignIn(bridge: DesktopBridge, onConnected?: () => Promise<void>) {
  const [status, setStatus] = useState<MicrosoftIntakeStatus>();
  const [prompt, setPrompt] = useState<MicrosoftDevicePrompt>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const mounted = useRef(true);
  const inFlight = useRef(false);
  const generation = useRef(0);
  const signingIn = useRef(false);
  const connectedCallback = useRef(onConnected);
  useEffect(() => { connectedCallback.current = onConnected; }, [onConnected]);

  useEffect(() => {
    mounted.current = true;
    let active = true;
    void bridge.microsoftIntakeStatus?.().then((next) => {
      if (!active) return;
      setStatus(next);
    }).catch((cause) => { if (active) setError(explainMicrosoftError(cause)); });
    return () => {
      active = false; mounted.current = false; generation.current += 1;
      // Closing during a pending sign-in must not silently connect later.
      if (signingIn.current) { signingIn.current = false; void bridge.microsoftDisconnect?.().catch(() => {}); }
    };
  }, [bridge]);

  const run = async (action: () => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true; setBusy(true); setError('');
    try { await action(); }
    catch (cause) { if (mounted.current) setError(explainMicrosoftError(cause)); }
    finally { inFlight.current = false; if (mounted.current) setBusy(false); }
  };
  const refresh = async () => {
    const next = await bridge.microsoftIntakeStatus?.();
    if (next && mounted.current) setStatus(next);
  };
  const begin = () => void run(async () => {
    const version = ++generation.current;
    signingIn.current = true;
    try {
      const next = await bridge.microsoftSignInStart?.();
      if (mounted.current && generation.current === version) setPrompt(next);
    } catch (cause) { signingIn.current = false; throw cause; }
  });
  const disconnect = async () => {
    generation.current += 1; signingIn.current = false; setPrompt(undefined);
    await bridge.microsoftDisconnect?.(); await refresh();
  };

  useEffect(() => {
    if (!prompt) return;
    const version = generation.current;
    let active = true;
    let timer: number | undefined;
    const poll = async () => {
      if (!active || !mounted.current || generation.current !== version) return;
      if (Date.now() >= prompt.expiresAt * 1000) {
        signingIn.current = false; setPrompt(undefined); setError('Microsoft sign-in expired. Start again; your files remain held.');
        void bridge.microsoftDisconnect?.().catch(() => {}); return;
      }
      try {
        const result = await bridge.microsoftSignInPoll?.();
        if (!active || !mounted.current || generation.current !== version) return;
        if (result?.state === 'connected') {
          signingIn.current = false; setPrompt(undefined); await refresh();
          if (mounted.current && generation.current === version) await connectedCallback.current?.();
        } else {
          timer = window.setTimeout(() => { void poll(); }, Math.max(5, result?.intervalSeconds ?? prompt.intervalSeconds) * 1000);
        }
      } catch (cause) {
        if (active && mounted.current) { signingIn.current = false; setPrompt(undefined); setError(explainMicrosoftError(cause)); }
      }
    };
    timer = window.setTimeout(() => { void poll(); }, Math.max(5, prompt.intervalSeconds) * 1000);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, prompt]);

  // This reads local application state only. Microsoft's own calls happen in
  // the backend and remain subject to its polling, consent, and retry limits.
  useEffect(() => {
    if (!status?.connected) return;
    let active = true;
    let timer: number | undefined;
    const poll = async () => {
      try { const next = await bridge.microsoftIntakeStatus?.(); if (active && next) setStatus(next); }
      catch { /* Keep the last status, without claiming any new verification. */ }
      if (active) timer = window.setTimeout(() => { void poll(); }, 5000);
    };
    timer = window.setTimeout(() => { void poll(); }, 5000);
    return () => { active = false; if (timer !== undefined) window.clearTimeout(timer); };
  }, [bridge, status?.connected]);

  return { status, prompt, busy, error, run, refresh, begin, disconnect };
}
