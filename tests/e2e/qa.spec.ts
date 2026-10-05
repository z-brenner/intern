import { mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { expect, test } from '@playwright/test';

test.describe('whole-product browser QA', () => {
  test.use({ viewport: { width: 1536, height: 1024 } });

  test('captures the accepted primary state at 1536 by 1024 with accessible controls', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('/');

    await expect(page.getByRole('main', { name: 'Intern' })).toBeVisible();
    await expect(page.getByRole('navigation', { name: 'Queue navigation' })).toBeVisible();
    await expect(page.getByRole('complementary', { name: 'Review item' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add files' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add folder' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Pause queue' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Settings' })).toHaveCount(1);
    await expect(page.getByLabel('Filename')).toBeVisible();
    await expect(page.getByLabel('Description')).toBeVisible();

    const geometry = await page.evaluate(() => {
      const box = (selector: string) => document.querySelector(selector)?.getBoundingClientRect();
      return {
        viewport: [window.innerWidth, window.innerHeight],
        headerHeight: box('.app-header')?.height,
        sidebarWidth: box('.sidebar')?.width,
        inspectorWidth: box('.inspector')?.width,
        horizontalOverflow: document.documentElement.scrollWidth - window.innerWidth,
      };
    });
    expect(geometry).toEqual({ viewport: [1536, 1024], headerHeight: 72, sidebarWidth: 230, inspectorWidth: 370, horizontalOverflow: 0 });

    const addFiles = page.getByRole('button', { name: 'Add files' });
    await addFiles.focus();
    const focusStyle = await addFiles.evaluate((element) => {
      const style = getComputedStyle(element);
      return { width: style.outlineWidth, style: style.outlineStyle, color: style.outlineColor };
    });
    expect(Number.parseFloat(focusStyle.width)).toBeGreaterThanOrEqual(2);
    expect(focusStyle.style).not.toBe('none');
    expect(focusStyle.color).not.toBe('transparent');

    const statusContrast = await page.evaluate(() => {
      const luminance = (color: string) => {
        const channels = color.match(/[\d.]+/g)?.slice(0, 3).map((value) => Number(value) / 255) ?? [];
        const [red, green, blue] = channels.map((value) => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4);
        return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
      };
      const ratio = (selector: string) => {
        const element = document.querySelector(selector);
        if (!element) return 0;
        const foreground = luminance(getComputedStyle(element).color);
        const rowColor = getComputedStyle(element.closest('tr') ?? element).backgroundColor;
        const background = luminance(rowColor === 'rgba(0, 0, 0, 0)' ? 'rgb(255, 255, 255)' : rowColor);
        return (Math.max(foreground, background) + 0.05) / (Math.min(foreground, background) + 0.05);
      };
      return { review: ratio('.status.review'), waiting: ratio('.status.waiting') };
    });
    expect(statusContrast.review).toBeGreaterThanOrEqual(4.5);
    expect(statusContrast.waiting).toBeGreaterThanOrEqual(4.5);

    const filename = page.getByLabel('Filename');
    await filename.focus();
    await expect(filename).toBeFocused();

    // The whole proposed name is on screen with the caret in it. A one-line
    // input scrolled a long name to its tail on focus ("etween ABC Properties
    // LLC and TenantCo Inc.pdf"), and this test had to scroll it back by hand
    // before the capture; the field now wraps and grows instead.
    await expect(filename).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');
    const field = await filename.evaluate((element: HTMLTextAreaElement) => ({
      overflowY: element.scrollHeight - element.clientHeight,
      overflowX: element.scrollWidth - element.clientWidth,
      lines: Math.round((element.clientHeight - Number.parseFloat(getComputedStyle(element).paddingTop) - Number.parseFloat(getComputedStyle(element).paddingBottom)) / Number.parseFloat(getComputedStyle(element).lineHeight)),
    }));
    expect(field.overflowY).toBeLessThanOrEqual(0);
    expect(field.overflowX).toBeLessThanOrEqual(0);
    expect(field.lines).toBeGreaterThan(1);
    await expect(page.locator('.filename-extension')).toHaveText('.pdf');

    if (process.env.INTERN_QA_CAPTURE === '1') {
      const capture = resolve('docs/qa/latest-implementation.png');
      await mkdir(dirname(capture), { recursive: true });
      await page.screenshot({ path: capture, animations: 'disabled', fullPage: false });
    }
  });

  test('keeps ready, active, and completed queue states actionable without persistent clutter', async ({ page }) => {
    await page.goto('/');

    await expect(page.getByRole('button', { name: 'Apply all ready' })).toBeVisible();
    await page.getByRole('button', { name: 'Select Q1 Financials.pdf' }).click();
    await expect(page.getByRole('button', { name: 'Cancel processing' })).toBeVisible();
    await page.getByRole('button', { name: 'Completed' }).click();
    await expect(page.getByRole('button', { name: 'Clear history' })).toBeVisible();
  });

  // The Tauri window opens at 1200x800. The shell used to grow with its
  // content, so the decision buttons sat at y=931 - below the fold on first
  // launch, with the seeded review item already open beside the queue.
  test('approve is visible at the default window', async ({ page }) => {
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.goto('/');
    const inspector = page.getByRole('complementary', { name: 'Review item' });
    await expect(inspector.getByLabel('Filename')).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc');

    await expect(inspector.getByRole('button', { name: 'Approve & rename' })).toBeInViewport({ ratio: 1 });
    await expect(inspector.getByRole('button', { name: 'Keep original' })).toBeInViewport({ ratio: 1 });
    // The page itself never scrolls; the table and the inspector do.
    expect(await page.evaluate(() => document.documentElement.scrollHeight - window.innerHeight)).toBeLessThanOrEqual(0);
  });

  // With a long queue, selecting a late row used to scroll the whole page:
  // the inspector's content stayed at the top, thousands of pixels above, and
  // the header and navigation scrolled away with it.
  test('inspector visible after selecting the last of 60', async ({ page }) => {
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.goto('/');
    await expect(page.getByRole('row')).toHaveCount(9);
    // Eight seeded rows in the Queue view, plus 52 dropped.
    await page.getByRole('region', { name: 'Drag files or folders here to add to the queue' }).evaluate((zone) => {
      const transfer = new DataTransfer();
      for (let index = 1; index <= 52; index += 1) transfer.items.add(new File(['scan'], `Scan ${String(index).padStart(2, '0')}.pdf`, { type: 'application/pdf' }));
      zone.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer }));
    });
    await expect(page.getByRole('row')).toHaveCount(61);
    await expect(page.getByText('60 items')).toBeVisible();

    await page.getByRole('button', { name: 'Select Employment Agreement - John Smith.pdf' }).click();
    await page.getByRole('button', { name: 'Select Scan 52.pdf' }).click();

    const inspector = page.getByRole('complementary', { name: 'Review item' });
    await expect(inspector).toContainText('Scan 52.pdf');
    await expect(inspector.getByRole('heading', { name: 'Review item' })).toBeInViewport({ ratio: 1 });
    await expect(page.getByRole('row', { name: /Scan 52\.pdf/ })).toBeInViewport();
    expect((await page.locator('.app-header').boundingBox())?.y).toBe(0);
    await expect(page.getByRole('navigation', { name: 'Queue navigation' })).toBeInViewport();
    // The column headings stay put above the rows that scrolled under them.
    const heading = await page.getByRole('columnheader', { name: 'Status' }).boundingBox();
    const table = await page.locator('.table-wrap').boundingBox();
    expect(heading && table && Math.abs(heading.y - table.y)).toBeLessThanOrEqual(1);
  });

  // The banner was an unplaced grid child: auto-placement put it in the
  // 230x72 top-left cell, pushed the header to the second row, and painted
  // its Install and Not now buttons underneath the header.
  test('update banner does not overlap header', async ({ page }) => {
    await page.setViewportSize({ width: 1200, height: 800 });
    await page.goto('/?update=available');
    const banner = page.getByRole('status', { name: 'Update available' });
    const install = banner.getByRole('button', { name: /^Install .+ and restart$/ });
    await expect(install).toBeVisible();

    const [bannerBox, headerBox] = await Promise.all([banner.boundingBox(), page.locator('.app-header').boundingBox()]);
    expect(bannerBox).toMatchObject({ x: 0, y: 0, width: 1200 });
    expect(headerBox?.height).toBe(72);
    expect(headerBox!.y).toBeGreaterThanOrEqual(bannerBox!.y + bannerBox!.height);
    await expect(banner.getByRole('button', { name: 'Not now' })).toBeInViewport({ ratio: 1 });

    // A real click: Playwright refuses one that something else would receive.
    await install.click();
    await expect(banner.getByRole('alert')).toContainText('only available in the desktop application');
    await expect(page.getByRole('button', { name: 'Approve & rename' })).toBeInViewport({ ratio: 1 });
  });

  test('preserves labels, focus targets, and a non-overflowing inspector drawer at 1024 pixels', async ({ page }) => {
    await page.setViewportSize({ width: 1024, height: 768 });
    await page.goto('/');
    const navigation = page.locator('.sidebar');
    // This used to open on the drawer, because the selection Intern seeds so
    // the panel is not empty opened it. A person who has clicked nothing is
    // owed their queue, so the drawer is opened here the way one is opened.
    await expect(navigation).not.toHaveAttribute('inert', '');
    await expect(page.getByRole('complementary', { name: 'Review item' })).toBeVisible();
    // Collapsed, the labels are tooltips; the accessible names also carry the counts.
    for (const name of ['Queue', 'Needs Review', 'Completed', 'Settings']) {
      await expect(navigation.locator(`button[title="${name}"]`)).toBeVisible();
    }
    await expect(navigation.getByRole('button', { name: 'Needs Review, 1' })).toBeVisible();
    await expect(navigation.locator('button[data-view="review"] .nav-badge')).toHaveText('1');
    await expect(navigation.locator('button[data-view="review"] .nav-badge')).toBeVisible();
    await page.getByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' }).click();
    await expect(navigation).toHaveAttribute('inert', '');
    const drawer = page.getByRole('dialog', { name: 'Review item' });
    await expect(drawer).toBeVisible();
    await expect(page.getByLabel('Filename')).toBeFocused();
    const geometry = await page.evaluate(() => {
      const sidebar = document.querySelector('.sidebar')?.getBoundingClientRect();
      const inspector = document.querySelector('.inspector')?.getBoundingClientRect();
      return {
        sidebarWidth: sidebar?.width,
        inspectorWidth: inspector?.width,
        inspectorRight: inspector ? window.innerWidth - inspector.right : undefined,
        horizontalOverflow: document.documentElement.scrollWidth - window.innerWidth,
      };
    });
    expect(geometry).toEqual({ sidebarWidth: 64, inspectorWidth: 370, inspectorRight: 0, horizontalOverflow: 0 });

    const lastDrawerAction = page.getByRole('button', { name: 'More review actions' });
    await lastDrawerAction.focus();
    await page.keyboard.press('Tab');
    await expect(page.getByRole('button', { name: 'Close review' })).toBeFocused();
    await page.keyboard.press('Escape');

    const trigger = page.getByRole('button', { name: 'Select Lease Agreement - 123 Main St.pdf' });
    await trigger.click();
    await expect(page.getByRole('dialog', { name: 'Review item' })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(trigger).toBeFocused();
  });
});
