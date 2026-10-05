import { expect, test } from '@playwright/test';

test('mixed batch can be reviewed, approved, and undone entirely in memory', async ({ page }) => {
  await page.goto('/?fixtureBatch=1');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();

  await page.getByRole('region', { name: 'Drag files or folders here to add to the queue' }).evaluate((dropZone) => {
    const transfer = new DataTransfer();
    transfer.items.add(new File(['fictional invoice'], 'duplicate-invoice-a.pdf', { type: 'application/pdf' }));
    transfer.items.add(new File(['fictional invoice'], 'duplicate-invoice-b.pdf', { type: 'application/pdf' }));
    transfer.items.add(new File(['unsupported'], 'unsupported.csv', { type: 'text/csv' }));
    transfer.items.add(new File(['lock'], '~$nda.docx', { type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' }));
    dropZone.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });

  await expect(page.getByRole('row', { name: /duplicate-invoice-a\.pdf/i })).toContainText('Needs review');
  await expect(page.getByRole('row', { name: /duplicate-invoice-b\.pdf/i })).toContainText('Needs review');
  // Files Intern cannot read are left out of the queue and named, with the
  // reason, instead of refusing the whole drop or sitting there as failures.
  await expect(page.getByRole('note', { name: 'Files not added' })).toHaveText(/Added 2 documents\. Skipped 2: unsupported\.csv \(not a supported format\), ~\$nda\.docx \(a temporary or hidden file\)\./);
  await expect(page.getByRole('row', { name: /unsupported\.csv/i })).toHaveCount(0);
  await expect(page.getByRole('row', { name: /~\$nda\.docx/i })).toHaveCount(0);

  await page.getByRole('button', { name: 'Select duplicate-invoice-b.pdf' }).click();
  await expect(page.getByText(/different path.*separate/i)).toBeVisible();

  await page.getByRole('region', { name: 'Drag files or folders here to add to the queue' }).evaluate((dropZone) => {
    const transfer = new DataTransfer();
    transfer.items.add(new File(['fictional invoice'], 'duplicate-invoice-a.pdf', { type: 'application/pdf' }));
    dropZone.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
  });
  await expect(page.getByRole('row', { name: /duplicate-invoice-a\.pdf/i })).toHaveCount(1);
  await expect(page.getByRole('complementary', { name: 'Review item' }).getByText('duplicate-invoice-a.pdf', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Needs Review' }).click();
  await page.getByRole('button', { name: 'Select duplicate-invoice-a.pdf' }).click();
  await page.getByLabel('Filename').fill('2025-04-30 Invoice INV-2048 from Nimbus Orchard Supply Co.pdf');
  await page.getByLabel('Description').fill('Invoice INV-2048 dated April 30, 2025 for Atlas Threadworks LLC.');
  await page.getByRole('button', { name: 'Approve & rename' }).click();

  await page.getByRole('button', { name: 'Completed' }).click();
  const completed = page.getByRole('row', { name: /duplicate-invoice-a\.pdf/i });
  await expect(completed).toContainText('2025-04-30 Invoice INV-2048 from Nimbus Orchard Supply Co.pdf');
  await completed.getByRole('button', { name: /Select/ }).click();
  await page.getByRole('button', { name: 'Undo' }).click();

  await page.getByRole('button', { name: 'Needs Review' }).click();
  await expect(page.getByRole('row', { name: /duplicate-invoice-a\.pdf/i })).toBeVisible();
});

test('naming your organisation in Settings files a waiting proposal by the other side', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  const lease = page.getByRole('row', { name: /Lease Agreement - 123 Main St\.pdf/i });
  await expect(lease).toContainText('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc.pdf');

  await page.getByRole('button', { name: /Settings/i }).click();
  const dialog = page.getByRole('dialog', { name: /Settings/i });
  // One name per line; the blank line a list picks up is not a name.
  await dialog.getByLabel('Your organisation\'s names').fill('TenantCo Inc.\n\n');
  await dialog.getByRole('button', { name: 'Save settings' }).click();
  await expect(dialog).toBeHidden();

  // The queue shows the new name at once, by the counterparty alone, and the
  // inspector says why the evidence names a party the name does not.
  await expect(lease).toContainText('2023-09-15 Lease Agreement with ABC Properties LLC.pdf');
  await page.getByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' }).click();
  const inspector = page.getByRole('complementary', { name: 'Review item' });
  await expect(inspector.getByLabel('Filename')).toHaveValue('2023-09-15 Lease Agreement with ABC Properties LLC.pdf');
  await expect(inspector.getByRole('note', { name: 'Filed by the other side' })).toHaveText('Filed by the other side: TenantCo Inc. is your organisation, so it is left out of the name. Change this under Settings.');
  await expect(inspector.getByText('TenantCo Inc.', { exact: true }).first()).toBeVisible();

  // Saved as the backend stores it.
  await page.getByRole('button', { name: /Settings/i }).click();
  await expect(page.getByRole('dialog', { name: /Settings/i }).getByLabel('Your organisation\'s names')).toHaveValue('TenantCo Inc.');
});
