import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import type { SelectionBoundary } from '../../lib/bridge';
import { PINNED_MODEL_BYTES, PINNED_SMALL_MODEL_BYTES, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from '../../lib/inMemoryBridge';

/** The previous release's model file, exactly as the backend reports it. */
const legacyModel = { file: 'Qwen3.5-2B-Q4_K_M.gguf', bytes: 1_280_835_840 };

const selection: SelectionBoundary = {
  pickFiles: async () => [],
  pickFolder: async () => undefined,
  pickExistingModelFiles: async () => undefined,
  resolveDrop: async () => ({}),
};

function requiredBridge(options: InMemoryBridgeOptions = {}) {
  return createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES }, ...options });
}

/** What `navigator.deviceMemory` reports, or nothing, as in a browser that does not say. */
function reportMemory(gigabytes: number | undefined) {
  if (gigabytes === undefined) Reflect.deleteProperty(navigator, 'deviceMemory');
  else Object.defineProperty(navigator, 'deviceMemory', { value: gigabytes, configurable: true });
}

afterEach(() => reportMemory(undefined));

describe('choosing the local model during setup', () => {
  it('offers the smaller model with what it trades, and downloads it only when chosen', async () => {
    const bridge = requiredBridge({ downloadIntervalMs: 60_000 });
    const setLocalModel = vi.spyOn(bridge, 'setLocalModel');
    const startModelDownload = vi.spyOn(bridge, 'startModelDownload');
    render(<App bridge={bridge} selection={selection} />);

    expect(await screen.findByText(/Download 1\.07 GiB of model files/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Download model' })).toBeEnabled();
    const smaller = await screen.findByRole('button', { name: 'Use the smaller model (493 MiB) instead' });
    expect(screen.getByText(/names fewer documents correctly: 8 of 12 in Intern's evaluation, against 11 of 12/)).toBeVisible();
    expect(setLocalModel).not.toHaveBeenCalled();
    expect(startModelDownload).not.toHaveBeenCalled();

    fireEvent.click(smaller);

    await waitFor(() => expect(setLocalModel).toHaveBeenCalledWith('bonsai-2b'));
    await waitFor(() => expect(startModelDownload).toHaveBeenCalledOnce());
    expect(await screen.findByText('0 bytes of 493 MiB · 0%')).toBeVisible();
    expect((await bridge.getSetup()).totalBytes).toBe(PINNED_SMALL_MODEL_BYTES);
    // The way back, held while the download runs.
    expect(await screen.findByRole('button', { name: 'Use the recommended model (1.07 GiB) instead' })).toBeDisabled();
    expect(screen.getByText(/names more documents correctly \(11 of 12 in Intern's evaluation, against 8 of 12\)/)).toBeVisible();
  });

  it('switches back to the recommended model the same way', async () => {
    const bridge = requiredBridge({ settings: { localModel: 'bonsai-2b' }, setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_SMALL_MODEL_BYTES }, downloadIntervalMs: 60_000 });
    const setLocalModel = vi.spyOn(bridge, 'setLocalModel');
    render(<App bridge={bridge} selection={selection} />);

    expect(await screen.findByText(/Download 493 MiB of model files/)).toBeVisible();
    expect(screen.queryByRole('button', { name: /Use the smaller model/ })).not.toBeInTheDocument();
    fireEvent.click(await screen.findByRole('button', { name: 'Use the recommended model (1.07 GiB) instead' }));

    await waitFor(() => expect(setLocalModel).toHaveBeenCalledWith('bonsai-4b'));
    expect(await screen.findByText('0 bytes of 1.07 GiB · 0%')).toBeVisible();
  });

  it('starts an installed model without downloading it again', async () => {
    const bridge = requiredBridge({ installedModels: ['bonsai-2b'] });
    const startModelDownload = vi.spyOn(bridge, 'startModelDownload');
    render(<App bridge={bridge} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Use the smaller model (493 MiB) instead' }));

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeVisible();
    expect(startModelDownload).not.toHaveBeenCalled();
  });

  it('says why a switch was refused while a download runs', async () => {
    const bridge = requiredBridge();
    render(<App bridge={{ ...bridge, setLocalModel: async () => { throw { code: 'SETUP_BUSY', message: 'a model setup operation is already active' }; } }} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Use the smaller model (493 MiB) instead' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('(SETUP_BUSY)');
  });

  it('names both accepted files, with their sizes, when a chosen file does not match', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'failed', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES, error: 'MODEL_FILE_INVALID' } })} selection={selection} />);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('The selected file did not match either model Intern pins. Choose Ternary-Bonsai-2-4B-PQ2_0.gguf (1.07 GiB) or Ternary-Bonsai-2-2B-PQ2_0.gguf (493 MiB), or let Intern download one. (MODEL_FILE_INVALID)');
    expect(alert).not.toHaveTextContent(/Qwen/);
  });

  it('names both files from the pinned facts when the backend cannot list them', async () => {
    const bridge = createInMemoryBridge({ setup: { state: 'failed', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES, error: 'MODEL_MANIFEST_INVALID' } });
    render(<App bridge={{ ...bridge, getLocalModels: async () => { throw new Error('unknown command'); } }} selection={selection} />);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Ternary-Bonsai-2-4B-PQ2_0.gguf (1.07 GiB) or Ternary-Bonsai-2-2B-PQ2_0.gguf (493 MiB)');
  });

  it('installs a chosen copy of the other model as the chosen one', async () => {
    const bridge = requiredBridge();
    const picking: SelectionBoundary = { ...selection, pickExistingModelFiles: async () => ({ modelPath: 'D:\\Models\\Ternary-Bonsai-2-2B-PQ2_0.gguf' }) };
    render(<App bridge={bridge} selection={picking} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose existing model files' }));

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeVisible();
    const models = await bridge.getLocalModels();
    expect(models.find((model) => model.selected)).toMatchObject({ id: 'bonsai-2b', installed: true });
    expect((await bridge.getSettings()).localModel).toBe('bonsai-2b');
  });
});

describe('the memory hint', () => {
  it('suggests the smaller model on a computer with less than 8 GB, and changes nothing', async () => {
    reportMemory(4);
    const bridge = requiredBridge();
    const setLocalModel = vi.spyOn(bridge, 'setLocalModel');
    render(<App bridge={bridge} selection={selection} />);

    expect(await screen.findByRole('note')).toHaveTextContent('This computer has less than 8 GB of memory. The smaller model offered below needs about half as much, and may suit it better.');
    expect(screen.getByRole('button', { name: 'Download model' })).toBeEnabled();
    expect(setLocalModel).not.toHaveBeenCalled();
    expect((await bridge.getSetup()).totalBytes).toBe(PINNED_MODEL_BYTES);
  });

  it.each([[8], [undefined]])('says nothing when the computer reports %s GB', async (gigabytes) => {
    reportMemory(gigabytes);
    render(<App bridge={requiredBridge()} selection={selection} />);

    await screen.findByRole('button', { name: 'Use the smaller model (493 MiB) instead' });
    expect(screen.queryByRole('note')).not.toBeInTheDocument();
  });

  it('says nothing once the smaller model is chosen', async () => {
    reportMemory(2);
    render(<App bridge={requiredBridge({ settings: { localModel: 'bonsai-2b' }, setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_SMALL_MODEL_BYTES } })} selection={selection} />);

    await screen.findByRole('button', { name: 'Use the recommended model (1.07 GiB) instead' });
    expect(screen.queryByRole('note')).not.toBeInTheDocument();
  });
});

describe('the previous model', () => {
  it('is removed only when a person clicks, and then no longer offered', async () => {
    const bridge = requiredBridge({ legacyModel });
    const removeLegacyModel = vi.spyOn(bridge, 'removeLegacyModel');
    render(<App bridge={bridge} selection={selection} />);

    const group = await screen.findByRole('group', { name: 'Previous model' });
    expect(group).toHaveTextContent('An earlier version of Intern downloaded Qwen3.5-2B-Q4_K_M.gguf. This version no longer uses it.');
    expect(removeLegacyModel).not.toHaveBeenCalled();

    fireEvent.click(within(group).getByRole('button', { name: 'Remove the previous model (1.19 GiB)' }));

    await waitFor(() => expect(removeLegacyModel).toHaveBeenCalledOnce());
    expect(await within(group).findByRole('status')).toHaveTextContent('Removed the previous model. 1.19 GiB freed.');
    expect(screen.queryByRole('button', { name: /Remove the previous model/ })).not.toBeInTheDocument();
    expect(await bridge.getLegacyModel()).toBeNull();
  });

  it('is not offered when there is none', async () => {
    render(<App bridge={requiredBridge()} selection={selection} />);

    await screen.findByRole('button', { name: 'Download model' });
    expect(screen.queryByRole('group', { name: 'Previous model' })).not.toBeInTheDocument();
  });

  it('says why it was left alone when the file is not exactly the old model', async () => {
    const bridge = requiredBridge({ legacyModel });
    render(<App bridge={{ ...bridge, removeLegacyModel: async () => { throw { code: 'LEGACY_MODEL_MISMATCH', message: 'the file is not the previous model' }; } }} selection={selection} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Remove the previous model (1.19 GiB)' }));

    expect(await screen.findByRole('alert')).toHaveTextContent('not exactly the previous model, so Intern left it alone. (LEGACY_MODEL_MISMATCH)');
    expect(screen.getByRole('button', { name: 'Remove the previous model (1.19 GiB)' })).toBeEnabled();
  });

  it('is offered in Settings too', async () => {
    const bridge = createInMemoryBridge({ legacyModel });
    const removeLegacyModel = vi.spyOn(bridge, 'removeLegacyModel');
    render(<App bridge={bridge} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });

    const remove = await within(dialog).findByRole('button', { name: 'Remove the previous model (1.19 GiB)' });
    expect(removeLegacyModel).not.toHaveBeenCalled();
    fireEvent.click(remove);

    await waitFor(() => expect(removeLegacyModel).toHaveBeenCalledOnce());
    await waitFor(() => expect(within(dialog).queryByRole('button', { name: /Remove the previous model/ })).not.toBeInTheDocument());
  });
});

describe('choosing the local model in Settings', () => {
  const openSettings = async () => {
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    return screen.findByRole('dialog', { name: 'Settings' });
  };

  it('offers both models with their sizes and saves the choice, which then waits for a download', async () => {
    const bridge = createInMemoryBridge();
    const saveSettings = vi.spyOn(bridge, 'saveSettings');
    const startModelDownload = vi.spyOn(bridge, 'startModelDownload');
    render(<App bridge={bridge} />);
    const dialog = await openSettings();

    const group = within(dialog).getByRole('radiogroup', { name: 'Local model' });
    const recommended = within(group).getByRole('radio', { name: 'Bonsai 4B, 1.07 GiB (recommended)' });
    expect(recommended).toBeChecked();
    const smaller = await within(group).findByRole('radio', { name: 'Bonsai 2B, 493 MiB (for less memory) · not downloaded' });
    expect(group).toHaveTextContent('named only 8 of 12 correctly');
    expect(group).toHaveTextContent('Switching unloads the model in use before loading the other.');

    fireEvent.click(smaller);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ localModel: 'bonsai-2b', modelSource: 'local' })));
    // Not downloaded yet: setup waits for the person to start it.
    expect(await screen.findByText(/Download 493 MiB of model files/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Download model' })).toBeEnabled();
    expect(startModelDownload).not.toHaveBeenCalled();
  });

  it('switches to an installed model without leaving the queue', async () => {
    const bridge = createInMemoryBridge({ installedModels: ['bonsai-4b', 'bonsai-2b'] });
    render(<App bridge={bridge} />);
    const dialog = await openSettings();

    fireEvent.click(await within(dialog).findByRole('radio', { name: 'Bonsai 2B, 493 MiB (for less memory)' }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect(screen.getByRole('main', { name: 'Intern' })).toBeVisible();
    expect(await bridge.getSetup()).toMatchObject({ state: 'ready', localModel: 'bonsai-2b', totalBytes: PINNED_SMALL_MODEL_BYTES });
  });

  it('hides the local choice while a hosted model is chosen', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();

    fireEvent.click(within(dialog).getByLabelText('Hosted model with my API key'));
    expect(within(dialog).queryByRole('radiogroup', { name: 'Local model' })).not.toBeInTheDocument();
  });

  it('explains a switch refused while the download runs', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={{ ...bridge, saveSettings: async () => { throw { code: 'SETUP_BUSY', message: 'busy' }; } }} />);
    const dialog = await openSettings();

    fireEvent.click(await within(dialog).findByRole('radio', { name: /Bonsai 2B/ }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    expect(await within(dialog).findByRole('alert')).toHaveTextContent('could not be switched while its download or check is running');
  });
});
