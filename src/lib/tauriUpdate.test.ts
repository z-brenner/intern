import { beforeEach, describe, expect, it, vi } from 'vitest';
import { TauriBridge, type TauriTransport } from './tauriBridge';

const download = vi.fn<() => Promise<void>>();
const install = vi.fn<() => Promise<void>>();

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: async () => ({ version: '0.1.0-alpha.12', body: 'Notes', date: '2026-10-05', download, install }),
}));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: async () => '0.1.0-alpha.11' }));

function recording() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const transport: TauriTransport = {
    invoke: async <T>(command: string, args?: Record<string, unknown>) => {
      calls.push({ command, args });
      return undefined as T;
    },
    listen: async () => () => {},
  };
  return { transport, calls };
}

describe('installing an update', () => {
  beforeEach(() => {
    download.mockReset();
    download.mockImplementation(async () => {});
    install.mockReset();
  });

  // The installer starts Intern again with this process's arguments. Documents
  // "Send to > Intern" named among them must not be added a second time, so
  // the backend is told before the installer can take over.
  it('says the relaunch is coming before the installer runs', async () => {
    const { transport, calls } = recording();
    install.mockImplementation(async () => {
      expect(calls).toEqual([{ command: 'update_relaunch_expected', args: { expected: true } }]);
    });
    const bridge = new TauriBridge(transport);
    await bridge.checkForUpdate();

    await bridge.installUpdate();

    expect(install).toHaveBeenCalledOnce();
    expect(calls).toHaveLength(1);
  });

  it('takes it back when the install fails, so the next launch is a person\'s own', async () => {
    const { transport, calls } = recording();
    install.mockImplementation(async () => { throw new Error('signature rejected'); });
    const bridge = new TauriBridge(transport);
    await bridge.checkForUpdate();

    await expect(bridge.installUpdate()).rejects.toThrow('signature rejected');

    expect(calls).toEqual([
      { command: 'update_relaunch_expected', args: { expected: true } },
      { command: 'update_relaunch_expected', args: { expected: false } },
    ]);
  });

  // Told only once the installer is about to take over: a download that
  // failed, or a step before the install that stopped it, relaunches nothing,
  // and a record left behind could pass a person's own launch within the hour
  // off as the update's.
  it('says nothing when the update never reaches its installer', async () => {
    const { transport, calls } = recording();
    download.mockImplementation(async () => { throw new Error('signature rejected'); });
    const bridge = new TauriBridge(transport);
    await bridge.checkForUpdate();

    await expect(bridge.installUpdate()).rejects.toThrow('signature rejected');
    download.mockImplementation(async () => {});
    await expect(bridge.installUpdate(undefined, async () => { throw new Error('a rename is still moving'); })).rejects.toThrow('a rename is still moving');

    expect(install).not.toHaveBeenCalled();
    expect(calls).toEqual([]);
  });

  it('still installs when the backend cannot be told', async () => {
    const transport: TauriTransport = {
      invoke: async () => { throw new Error('APP_DATA_UNAVAILABLE'); },
      listen: async () => () => {},
    };
    install.mockImplementation(async () => {});
    const bridge = new TauriBridge(transport);
    await bridge.checkForUpdate();

    await bridge.installUpdate();

    expect(install).toHaveBeenCalledOnce();
  });
});
