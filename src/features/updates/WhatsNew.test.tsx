import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { WhatsNew, LAST_SEEN_VERSION_KEY } from './WhatsNew';
import { changesSince, releaseNotes } from './releaseNotes';
import packageInfo from '../../../package.json';
import { App } from '../../App';
import { createReviewBatchBridge } from '../../lib/inMemoryBridge';
import { openDetails } from '../queue/gridTesting';

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function () { this.removeAttribute('open'); };
});

describe('update summary', () => {
  it('ships plain-language notes for the installed release version', () => {
    expect(releaseNotes.find((release) => release.version === packageInfo.version)?.changes.length).toBeGreaterThan(0);
  });

  it('does not let review shortcuts act on documents behind the summary', async () => {
    const base = createReviewBatchBridge();
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove) };
    render(<App bridge={bridge} />);
    await openDetails();
    render(<WhatsNew getVersion={async () => '0.1.0-alpha.16'} />);
    const proceed = await screen.findByRole('button', { name: 'Continue' });
    for (const key of ['j', 'k', 'F2', 'Delete', '/']) {
      fireEvent.keyDown(proceed, { key });
      expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Scan 0101.pdf');
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    }
    fireEvent.keyDown(proceed, { key: 'Enter', ctrlKey: true });
    fireEvent.keyDown(proceed, { key: 'k', code: 'KeyK', altKey: true });
    fireEvent.keyDown(proceed, { key: 'n', code: 'KeyN', altKey: true });
    expect(bridge.approve).not.toHaveBeenCalled();
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(bridge.remove).not.toHaveBeenCalled();
    expect(screen.getByRole('complementary', { name: 'Review item' })).toHaveTextContent('Scan 0101.pdf');
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('shows the new changes after upgrading and remembers dismissal across launches', async () => {
    localStorage.setItem(LAST_SEEN_VERSION_KEY, '0.1.0-alpha.15');
    const getVersion = async () => '0.1.0-alpha.16';
    const view = render(<WhatsNew getVersion={getVersion} />);
    expect(await screen.findByRole('dialog', { name: 'What’s new in Intern' })).toBeVisible();
    expect(screen.getByText(/Status now comes first/)).toBeVisible();
    expect(localStorage.getItem(LAST_SEEN_VERSION_KEY)).toBe('0.1.0-alpha.15');
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    view.unmount();
    render(<WhatsNew getVersion={getVersion} />);
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_VERSION_KEY)).toBe('0.1.0-alpha.16'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('introduces the current changes when an older installation has no saved version', async () => {
    render(<WhatsNew getVersion={async () => '0.1.0-alpha.16'} />);
    const dialog = await screen.findByRole('dialog');
    fireEvent(dialog, new Event('cancel', { bubbles: true, cancelable: true }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(localStorage.getItem(LAST_SEEN_VERSION_KEY)).toBe('0.1.0-alpha.16');
  });

  it('does not announce a downgrade or overwrite the last acknowledged release', async () => {
    localStorage.setItem(LAST_SEEN_VERSION_KEY, '0.1.0-alpha.17');
    const getVersion = vi.fn(async () => '0.1.0-alpha.16');
    render(<WhatsNew getVersion={getVersion} />);
    await waitFor(() => expect(getVersion).toHaveBeenCalledOnce());
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(localStorage.getItem(LAST_SEEN_VERSION_KEY)).toBe('0.1.0-alpha.17');
  });

  it('leaves the version unacknowledged if loading the installed version fails', async () => {
    render(<WhatsNew getVersion={async () => { throw new Error('unavailable'); }} />);
    await Promise.resolve();
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(localStorage.getItem(LAST_SEEN_VERSION_KEY)).toBeNull();
  });

  it('can still be dismissed if persistence is unavailable', async () => {
    const persist = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('full'); });
    try {
      render(<WhatsNew getVersion={async () => '0.1.0-alpha.16'} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Continue' }));
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    } finally {
      persist.mockRestore();
    }
  });

  it('includes skipped releases and compares alpha numbers numerically', () => {
    const notes = [
      { version: '0.1.0-alpha.9', changes: ['Old'] },
      { version: '0.1.0-alpha.10', changes: ['First improvement'] },
      { version: '0.1.0-alpha.11', changes: ['Second improvement'] },
    ];
    expect(changesSince('0.1.0-alpha.9', '0.1.0-alpha.11', notes)).toEqual(['First improvement', 'Second improvement']);
    expect(changesSince(undefined, '0.1.0-alpha.11', notes)).toEqual(['Second improvement']);
    expect(changesSince('0.1.0-alpha.11', '0.1.0', [...notes, { version: '0.1.0', changes: ['Stable'] }])).toEqual(['Stable']);
  });
});
