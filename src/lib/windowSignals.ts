import { isTauriRuntime } from './tauriBridge';

/**
 * The window's own signals to Windows: the taskbar button's progress, and a
 * flash when a batch finishes while Intern is in the background. Both no-op
 * outside the desktop runtime, and a refusal is swallowed: they are extra
 * channels for what the window already says, never the only one.
 */

export type TaskbarState = { status: 'none' } | { status: 'normal' | 'paused' | 'error'; progress: number };

let last = '';

export async function setTaskbarProgress(state: TaskbarState): Promise<void> {
  if (!isTauriRuntime()) return;
  const key = JSON.stringify(state);
  if (key === last) return;
  last = key;
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    type Status = Parameters<ReturnType<typeof getCurrentWindow>['setProgressBar']>[0]['status'];
    await getCurrentWindow().setProgressBar({
      status: state.status as Status,
      ...(state.status === 'none' ? {} : { progress: Math.max(0, Math.min(100, Math.round(state.progress))) }),
    });
  } catch {
    last = '';
  }
}

/** Flash the taskbar button once a batch is done, if nobody is looking at the window. */
export async function requestAttentionIfUnfocused(): Promise<void> {
  if (!isTauriRuntime()) return;
  try {
    const { getCurrentWindow, UserAttentionType } = await import('@tauri-apps/api/window');
    const current = getCurrentWindow();
    if (await current.isFocused()) return;
    await current.requestUserAttention(UserAttentionType.Informational);
  } catch {
    // The taskbar is an extra; the queue already says the batch is done.
  }
}
