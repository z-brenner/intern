import { expect, test } from '@playwright/test';

// A document Intern cannot read waits in review, and a person can name it
// themselves and file it. The browser build stands a file whose name says it
// is password-protected in for one. A dropped file is not in the input
// folder, so it keeps its name: only files there are renamed NEEDS REVIEW.
test('an unreadable document waits in review, is named by hand, and filed', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();

  await page.getByRole('region', { name: 'Drag files or folders here to add to the queue' }).evaluate((dropZone) => {
    const transfer = new DataTransfer();
    transfer.items.add(new File(['encrypted'], 'Lease password.pdf', { type: 'application/pdf' }));
    dropZone.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });

  const row = page.getByRole('row', { name: /Lease password\.pdf/ });
  await expect(row).toContainText('Needs review');
  await expect(page.getByText('NEEDS REVIEW - Lease password.pdf')).toHaveCount(0);
  await page.getByRole('button', { name: 'Select Lease password.pdf' }).click();
  const panel = page.getByRole('complementary', { name: 'Review item' });
  await expect(panel.getByRole('heading', { name: 'Name it yourself' })).toBeVisible();
  await expect(panel.getByText(/password-protected/)).toBeVisible();
  // It starts from the name the file arrived with; every rename needs a date.
  await expect(panel.getByLabel('Filename')).toHaveValue('Lease password');
  await panel.getByLabel('Filename').fill('2026-01-02 Lease');
  await panel.getByRole('button', { name: 'Approve & rename' }).click();

  await page.getByRole('button', { name: 'Completed' }).click();
  await expect(page.getByRole('row', { name: /Lease password\.pdf/ })).toContainText('2026-01-02 Lease.pdf');
});
