import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { readByLine } from '../../components/ReviewInspector';
import { ACCURATE_MODEL_BYTES, FAST_MODEL_BYTES, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem, SetupState } from '../../types';
import { openRow } from '../queue/gridTesting';
import { accuracyStatus } from './LocalModel';

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('Local processing', { selector: 'summary' }));
  return dialog;
}


/** The fast model in place and the accurate one 41% of the way down. */
function arriving(): SetupState {
  return {
    state: 'ready',
    downloadedBytes: FAST_MODEL_BYTES,
    totalBytes: FAST_MODEL_BYTES,
    tiers: [
      { tier: 'fast', displayName: 'Qwen3.5 4B', state: 'ready', downloadedBytes: FAST_MODEL_BYTES, totalBytes: FAST_MODEL_BYTES },
      { tier: 'accurate', displayName: 'Qwen3.5 9B', state: 'downloading', downloadedBytes: Math.floor(ACCURATE_MODEL_BYTES * 0.415), totalBytes: ACCURATE_MODEL_BYTES },
    ],
  };
}

describe('the local model setting', () => {
  it('offers three choices, Automatic by default, and saves the one chosen', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    const dialog = await openSettings();
    const local = within(dialog).getByRole('group', { name: 'Local model' });

    expect(within(local).getByLabelText(/Automatic \(recommended\)/)).toBeChecked();
    expect(within(local).getByText(/Qwen3\.5 9B reads every document\. Qwen3\.5 4B, if you have it, stands in only while this computer is short of memory\./)).toBeVisible();
    // Fastest says what it costs: every quick reading waits for a check.
    expect(within(local).getByText(/Qwen3\.5 4B for every document\. About half the time, but every name it gives waits for you to check it\./)).toBeVisible();
    expect(within(local).getByLabelText(/Most accurate/)).not.toBeChecked();
    expect(within(local).getByLabelText(/Fastest/)).not.toBeChecked();
    // Each model, its size, and where it is.
    expect(within(local).getByRole('listitem', { name: 'Qwen3.5 4B' })).toHaveTextContent('Qwen3.5 4B · 2.55 GiB · Ready');
    expect(within(local).getByRole('listitem', { name: 'Qwen3.5 9B' })).toHaveTextContent('Qwen3.5 9B · 5.29 GiB · Ready');

    fireEvent.click(within(local).getByLabelText(/Fastest/));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ modelSource: 'local', modelPreference: 'fast' })));
  });

  // Someone who chose only the accurate model's file from disk has just that
  // one. The quick one is offered, never fetched unasked.
  it('offers the quick model to someone who has only the accurate one', async () => {
    const bridge = createInMemoryBridge({ quick: { state: 'required' }, downloadIntervalMs: 60_000 });
    const startModelDownload = vi.fn(bridge.startModelDownload);
    render(<App bridge={{ ...bridge, startModelDownload }} />);
    const dialog = await openSettings();
    const quick = within(dialog).getByRole('listitem', { name: 'Qwen3.5 4B' });
    expect(quick).toHaveTextContent('Not downloaded');
    expect(startModelDownload).not.toHaveBeenCalled();
    fireEvent.click(within(quick).getByRole('button', { name: 'Download the quick model (2.55 GiB) for quicker readings and low-memory fallback' }));
    await waitFor(() => expect(startModelDownload).toHaveBeenCalledWith('fast'));
    expect(await within(quick).findByText(/Downloading/)).toBeVisible();
  });

  it('shows the accurate model arriving, and pauses and resumes it', async () => {
    const bridge = createInMemoryBridge({ accurate: { state: 'required', downloadedBytes: 0 }, downloadIntervalMs: 60_000 });
    const startModelDownload = vi.fn(bridge.startModelDownload);
    const setupCancel = vi.fn(bridge.setupCancel);
    render(<App bridge={{ ...bridge, startModelDownload, setupCancel }} />);
    const dialog = await openSettings();
    const accurate = within(dialog).getByRole('listitem', { name: 'Qwen3.5 9B' });
    expect(accurate).toHaveTextContent('Not downloaded');

    fireEvent.click(within(accurate).getByRole('button', { name: 'Download' }));
    await waitFor(() => expect(startModelDownload).toHaveBeenCalledWith('accurate'));
    expect(await within(accurate).findByText(/Downloading/)).toBeVisible();
    expect(within(accurate).getByRole('progressbar', { name: 'Qwen3.5 9B download' })).toBeVisible();

    fireEvent.click(within(accurate).getByRole('button', { name: 'Pause download' }));
    await waitFor(() => expect(setupCancel).toHaveBeenCalledWith('accurate'));
    expect(await within(accurate).findByRole('button', { name: /Download|Resume download/ })).toBeEnabled();
  });

  it('offers to download again a model that failed its check, and says it is not used', async () => {
    render(<App bridge={createInMemoryBridge({ accurate: { state: 'failed', error: 'MODEL_SELF_TEST_FAILED' } })} />);
    const dialog = await openSettings();
    const accurate = within(dialog).getByRole('listitem', { name: 'Qwen3.5 9B' });
    expect(accurate).toHaveTextContent('Not usable');
    expect(accurate).toHaveTextContent('did not pass its check on this computer, so Intern does not use it');
    expect(within(accurate).getByRole('button', { name: 'Download again' })).toBeEnabled();
  });
});

