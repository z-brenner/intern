import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';

const packageInfo = JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf8')) as { version: string };

test('the update summary holds focus, dismisses, and stays dismissed after restarting the page', async ({ page }) => {
  await page.goto('/?whatsNew=1');
  const summary = page.getByRole('dialog', { name: 'What’s new in ATLAS' });
  await expect(summary).toBeVisible();
  const proceed = summary.getByRole('button', { name: 'Continue' });
  await expect(proceed).toBeFocused();
  await page.keyboard.press('Tab');
  // Chromium may focus its own browser chrome between cycles; a second Tab
  // must return to the sole modal control, never to the inert app beneath it.
  await page.keyboard.press('Tab');
  await expect(proceed).toBeFocused();
  await proceed.click();
  await expect(summary).toHaveCount(0);
  await expect(page.getByRole('main', { name: 'ATLAS' })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('intern.lastSeenVersion'))).toBe(packageInfo.version);
  await page.reload();
  await expect(page.getByRole('main', { name: 'ATLAS' })).toBeVisible();
  await expect(summary).toHaveCount(0);
});

// The app's smallest window (src-tauri/tauri.conf.json: 800 by 600). This
// release's summary must fit it whole: a dialog that scrolls takes a Tab
// stop of its own, and focus left Continue for the dialog. It overflowed by
// two pixels there.
test('the update summary fits the smallest window and keeps focus on Continue', async ({ page }) => {
  await page.setViewportSize({ width: 800, height: 600 });
  await page.goto('/?whatsNew=1');
  const summary = page.getByRole('dialog', { name: 'What’s new in ATLAS' });
  await expect(summary).toBeVisible();
  const proceed = summary.getByRole('button', { name: 'Continue' });
  await expect(proceed).toBeFocused();
  expect(await summary.evaluate((node) => node.scrollHeight - node.clientHeight)).toBeLessThanOrEqual(0);
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  await expect(proceed).toBeFocused();
});

test('Escape dismisses and acknowledges the update summary', async ({ page }) => {
  await page.goto('/?whatsNew=1');
  await expect(page.getByRole('dialog', { name: 'What’s new in ATLAS' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem('intern.lastSeenVersion'))).toBe(packageInfo.version);
});
