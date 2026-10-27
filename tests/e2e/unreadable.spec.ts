import { expect, test } from '@playwright/test';
import { openDetails } from './grid';

// A dropped document is copied into the input folder, and one Intern cannot
// read waits in review there, renamed NEEDS REVIEW where it lies; a person
// can name it themselves and file it. The browser build stands a file whose
// name says it is password-protected in for one.
test('an unreadable document is renamed for review, named by hand, and filed', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();

  await page.getByRole('region', { name: 'Drag files or folders here to copy them into Input' }).evaluate((dropZone) => {
    const transfer = new DataTransfer();
    transfer.items.add(new File(['encrypted'], 'Lease password.pdf', { type: 'application/pdf' }));
    dropZone.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });

  const row = page.getByRole('row', { name: /NEEDS REVIEW - Lease password\.pdf/ });
  await expect(row).toContainText('Needs review');
  const panel = await openDetails(page, 'NEEDS REVIEW - Lease password.pdf');
  await expect(panel.getByRole('heading', { name: 'Name it yourself' })).toBeVisible();
  await expect(panel.getByText(/password-protected/)).toBeVisible();
  // It starts from the name the file arrived with, without the prefix;
  // every rename needs a date.
  await expect(panel.getByLabel('Filename')).toHaveValue('Lease password');
  await panel.getByLabel('Filename').fill('2026-01-02 Lease');
  await panel.getByRole('button', { name: 'Approve & rename' }).click();

  await page.getByRole('button', { name: 'Completed' }).click();
  await expect(page.getByRole('row', { name: /Lease password\.pdf/ })).toContainText('2026-01-02 Lease.pdf');
});