describe('a model whose runtime would not start', () => {
  it('says why and offers to try again rather than to download it again', async () => {
    const bridge = createInMemoryBridge({ accurate: { state: 'failed', error: 'MODEL_SERVER_START_FAILED' } });
    const startModelDownload = vi.fn(bridge.startModelDownload);
    render(<App bridge={{ ...bridge, startModelDownload }} />);
    const dialog = await openSettings();
    const accurate = within(dialog).getByRole('listitem', { name: 'Qwen3.5 9B' });
    expect(accurate).toHaveTextContent(/runtime would not start on this computer/);
    expect(within(accurate).queryByRole('button', { name: 'Download again' })).not.toBeInTheDocument();
    fireEvent.click(within(accurate).getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(startModelDownload).toHaveBeenCalledWith('accurate'));
  });
});

describe('the accurate model arriving behind the window', () => {
  it('is one quiet line on the main screen while it downloads, and nothing after', async () => {
    const setup = arriving();
    const base = createInMemoryBridge();
    let current = setup;
    render(<App bridge={{ ...base, getSetup: async () => current }} />);

    expect(await screen.findByRole('status', { name: 'Local model status' })).toHaveTextContent('Improving accuracy: downloading Qwen3.5 9B, 41%');
    current = { ...setup, tiers: setup.tiers?.map((tier) => ({ ...tier, state: 'ready', downloadedBytes: tier.totalBytes })) };
    await waitFor(() => expect(screen.queryByRole('status', { name: 'Local model status' })).not.toBeInTheDocument());
  });

  it('says nothing when there is nothing arriving', () => {
    expect(accuracyStatus(undefined)).toBeUndefined();
    expect(accuracyStatus({ state: 'ready', downloadedBytes: 1, totalBytes: 1 })).toBeUndefined();
    expect(accuracyStatus(arriving())).toBe('Improving accuracy: downloading Qwen3.5 9B, 41%');
  });

  it('follows the first model with the accurate one once Download is pressed', async () => {
    const bridge = createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: FAST_MODEL_BYTES }, downloadIntervalMs: 5 });
    render(<App bridge={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Download model' }));
    await waitFor(async () => {
      const setup = await bridge.getSetup();
      expect(setup.tiers?.map((tier) => tier.state)).toEqual(['ready', 'ready']);
    });
  });
});

describe('who read a document', () => {
  const lease = (readBy: QueueItem['readBy']) => createInMemoryBridge({
    items: [{ id: 'lease', originalFilename: 'Lease Agreement - 123 Main St.pdf', status: 'review', proposedFilename: '2023-09-15 Lease Agreement.pdf', confidence: 0.72, readBy }],
  });

  it('is named in the review panel, and a stand-in reading says it will be read again', async () => {
    render(<App bridge={lease({ tier: 'fast', modelId: 'qwen3.5-4b-instruct-q4-k-m', displayName: 'Qwen3.5 4B', provisional: true })} />);
    await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    expect(screen.getByLabelText('Read by')).toHaveTextContent('Read by Qwen3.5 4B. Provisional — will be re-read by the accurate model.');
  });

  it('says nothing for a reading no local model is named for', async () => {
    render(<App bridge={lease(undefined)} />);
    await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    expect(screen.queryByLabelText('Read by')).not.toBeInTheDocument();
  });

  it('says what happened to a reading the cascade sent on', () => {
    const accurate = { tier: 'accurate' as const, modelId: 'qwen3.5-9b-instruct-q4-k-m', displayName: 'Qwen3.5 9B' };
    expect(readByLine(accurate, true)).toBe('Read by Qwen3.5 9B');
    expect(readByLine({ ...accurate, escalated: 'review_reason' }, true)).toBe('Read by Qwen3.5 9B, after a second look.');
    expect(readByLine({ ...accurate, tier: 'fast', displayName: 'Qwen3.5 4B', escalated: 'review_reason', escalationFailed: true }, true)).toBe('Read by Qwen3.5 4B. The accurate model could not read it again.');
    // Approved: whoever acts on a reading keeps it, so it is not read again.
    expect(readByLine({ ...accurate, tier: 'fast', displayName: 'Qwen3.5 4B', provisional: true }, false)).toBe('Read by Qwen3.5 4B');
  });
});
