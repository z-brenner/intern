import { useCallback, useEffect, useRef, useState } from 'react';
import type { DesktopBridge } from '../../lib/bridge';
import { processingStage } from '../../lib/tauriBridge';
import type { QueueBridgeEvent, QueueEventSource } from '../../lib/tauriBridge';
import type { QueueItem } from '../../types';
import { createQueueReader } from './queueReader';
import type { QueueReader } from './queueReader';

interface QueueIssue { kind: 'snapshot' | 'subscription'; cause: unknown }

export function useQueue(bridge: DesktopBridge) {
  const [items, setItems] = useState<QueueItem[]>([]);
  const [paused, setPaused] = useState(false);
  // Why the queue stopped taking work, when it stopped itself. Held until the
  // queue reports that it is running again, because nothing else on screen
  // says a backlog has quietly come to a halt.
  const [pipelineError, setPipelineError] = useState<string>();
  const [readError, setReadError] = useState<QueueIssue | null>(null);
  const [subscriptionError, setSubscriptionError] = useState<QueueIssue | null>(null);
  const [connectionAttempt, setConnectionAttempt] = useState(0);
  const connection = useRef<{ bridge: DesktopBridge; reader: QueueReader<QueueItem[]> } | null>(null);
  // Resolves with the list it read. State reaches the screen a render later,
  // and the read on screen may by then be a newer one a queue event started;
  // a caller that has just acted needs the queue as it is after its action,
  // to choose what to show next and to say what the action did.
  const refresh = useCallback(async () => {
    const current = connection.current;
    return current?.bridge === bridge ? current.reader.refresh() : undefined;
  }, [bridge]);
  const reconnect = useCallback(() => setConnectionAttempt((attempt) => attempt + 1), []);

  useEffect(() => {
    let active = true;
    let stop: (() => void) | undefined;
    const reader = createQueueReader(
      () => bridge.listItems(),
      (read) => { setItems(read); setReadError(null); },
      (cause) => setReadError({ kind: 'snapshot', cause }),
    );
    connection.current = { bridge, reader };
    // Background failures are surfaced in state; command callers still receive
    // the rejection from refresh() and can report their own operation outcome.
    const refreshInBackground = () => { void reader.refresh().catch(() => {}); };
    refreshInBackground();

    const source = bridge as DesktopBridge & Partial<QueueEventSource>;
    const subscribe = source.subscribeQueue;
    const onEvent = (event: QueueBridgeEvent) => {
      if (!active) return;
      if (event.type === 'progress') {
        setItems((current) => current.map((item) => {
          if (item.id !== event.itemId || (item.status !== 'waiting' && item.status !== 'processing')) return item;
          // A figure belongs to the stage that reported it; carried into the
          // next one it would claim progress on work that has not started.
          const stage = processingStage(event.stage) ?? item.stage;
          const progress = event.progress ?? (stage === item.stage ? item.progress : undefined);
          const next: QueueItem = { ...item, status: 'processing' };
          if (stage) next.stage = stage;
          if (progress === undefined) delete next.progress;
          else next.progress = progress;
          return next;
        }));
        return;
      }
      if (event.paused !== undefined) setPaused(event.paused);
      if (event.error) setPipelineError(event.error);
      else if (event.paused === false) setPipelineError(undefined);
      refreshInBackground();
    };
    if (subscribe) {
      // Promise.resolve also captures a synchronous transport failure. A late
      // subscription is disposed immediately, including Strict Mode remounts.
      void Promise.resolve().then(() => subscribe.call(source, onEvent)).then((unsubscribe) => {
        if (!active) { unsubscribe(); return; }
        stop = unsubscribe;
        setSubscriptionError(null);
        // Close the gap between the initial read and listener registration.
        refreshInBackground();
      }).catch((cause) => {
        if (active) setSubscriptionError({ kind: 'subscription', cause });
      });
    } else {
      setSubscriptionError(null);
    }
    return () => {
      active = false;
      reader.dispose();
      if (connection.current?.reader === reader) connection.current = null;
      stop?.();
    };
  }, [bridge, connectionAttempt]);

  const execute = useCallback(async (action: () => Promise<void>) => { await action(); await refresh(); }, [refresh]);
  return { items, paused, setPaused, refresh, execute, error: readError ?? subscriptionError, pipelineError, reconnect };
}
