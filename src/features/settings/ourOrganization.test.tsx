import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('How Intern names files', { selector: 'summary' }));
  return dialog;
}

describe('your organization\'s names', () => {
  it('keeps what is typed while editing and saves trimmed, non-empty lines', async () => {
    const base = createInMemoryBridge({ settings: { ourNames: ['Contoso Worldwide, Inc.'] } });
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    let dialog = await openSettings();
    const names = within(dialog).getByLabelText('Your organizations');
    expect(names).toHaveValue('Contoso Worldwide, Inc.');
    expect(names).toHaveAccessibleDescription('List each of your company names, one per line. Intern puts the matching name right after the date: 2025-03-26 Vistage Malaysia - Termination Letter for Johnny Appleseed.');

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
    expect(within(dialog).getByLabelText('Your organizations')).toHaveValue('Contoso Worldwide, Inc.\nContoso');
  });

  it('saves an empty list when every line is cleared', async () => {
    const base = createInMemoryBridge({ settings: { ourNames: ['Contoso'] } });
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    const dialog = await openSettings();
    fireEvent.change(within(dialog).getByLabelText('Your organizations'), { target: { value: ' \n' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ ourNames: [] })));
  });

  // An own name under four letters and digits would match far too much, so
  // the backend ignores it. Saved without a word, "EY" looked like it worked.
  it('says which names are too short to be used', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    const names = within(dialog).getByLabelText('Your organizations');

    fireEvent.change(names, { target: { value: 'E.Y.\nPwC\nContoso Worldwide, Inc.\nKPMG\n' } });
    expect(names).toHaveAccessibleDescription(expect.stringContaining('“E.Y.”, “PwC” are too short to match and will not be used. Use the full name as your documents print it.'));

    fireEvent.change(names, { target: { value: 'IBM\nKPMG' } });
    expect(names).toHaveAccessibleDescription(expect.stringContaining('“IBM” is too short to match'));

    fireEvent.change(names, { target: { value: 'KPMG' } });
    expect(within(dialog).queryByText(/too short to match/)).not.toBeInTheDocument();
    expect(names).toHaveAccessibleDescription('List each of your company names, one per line. Intern puts the matching name right after the date: 2025-03-26 Vistage Malaysia - Termination Letter for Johnny Appleseed.');
  });

  /*
    The backend renames what is waiting and announces it with a queue event.
    The queue must show the new name once Settings closes, not only after the
    next unrelated change happens to read it again.
  */
  it('leads a waiting proposal with the organization as soon as it is saved', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const lease = await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i });
    expect(lease).toHaveTextContent('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf');
    // Open in the inspector before the rename, so the name it shows is one
    // it had already taken as the starting draft.
    await openRow(lease);
    expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');

    const dialog = await openSettings();
    fireEvent.change(within(dialog).getByLabelText('Your organizations'), { target: { value: 'TenantCo Inc.' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));

    await waitFor(() => expect(screen.getByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i })).toHaveTextContent('2023-09-15 TenantCo - Lease Agreement with ABC Properties LLC.pdf'));
    // The inspector follows the new proposal rather than offering the old
    // name to approve.
    await waitFor(() => expect(screen.getByLabelText('Filename')).toHaveValue('2023-09-15 TenantCo - Lease Agreement with ABC Properties LLC'));
    // The organization leads the name, so the panel has nothing left out to explain.
    expect(screen.queryByRole('note', { name: 'Filed by the other side' })).not.toBeInTheDocument();
  });
});
