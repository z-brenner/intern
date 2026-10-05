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
