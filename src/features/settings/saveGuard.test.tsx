import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SettingsDialog } from '../../components/SettingsDialog';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { AppSettings } from '../../types';

/*
  App holds a set of hardcoded defaults until the real settings arrive. When
  the read failed the failure was swallowed, so the dialog showed those
  defaults as though they were the person's own configuration - and Save wrote
  them over a destination, a watched folder, and a machine name.
*/
describe('settings that could not be read', () => {
  it('refuses to save over settings it could not load', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    const getSettings = vi.fn(async () => { throw new Error('The settings file could not be opened.'); });
    render(<App bridge={{ ...base, getSettings, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });

    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    expect(await within(dialog).findByRole('alert')).toHaveTextContent('could not read your settings');
    await waitFor(() => expect(saveSettings).not.toHaveBeenCalled());
  });

  // App's read lands a render after it mounts. A choice made in the dialog in
  // that moment was overwritten by the read, and Save stored the old value.
  it('keeps what the person changed when the settings arrive after the dialog opened', async () => {
    const placeholders: AppSettings = { destination: '', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: '', intakeEnabled: false, processOthersUploads: false, machineLabel: '', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'anthropic', hostedBaseUrl: '', hostedModel: '' };
    const loaded: AppSettings = { ...placeholders, destination: 'D:\\Filed', machineLabel: 'Front desk', intakeLocalOnly: true };
    const onSave = vi.fn(async () => undefined);
    const props = { bridge: createInMemoryBridge(), onSave, onClose: vi.fn(), onCheckForUpdate: async () => ({ state: 'unsupported' as const }), onInstallUpdate: async () => undefined };
    const view = render(<SettingsDialog settings={placeholders} {...props} />);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(within(dialog).getByLabelText('Hosted model with my API key'));

    view.rerender(<SettingsDialog settings={loaded} {...props} />);

    // The read's values replace the placeholders it was not shown; the choice stays.
    expect(within(dialog).getByLabelText('Destination folder')).toHaveValue('D:\\Filed');
    expect(within(dialog).getByLabelText('Hosted model with my API key')).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...loaded, modelSource: 'hosted' }));
  });
});
