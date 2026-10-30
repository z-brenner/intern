import { expect } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

/*
  The queue table is a grid of cells, and selecting a row no longer opens the
  Details panel: a test that reviews an item clicks its row, then Details, as
  a person does.
*/

type Column = 'original' | 'status' | 'proposed' | 'description' | 'check';

/** A row's cell, found by the row's original filename. */
export function queueCell(page: Page, originalFilename: string, column: Column = 'original'): Locator {
  const original = page.getByRole('gridcell', { name: originalFilename, exact: true }).and(page.locator('[data-column="original"]'));
  if (column === 'original') return original;
  return page.getByRole('row').filter({ has: original }).locator(`[data-column="${column}"]`);
}

export const detailsToggle = (page: Page) => page.getByRole('button', { name: 'Details', exact: true });

/** Select a row (when named) and open the Details panel for it. Resolves with the panel. */
export async function openDetails(page: Page, originalFilename?: string): Promise<Locator> {
  if (originalFilename) await queueCell(page, originalFilename).click();
  const toggle = detailsToggle(page);
  await expect(toggle).toBeEnabled();
  if (await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
  const narrow = await page.evaluate(() => window.matchMedia('(max-width: 1100px)').matches);
  const panel = page.getByRole(narrow ? 'dialog' : 'complementary', { name: 'Review item' });
  await expect(panel).toBeVisible();
  if (originalFilename) await expect(panel).toContainText(originalFilename);
  return panel;
}
