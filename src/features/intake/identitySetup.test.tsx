import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, it } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';

// The in-memory bridge mirrors the shipping build: no SharePoint deployment.
// Microsoft upload verification reads the same provisioned identifiers, so
// Settings offers the machine label and the watched folder but no Microsoft
// panel that could only ever show an internal error.
it('keeps the machine label and leaves out Microsoft identity in a build without the deployment', async () => {
  render(<App bridge={createInMemoryBridge()} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Settings' }));
  const dialog = screen.getByRole('dialog', { name: 'Settings' });
  // The shared-intake section appears once Settings knows this build is not managed.
  expect(await within(dialog).findByLabelText("This machine's name")).toBeVisible();
  expect(within(dialog).getByLabelText('Intake folder')).toBeVisible();
  expect(within(dialog).queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
  expect(within(dialog).queryByText(/browser preview/i)).not.toBeInTheDocument();
});
