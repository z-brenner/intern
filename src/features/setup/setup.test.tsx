import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import type { ExistingModelFiles, SelectionBoundary } from '../../lib/bridge';
import { PINNED_MODEL_BYTES, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { SetupState } from '../../types';
// The manifest the installer ships and the backend reads. Importing the real
// file is the point: it is what makes the assertion below a drift guard rather
// than a second copy of the same number.
import modelManifest from '../../../src-tauri/resources/model-manifest.json';

const modelFiles = { modelPath: 'C:\\Models\\intern-q4.gguf' };

function setupSelection(pickExistingModelFiles: () => Promise<ExistingModelFiles | undefined>): SelectionBoundary {
  return {
    pickFiles: async () => [],
    pickFolder: async () => undefined,
    pickExistingModelFiles,
    resolveDrop: async () => ({}),
  };
}

describe('setup and queue controls', () => {
  it('shows an inert loading state while setup is still being read', () => {
    const baseBridge = createInMemoryBridge();
    const bridge = { ...baseBridge, getSetup: () => new Promise<never>(() => {}) };
    render(<App bridge={bridge} />);

    expect(screen.getByRole('status', { name: 'Loading setup' })).toBeVisible();
    expect(screen.queryByRole('button', { name: /Download model/i })).not.toBeInTheDocument();
  });

  // Readable sizes and a percentage, with the exact count kept for support.
  // "123,456,789 of 3,221,225,472 bytes" said neither how far along nor how long.
  it('reports local model download progress in readable sizes, with exact bytes on hand', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'downloading', downloadedBytes: 123_456_789, totalBytes: 3_221_225_472 } })} />);

    const progress = await screen.findByText('118 MiB of 3.00 GiB · 3%');
    expect(progress).toBeVisible();
    expect(progress).toHaveAttribute('title', '123,456,789 of 3,221,225,472 bytes');
  });

  it('quotes only what has arrived while the total is not yet known', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'downloading', downloadedBytes: 2_097_152, totalBytes: 0 } })} />);

    const progress = await screen.findByText('2 MiB');
    expect(progress).toHaveAttribute('title', '2,097,152 bytes');
  });

  it('adds the time left once the download has been moving for a few seconds', async () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    try {
      const total = PINNED_MODEL_BYTES;
      const MiB = 1024 ** 2;
      let setup: SetupState = { state: 'downloading', downloadedBytes: 363 * MiB, totalBytes: total };
      let listener!: (next: SetupState) => void;
      const bridge = { ...createInMemoryBridge(), getSetup: async () => ({ ...setup }), subscribeSetup: async (next: typeof listener) => { listener = next; return () => {}; } };
      vi.setSystemTime(0);
      render(<App bridge={bridge} />);
      expect(await screen.findByText('363 MiB of 1.19 GiB · 29%')).toBeVisible();
      // The first state can show before the screen has subscribed to the
      // ones after it; on a busy runner that left `listener` unset.
      await vi.waitFor(() => expect(listener).toBeTypeOf('function'));

      // Three megabytes a second, by the samples, with 828 MiB still to come.
      for (const [at, mib] of [[5_000, 378], [10_000, 393]] as const) {
        vi.setSystemTime(at);
        setup = { ...setup, downloadedBytes: mib * MiB };
        act(() => listener({ ...setup }));
      }

      expect(await screen.findByText('393 MiB of 1.19 GiB · 32% · about 5 min left')).toBeVisible();
    } finally {
      vi.useRealTimers();
    }
  });

  it('quotes the download size the manifest will actually fetch, not a hardcoded one', async () => {
    const total = modelManifest.files.reduce((sum, file) => sum + file.size, 0);

    // The setup screen is the first thing a new user sees, and it used to
    // announce a hardcoded "approximately 3.27 GB" - a model plus a vision
    // projector from an earlier design - for a download that is one 1.19 GiB
    // file. Both halves are pinned here: the demo total must equal the shipped
    // manifest, and the screen must render whatever total it is given.
    expect(PINNED_MODEL_BYTES).toBe(total);

    render(<App
      bridge={createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: total } })}
      selection={setupSelection(async () => undefined)}
    />);

    expect(await screen.findByText(/Download 1\.19 GiB of model files/i)).toBeVisible();
    expect(screen.queryByText(/3\.27 GB/i)).not.toBeInTheDocument();
  });

  it('explains the local privacy boundary and both setup sources', async () => {
    render(<App
      bridge={createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES } })}
      selection={setupSelection(async () => undefined)}
    />);

    expect(await screen.findByRole('heading', { name: 'Set up Intern' })).toBeVisible();
    expect(screen.getByText(/documents.*stay on this device/i)).toBeVisible();
    expect(screen.getByText(/processing runs fully locally/i)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Download model' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeEnabled();
  });

  it('pauses and resumes processing progress', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Pause queue' }));
    expect(await screen.findByRole('button', { name: 'Resume queue' })).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Resume queue' }));
    expect(await screen.findByRole('row', { name: /Q1 Financials/ })).toHaveTextContent('Reading document…');
    expect(screen.queryByText(/\(0%\)/)).not.toBeInTheDocument();
  });

  it('keeps the queue inaccessible after local setup failure', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'failed', error: 'The local model could not be downloaded.' } })} />);

    expect(await screen.findByRole('main', { name: 'Intern setup' })).toBeVisible();
    expect(screen.queryByRole('navigation', { name: 'Queue navigation' })).not.toBeInTheDocument();
  });

  it('polls staged in-memory download progress and disables the download action', async () => {
    const bridge = createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: 300 }, downloadStepBytes: 100, downloadIntervalMs: 400 });
    render(<App bridge={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Download model' }));

    expect(await screen.findByRole('button', { name: 'Downloading model…' })).toBeDisabled();
    expect(await screen.findByText('0 bytes of 300 bytes · 0%')).toBeVisible();
    await waitFor(() => expect(screen.getByText('100 bytes of 300 bytes · 33%')).toBeVisible(), { timeout: 1_200 });
  });

  it('cancels an active setup and offers resume without losing progress', async () => {
    let setup: SetupState = { state: 'downloading', downloadedBytes: 120, totalBytes: 300 };
    const base = createInMemoryBridge();
    const setupCancel = vi.fn(async () => {
      setup = { state: 'required', downloadedBytes: 120, totalBytes: 300, error: 'MODEL_DOWNLOAD_CANCELED' };
    });
    const bridge = { ...base, getSetup: async () => ({ ...setup }), setupCancel };
    render(<App bridge={bridge} selection={setupSelection(async () => modelFiles)} />);

    expect(await screen.findByRole('button', { name: 'Downloading model…' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeDisabled();
    const cancel = screen.getByRole('button', { name: 'Cancel setup' });
    expect(cancel).toBeEnabled();
    fireEvent.click(cancel);

    expect(await screen.findByRole('button', { name: 'Resume download' })).toBeEnabled();
    expect(screen.getByText('120 bytes of 300 bytes · 40%')).toBeVisible();
    expect(screen.getByRole('status', { name: 'Setup status' })).toHaveTextContent(/progress was saved/i);
    expect(setupCancel).toHaveBeenCalledOnce();
  });

  it('passes only native model paths from the selection boundary to setup', async () => {
    let setup: SetupState = { state: 'required', downloadedBytes: 0, totalBytes: 300 };
    const base = createInMemoryBridge();
    const setupChooseExisting = vi.fn(async () => { setup = { ...setup, state: 'downloading' }; });
    const bridge = { ...base, getSetup: async () => ({ ...setup }), setupChooseExisting };
    render(<App bridge={bridge} selection={setupSelection(async () => modelFiles)} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Choose existing model files' }));

    await waitFor(() => expect(setupChooseExisting).toHaveBeenCalledWith(modelFiles));
    expect(screen.getByRole('button', { name: 'Downloading model…' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeDisabled();
  });

  it('announces a busy setup command error and re-enables setup actions', async () => {
    const base = createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: 300 } });
    const startModelDownload = vi.fn(async () => { throw { code: 'SETUP_BUSY', message: 'a model setup operation is already active' }; });
    render(<App bridge={{ ...base, startModelDownload }} selection={setupSelection(async () => modelFiles)} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Download model' }));

    expect(await screen.findByRole('alert')).toHaveTextContent(/another model setup operation is already active/i);
    expect(screen.getByRole('button', { name: 'Download model' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeEnabled();
  });

  describe('updates', () => {
    const openSettings = async () => {
      const trigger = (await screen.findAllByRole('button', { name: 'Settings' }))[0];
      fireEvent.click(trigger);
    };

    it('checks automatically at launch, and again whenever asked', async () => {
      const checkForUpdate = vi.fn(async () => ({ state: 'current' as const, currentVersion: '0.1.0-alpha.2' }));
      render(<App bridge={{ ...createInMemoryBridge(), checkForUpdate }} />);

      // Once on its own, with nobody touching Settings.
      await waitFor(() => expect(checkForUpdate).toHaveBeenCalledTimes(1));

      await openSettings();
      fireEvent.click(screen.getByRole('button', { name: 'Check for updates' }));
      await waitFor(() => expect(screen.getByRole('status', { name: 'Update status' })).toHaveTextContent(/0\.1\.0-alpha\.2 is the latest release/i));
      // And again for the button's own, separate request.
      expect(checkForUpdate).toHaveBeenCalledTimes(2);
    });

    it('offers to install a newer version and names both versions', async () => {
      const bridge = createInMemoryBridge({ update: { state: 'available', currentVersion: '0.1.0-alpha.2', version: '0.2.0' } });
      const installUpdate = vi.fn(async () => {});
      render(<App bridge={{ ...bridge, installUpdate }} />);
      await openSettings();
      fireEvent.click(screen.getByRole('button', { name: 'Check for updates' }));

      await waitFor(() => expect(screen.getByRole('status', { name: 'Update status' })).toHaveTextContent(/Version 0\.2\.0 is available\. You have 0\.1\.0-alpha\.2/i));
      fireEvent.click(screen.getByRole('button', { name: /Install 0\.2\.0 and restart/i }));
      await waitFor(() => expect(installUpdate).toHaveBeenCalledTimes(1));
    });

    // The reason the whole signing apparatus exists. If this message ever
    // degrades into a generic failure, a user cannot tell "GitHub is down" from
    // "something served me code this build refuses to trust".
    it('says plainly when an update fails its signature check', async () => {
      const checkForUpdate = vi.fn(async () => { throw new Error('signature verification failed'); });
      render(<App bridge={{ ...createInMemoryBridge(), checkForUpdate }} />);
      await openSettings();
      fireEvent.click(screen.getByRole('button', { name: 'Check for updates' }));

      const alert = await screen.findByRole('alert');
      expect(alert).toHaveTextContent(/was not signed by this project's key, so Intern refused it/i);
    });

    it('reports an unreachable endpoint as a network problem, not a rejection', async () => {
      const checkForUpdate = vi.fn(async () => { throw new Error(''); });
      render(<App bridge={{ ...createInMemoryBridge(), checkForUpdate }} />);
      await openSettings();
      fireEvent.click(screen.getByRole('button', { name: 'Check for updates' }));

      const alert = await screen.findByRole('alert');
      expect(alert).toHaveTextContent(/Could not reach GitHub/i);
      expect(alert).not.toHaveTextContent(/signed/i);
    });
  });

  it.each([
    ['MODEL_FILE_INVALID', /did not match the model Intern pins/i],
    ['MODEL_SELF_TEST_FAILED', /local self-test failed/i],
    ['MODEL_SERVER_START_FAILED', /runtime would not start on this computer/i],
    ['MODEL_SERVER_UNHEALTHY', /did not become ready in time/i],
  ])('announces the %s setup failure with recovery guidance', async (error, message) => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'failed', downloadedBytes: 40, totalBytes: 300, error } })} selection={setupSelection(async () => modelFiles)} />);

    expect(await screen.findByRole('alert')).toHaveTextContent(message);
    expect(screen.getByRole('button', { name: 'Try download again' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeEnabled();
  });

  // A runtime that will not start, will not become ready, or answers its
  // self-test wrongly is running a model file that already passed its
  // checksum. These used to show a raw code, or advise downloading 1.2 GB
  // again; they name the log that says what happened instead.
  it.each(['MODEL_SELF_TEST_FAILED', 'MODEL_SERVER_START_FAILED', 'MODEL_SERVER_UNHEALTHY'])('points a %s runtime failure at its log, not at a download', async (error) => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'failed', downloadedBytes: 40, totalBytes: 300, error } })} selection={setupSelection(async () => modelFiles)} />);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('llama-server.log in %LOCALAPPDATA%\\com.intern.app\\logs');
    expect(alert).not.toHaveTextContent(/try the download again|choose a verified model file/i);
    expect(alert).toHaveTextContent(`(${error})`);
  });

  // These messages previously told a user to supply "Q4 or Q8 model and mmproj
  // GGUF files" and reported an "image self-test". This build pins exactly one
  // file and starts the server with --no-mmproj, so that advice named files that
  // do not exist. The names it does use must stay tied to the shipped manifest.
  it('never asks for model files this build does not use', async () => {
    const names = modelManifest.files.map((file) => file.name);
    expect(names).toEqual(['Qwen3.5-2B-Q4_K_M.gguf']);

    for (const error of ['MODEL_FILE_INVALID', 'MODEL_SELF_TEST_FAILED']) {
      const { unmount } = render(<App
        bridge={createInMemoryBridge({ setup: { state: 'failed', downloadedBytes: 40, totalBytes: PINNED_MODEL_BYTES, error } })}
        selection={setupSelection(async () => modelFiles)}
      />);
      const alert = await screen.findByRole('alert');
      expect(alert).not.toHaveTextContent(/mmproj|projector|Q8|image self-test/i);
      unmount();
    }
  });
});
