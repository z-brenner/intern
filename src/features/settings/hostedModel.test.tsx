import { fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

describe('simple processing settings', () => {
  it('offers no API key, provider, or server configuration', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(within(dialog).queryByLabelText('API key')).not.toBeInTheDocument();
    expect(within(dialog).queryByLabelText('API address')).not.toBeInTheDocument();
    expect(within(dialog).queryByLabelText('Provider')).not.toBeInTheDocument();
    expect(within(dialog).queryByLabelText('Hosted model with my API key')).not.toBeInTheDocument();
    const summary = within(dialog).getByText('Local processing', { selector: 'summary' });
    expect(summary.parentElement).not.toHaveAttribute('open');
    fireEvent.click(summary);
    expect(within(dialog).getByRole('group', { name: 'Local model' })).toBeVisible();
  });

  it('keeps optional settings collapsed and uses the same folder names as the tabs', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    const folders = within(dialog).getByRole('group', { name: 'Folders' });
    expect(within(folders).getByText('Queue folder')).toBeVisible();
    expect(within(folders).getByText('Completed folder')).toBeVisible();
    const naming = within(dialog).getByText('Naming and filing', { selector: 'summary' });
    naming.focus();
    fireEvent.keyDown(document, { key: 'Tab' });
    expect(within(dialog).getByText('Learned spellings', { selector: 'summary' })).toHaveFocus();
    for (const title of ['Naming and filing', 'Learned spellings', 'Local processing', 'SharePoint descriptions']) {
      expect(within(dialog).getByText(title, { selector: 'summary' }).parentElement).not.toHaveAttribute('open');
    }
  });
});
