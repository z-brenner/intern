import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { TOAST_TIMEOUT_MS, Toast } from './Toast';

describe('Toast', () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });
  const wait = (ms: number) => act(() => { vi.advanceTimersByTime(ms); });

  it('lets a success go after ten seconds, but not while it is pointed at or in focus', () => {
    const onDismiss = vi.fn();
    render(<Toast tone="success" label="Action result" action={{ label: 'Undo', onClick: vi.fn() }} onDismiss={onDismiss}>Renamed 3 documents.</Toast>);
    const toast = screen.getByRole('group', { name: 'Action result' });

    fireEvent.pointerEnter(toast);
    wait(TOAST_TIMEOUT_MS * 3);
    expect(onDismiss).not.toHaveBeenCalled();
    fireEvent.pointerLeave(toast);
    act(() => screen.getByRole('button', { name: 'Undo' }).focus());
    wait(TOAST_TIMEOUT_MS * 3);
    expect(onDismiss).not.toHaveBeenCalled();

    // Left alone, the whole ten seconds start again.
    act(() => screen.getByRole('button', { name: 'Undo' }).blur());
    wait(TOAST_TIMEOUT_MS - 1);
    expect(onDismiss).not.toHaveBeenCalled();
    wait(1);
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('keeps a failure until it is dismissed', () => {
    const onDismiss = vi.fn();
    render(<Toast tone="error" label="Action error" onDismiss={onDismiss}>The destination is locked.</Toast>);

    wait(TOAST_TIMEOUT_MS * 6);

    expect(onDismiss).not.toHaveBeenCalled();
    expect(screen.getByRole('alert', { name: 'Action error' })).toHaveTextContent('The destination is locked.');
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('offers no way to wave away work still running', () => {
    render(<Toast tone="progress" label="Action progress">Applying 7 of 40…</Toast>);

    expect(screen.getByRole('group', { name: 'Action progress' })).toHaveTextContent('Applying 7 of 40…');
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
    // Not an alert: progress is not an error, and it is not read at every step.
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });
});
