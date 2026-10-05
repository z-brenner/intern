import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

describe('setup with a hosted model', () => {
  it('lets a hosted model stand in for the download', async () => {
    const bridge = createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0 }, hostedKey: 'sk-ant-api03-already-stored-0001', settings: { destination: 'C:\\Filed' } });
    render(<App bridge={bridge} />);

    const setup = await screen.findByRole('main', { name: 'Intern setup' });
    expect(within(setup).getByText(/document text is sent to that service/)).toBeVisible();
    fireEvent.click(within(setup).getByRole('button', { name: 'Set up a hosted model' }));

    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    // The settings read at launch reach an open dialog through an effect that
    // replaces its draft. On a loaded machine that effect was still pending
    // when the hosted choice below was clicked and ran after it, so the local
    // model was saved; change nothing until the saved settings are showing.
    await waitFor(() => expect(within(dialog).getByLabelText('Destination folder')).toHaveValue('C:\\Filed'));
    fireEvent.click(within(dialog).getByLabelText('Hosted model with my API key'));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    // No download happened; the queue is usable on the hosted model.
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeVisible();
    expect(screen.getByText('Hosted model · Text leaves this device')).toBeVisible();
    await waitFor(async () => expect((await bridge.getSetup()).state).toBe('required'));
  });

  it('keeps the download screen when the hosted model is chosen but has no key', async () => {
    const bridge = createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0 } });
    render(<App bridge={bridge} />);
    const setup = await screen.findByRole('main', { name: 'Intern setup' });
    fireEvent.click(within(setup).getByRole('button', { name: 'Set up a hosted model' }));
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(within(dialog).getByLabelText('Hosted model with my API key'));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    expect(await within(dialog).findByRole('alert')).toHaveTextContent('Paste an API key');
    expect(screen.getByRole('main', { name: 'Intern setup' })).toBeVisible();
  });
});
