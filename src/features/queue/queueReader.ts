export interface QueueReader<T> {
  /**
   * Reads the queue, and publishes the read unless a newer one has started
   * since. Either way it resolves with what it read: a caller that has just
   * run a command wants the queue as it is after that command, and a queue
   * event that starts a background read a moment later must not leave it
   * holding the read from before. Undefined once disposed, or when a newer
   * read has taken over from one that failed.
   */
  refresh(): Promise<T | undefined>;
  dispose(): void;
}

/**
 * Only the most recently requested snapshot may update the queue. Desktop
 * commands and queue events can overlap, and their replies need not arrive in
 * order. A reader belongs to one subscription lifetime, never to a new bridge
 * or a remounted effect. Disposing it also absorbs obsolete rejections.
 */
export function createQueueReader<T>(
  load: () => Promise<T>,
  publish: (snapshot: T) => void,
  fail: (error: unknown) => void,
): QueueReader<T> {
  let active = true;
  let revision = 0;
  return {
    async refresh() {
      if (!active) return undefined;
      const request = ++revision;
      try {
        const snapshot = await load();
        if (active && request === revision) publish(snapshot);
        return active ? snapshot : undefined;
      } catch (error) {
        if (!active || request !== revision) return undefined;
        fail(error);
        throw error;
      }
    },
    dispose() { active = false; revision += 1; },
  };
}
