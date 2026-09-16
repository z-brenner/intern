/**
 * Guided SharePoint onboarding, driven end to end in the browser build against
 * the in-memory bridge's fake deployment (`?sharePoint=fake`, read only by the
 * Vite dev server - see `src/BrowserApp.tsx`). It proves the interface a person
 * walks through, not Microsoft, OneDrive, or the backend underneath it.
 *
 * The fake takes one Microsoft poll to connect and three library rescans to
 * appear, each on the real five-second interval, so this run waits on the same
 * clock a person would.
 */
import { expect, test, type Page } from '@playwright/test';

const setup = (page: Page) => page.getByRole('main', { name: 'Intern setup' });

/**
 * The same accessibility checks the QA suite makes of the main window, applied
 * to each onboarding step: nothing asks for a deployment identifier, the step's
 * heading receives focus, the rail says which step is current, Tab from the
 * heading reaches the step's action with a visible focus ring, and nothing
 * scrolls sideways.
 */
async function expectAccessibleStep(page: Page, heading: string, stepLabel: string, action: string) {
  const main = setup(page);
  await expect(main.getByRole('heading', { level: 1, name: heading })).toBeFocused();
  await expect(main.locator('input, textarea, select')).toHaveCount(0);
  await expect(page.getByLabel(/tenant|client|drive|folder|path|email|site/i)).toHaveCount(0);
  await expect(page.getByRole('textbox')).toHaveCount(0);
  await expect(main.getByRole('list', { name: 'Setup steps' }).locator('[aria-current="step"]')).toContainText(stepLabel);

  // Keyboard, not a scripted focus: after a mouse click Chrome only draws a
  // focus ring for focus that the keyboard moved.
  await expect(main.getByRole('button', { name: action })).toBeEnabled();
  await page.keyboard.press('Tab');
  await expect(main.getByRole('button', { name: action })).toBeFocused();
  const ring = await main.getByRole('button', { name: action }).evaluate((element) => {
    const style = getComputedStyle(element);
    return { width: style.outlineWidth, style: style.outlineStyle, color: style.outlineColor };
  });
  expect(Number.parseFloat(ring.width)).toBeGreaterThanOrEqual(2);
  expect(ring.style).not.toBe('none');
  expect(ring.color).not.toBe('transparent');

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBe(0);
}

test('a person sets up Intern for the fixed SharePoint library without typing any identifier', async ({ page }) => {
  test.slow();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/?sharePoint=fake');

  // Welcome.
  await expect(setup(page)).toBeVisible();
  await expectAccessibleStep(page, "Your team's documents, filed for you", 'Welcome', 'Set up Intern');
  await expect(setup(page)).toContainText(/only documents you upload/i);
  await page.getByRole('button', { name: 'Set up Intern' }).click();

  // The fake's model is already downloaded, so the model step is skipped.
  await expectAccessibleStep(page, 'Connect your Microsoft account', 'Microsoft account', 'Connect Microsoft account');
  await expect(page.getByRole('heading', { name: 'Get the local model' })).toHaveCount(0);
  const connect = page.getByRole('button', { name: 'Connect Microsoft account' });
  await expect(connect).toBeEnabled();
  await connect.click();

  // Device sign-in, then the verified account to confirm.
  await expect(page.getByRole('status', { name: 'Microsoft sign-in' })).toBeVisible();
  await expect(setup(page).getByText('ABCD-EFGH')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Open Microsoft sign-in' })).toBeVisible();
  await expect(page.getByText('pat.lee@contoso.example')).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText(/Is this the account you use to upload documents/i)).toBeVisible();
  await page.getByRole('button', { name: 'Yes, this is my account' }).click();

  // OneDrive sync: asked for, then pending while Intern rescans.
  await expectAccessibleStep(page, 'Sync the Files library', 'Library sync', 'Sync Files with OneDrive');
  await page.getByRole('button', { name: 'Sync Files with OneDrive' }).click();
  const waiting = page.getByRole('status', { name: 'Library sync' });
  await expect(waiting).toContainText(/Waiting for OneDrive/i);
  await expect(page.getByText(/OneDrive may ask you to confirm/i)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Open SharePoint' })).toBeVisible();
  await expect(setup(page).getByRole('link')).toHaveCount(0);

  // Ready, then activation.
  await expect(page.getByRole('heading', { name: 'Turn on filing' })).toBeVisible({ timeout: 45_000 });
  await expectAccessibleStep(page, 'Turn on filing', 'Turn on filing', 'Turn on filing');
  await expect(setup(page)).toContainText(/Upload documents directly into Files\/Inbox/);
  await page.getByRole('button', { name: 'Turn on filing' }).click();

  // Finished.
  await expectAccessibleStep(page, 'Watching Files/Inbox. Filing your documents into Files/Filed.', 'Finished', 'Open Intern');
  await expect(setup(page)).toContainText('Pat Lee');
  await expect(setup(page)).toContainText(/system tray/i);
  await page.getByRole('button', { name: 'Open Intern' }).click();

  // The app.
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Queue navigation' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Set up Intern' })).toHaveCount(0);
});

test('the default page opens the app, not onboarding', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Set up Intern' })).toHaveCount(0);
});
