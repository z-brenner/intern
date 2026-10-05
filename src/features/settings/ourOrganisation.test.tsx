import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  return screen.findByRole('dialog', { name: 'Settings' });
}

describe('your organisation\'s names', () => {
  it('keeps what is typed while editing and saves trimmed, non-empty lines', async () => {
    const base = createInMemoryBridge({ settings: { ourNames: ['Contoso Worldwide, Inc.'] } });
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    let dialog = await openSettings();
    const names = within(dialog).getByLabelText('Your organisation\'s names');
    expect(names).toHaveValue('Contoso Worldwide, Inc.');
    expect(names).toHaveAccessibleDescription('Your own firm\'s names. When a document names your firm and someone else, the filename and the Party folder use the other side.');

    // A list typed one name per line picks up padding and blank lines; the
    // box shows them as typed, or the caret would jump while typing.
    const typed = '  Contoso Worldwide, Inc. \n\nContoso\n   \n';
    fireEvent.change(names, { target: { value: typed } });
    expect(names).toHaveValue(typed);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ ourNames: ['Contoso Worldwide, Inc.', 'Contoso'] })));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect((await base.getSettings()).ourNames).toEqual(['Contoso Worldwide, Inc.', 'Contoso']);

    dialog = await openSettings();
    expect(within(dialog).getByLabelText('Your organisation\'s names')).toHaveValue('Contoso Worldwide, Inc.\nContoso');
  });

  it('saves an empty list when every line is cleared', async () => {
    const base = createInMemoryBridge({ settings: { ourNames: ['Contoso'] } });
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    const dialog = await openSettings();
    fireEvent.change(within(dialog).getByLabelText('Your organisation\'s names'), { target: { value: ' \n' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ ourNames: [] })));
  });

  /*
    The backend renames what is waiting and announces it with a queue event.
    The queue must show the new name once Settings closes, not only after the
    next unrelated change happens to read it again.
  */
  it('renames a waiting proposal by the other side as soon as it is saved', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const lease = await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i });
    expect(lease).toHaveTextContent('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf');
    // Open in the inspector before the rename, so the name it shows is one
    // it had already taken as the starting draft.
    fireEvent.click(within(lease).getByRole('button', { name: /select/i }));
    expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf');

    const dialog = await openSettings();
    fireEvent.change(within(dialog).getByLabelText('Your organisation\'s names'), { target: { value: 'TenantCo Inc.' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(screen.getByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i })).toHaveTextContent('2023-09-15 Lease Agreement with ABC Properties LLC.pdf'));
    // The inspector follows the new proposal rather than offering the old
    // name to approve.
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 Lease Agreement with ABC Properties LLC.pdf'));
    expect(screen.getByRole('note', { name: 'Filed by the other side' })).toHaveTextContent('TenantCo Inc. is your organisation');
  });
});
