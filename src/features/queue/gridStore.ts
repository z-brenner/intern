import { useSyncExternalStore } from 'react';
import type { CellRef, GridSelection } from './grid';

/**
 * The queue table's cells: the active cell, where keyboard focus is and Tab
 * lands, and the anchor a selection was started from. With an anchor the
 * selection is the rectangle from it to the active cell, which is the end
 * that moves as Shift+arrow, a drag or Shift+click extends it. Without one
 * nothing is selected, and the active cell is only where focus rests.
 *
 * Kept in a small store beside the table rather than in the app's own state:
 * a drag changes the selection on every cell the pointer crosses, and as app
 * state each step re-rendered the whole window and every cell of a long
 * queue. The table and its rows subscribe to it; the app reads it only when
 * it needs to (a copy, a Delete) and subscribes to the one fact its toolbar
 * shows, whether anything is selected.
 */
export interface GridState {
  focus?: CellRef;
  anchor?: CellRef;
}

export interface GridStore {
  get(): GridState;
  set(next: GridState): void;
  subscribe(listener: () => void): () => void;
}

export function createGridStore(): GridStore {
  let state: GridState = {};
  const listeners = new Set<() => void>();
  return {
    get: () => state,
    set: (next) => {
      if (next.focus === state.focus && next.anchor === state.anchor) return;
      state = next;
      for (const listener of listeners) listener();
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => { listeners.delete(listener); };
    },
  };
}

/** The selection a state holds: its anchor to its active cell, or nothing. */
export const selectionOf = (state: GridState): GridSelection | undefined => state.anchor && state.focus ? { anchor: state.anchor, focus: state.focus } : undefined;

/** Subscribe to one value derived from the store; re-renders only when it changes (compare primitives). */
export function useGridValue<T>(store: GridStore, select: (state: GridState) => T): T {
  return useSyncExternalStore(store.subscribe, () => select(store.get()));
}
