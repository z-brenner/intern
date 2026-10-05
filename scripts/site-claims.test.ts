import { readFile, readdir } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';

/**
 * The download page and the guide are what a privacy-minded downloader, or an
 * IT reviewer allow-listing traffic, actually reads. They once promised "no
 * background poll and no timer" and "the only two network requests, ever"
 * while the app checked for updates at every start and every six hours, and
 * said documents never leave the machine without mentioning the opt-in hosted
 * model (CI_RELEASE_DOCS-4). These tests tie the copy to the code.
 */

async function sitePages() {
  const names = (await readdir('site')).filter((name) => name.endsWith('.html')).sort();
  return Promise.all(names.map(async (name) => ({ name, html: await readFile(`site/${name}`, 'utf8') })));
}

/** The words a reader sees: markup, scripts and styles removed, whitespace collapsed. */
function text(html: string) {
  return html
    .replace(/<(script|style)[\s\S]*?<\/\1>/g, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/\s+/g, ' ');
}

/** The automatic check's interval in hours, read from the constant the app schedules it with. */
async function pollIntervalHours() {
  const app = await readFile('src/App.tsx', 'utf8');
  const definition = /export const UPDATE_POLL_INTERVAL_MS = ([\d\s*_]+);/.exec(app);
  expect(definition, 'src/App.tsx defines UPDATE_POLL_INTERVAL_MS as a product of numbers').not.toBeNull();
  const milliseconds = definition![1].split('*').reduce((product, factor) => product * Number(factor.replaceAll('_', '').trim()), 1);
  return milliseconds / (60 * 60 * 1000);
}

describe('what the site says about the network', () => {
  it('never claims there is no automatic update check while the app schedules one', async () => {
    const hours = await pollIntervalHours();
    expect(hours).toBe(6);
    const pages = await sitePages();
    expect(pages.map((page) => page.name)).toEqual(['guide.html', 'index.html']);
    for (const { name, html } of pages) {
      expect(text(html), name).not.toMatch(/no background poll|no timer|only when you press|only two network requests/i);
      // Both pages describe the check, so both must name its schedule.
      expect(text(html), name).toContain(`every ${hours} hours`);
    }
  });

  it('says the automatic check can be switched off, with the label the setting carries', async () => {
    const settings = await readFile('src/components/SettingsDialog.tsx', 'utf8');
    const label = 'Check for updates automatically';
    expect(settings).toContain(`${label} (when Intern starts and every 6 hours)`);
    for (const { name, html } of await sitePages()) expect(text(html), name).toContain(label);
  });

  it('mentions the hosted model wherever it promises that documents stay on the machine', async () => {
    for (const { name, html } of await sitePages()) {
      const words = text(html);
      expect(words, name).toMatch(/hosted model/i);
      // Each promise is qualified in the same sentence, not somewhere else on the page.
      for (const promise of words.matchAll(/never leaves? the (machine|computer)|no remote processing/gi)) {
        const following = words.slice(promise.index, promise.index + promise[0].length + 80);
        expect(following, `${name}: "${promise[0]}"`).toMatch(/unless you turn on a hosted model/i);
      }
    }
  });

  it('lists every kind of request on the download page', async () => {
    const index = text(await readFile('site/index.html', 'utf8'));
    for (const request of ['Model file', 'Update check', 'Hosted model, if you choose one', 'Microsoft Graph, provisioned builds only']) {
      expect(index).toContain(request);
    }
  });
});

describe('what the site says about installing', () => {
  it('warns about SmartScreen and Smart App Control before anyone runs the installer', async () => {
    for (const { name, html } of await sitePages()) {
      const words = text(html);
      expect(words, name).toContain('not Authenticode signed');
      expect(words, name).toContain('Windows protected your PC');
      expect(words, name).toMatch(/More info[\s\S]{0,40}Run anyway/);
      expect(words, name).toContain('Smart App Control');
    }
  });
});

// CI_RELEASE_DOCS-5. The guide taught "pick the folder in Settings and enable
// Watch a folder", which in the published build leaves every document held as
// an unverified upload.
describe('what the guide says about setting up a folder', () => {
  it('teaches the step-by-step setup for synced folders and the private local box for plain ones', async () => {
    const guide = text(await readFile('site/guide.html', 'utf8'));
    expect(guide).toContain('Set up a folder step by step…');
    expect(guide).toContain('only you add documents to');
    expect(guide).toContain('This is a private local intake');
    expect(guide).toMatch(/provisioned/);
    expect(guide).not.toMatch(/In Settings, pick that folder/);
    expect(guide).not.toMatch(/That's it\. Files that appear in the folder get processed/);
    // The same labels the app shows, so a reader can find them.
    const [settings, setup] = await Promise.all([
      readFile('src/components/SettingsDialog.tsx', 'utf8'),
      readFile('src/components/FolderSetupFlow.tsx', 'utf8'),
    ]);
    expect(settings).toContain('Set up a folder step by step…');
    expect(settings).toContain('This is a private local intake, not a shared or synced folder');
    expect(guide).toContain('This is a private local intake, not a shared or synced folder');
    expect(setup).toContain('Check again');
    expect(guide).toContain('Check again');
  });
});
