import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';

const packageInfo = JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf8')) as { version: string };

test('the update summary holds focus, dismisses, and stays dismissed after restarting the page', async ({ page }) => {
  await page.goto('/?whatsNew=1');
  const summary = page.getByRole('dialog', { name: 'What’s new in Intern' });
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
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('intern.lastSeenVersion'))).toBe(packageInfo.version);
  await page.reload();
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(summary).toHaveCount(0);
});

test('Escape dismisses and acknowledges the update summary', async ({ page }) => {
  await page.goto('/?whatsNew=1');
  await expect(page.getByRole('dialog', { name: 'What’s new in Intern' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem('intern.lastSeenVersion'))).toBe(packageInfo.version);
});
