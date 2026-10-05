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
    const retracted = /no background poll|no timer|only when you press|only two network requests/i;
    for (const { name, html } of pages) {
      // The words a reader sees, and the markup as well, whitespace collapsed:
      // an attribute, an SVG label, or a comment is where an old promise
      // survives a rewrite of the prose. Only the offending phrase is
      // reported, not the whole page.
      expect(retracted.exec(text(html))?.[0], name).toBeUndefined();
      expect(retracted.exec(html.replace(/\s+/g, ' '))?.[0], `${name} markup`).toBeUndefined();
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
      for (const promise of words.matchAll(/never leaves? the (machine|computer)|no remote processing|start to finish/gi)) {
        const following = words.slice(promise.index, promise.index + promise[0].length + 80);
        expect(following, `${name}: "${promise[0]}"`).toMatch(/unless you turn on a hosted model/i);
      }
    }
  });

  it('lists every kind of request on the download page', async () => {
    const index = text(await readFile('site/index.html', 'utf8'));
    for (const request of ['Model file', 'Update check', 'Hosted model, if you choose one', 'Microsoft sign-in and Graph, provisioned builds only']) {
      expect(index).toContain(request);
    }
  });

  // The list is for the IT reviewer allow-listing traffic, so it names hosts,
  // read from the one function every Microsoft request has to pass. It once
  // named Graph alone, while sign-in and every token renewal go elsewhere.
  it('names every Microsoft host a provisioned build may contact, on the download page and in the README', async () => {
    const transport = await readFile('crates/intern-intake/src/microsoft/transport.rs', 'utf8');
    const allowed = /pub fn deployment_allows_endpoint[\s\S]*?\n\}\n/.exec(transport)?.[0] ?? '';
    const hosts = [...allowed.matchAll(/\(Some\("([\w.-]+)"\),/g)].map((match) => match[1]);
    expect(hosts).toContain('graph.microsoft.com');
    expect(hosts.length).toBeGreaterThan(1);
    const [index, readme] = await Promise.all([readFile('site/index.html', 'utf8'), readFile('README.md', 'utf8')]);
    for (const host of hosts) {
      expect(text(index), host).toContain(host);
      expect(readme, host).toContain(host);
    }
  });

  // Unticking the automatic check stops only the check. A hosted model is
  // sent each arriving document without a click, and a provisioned build asks
  // Microsoft about each upload, so the answer to "nothing on its own" has to
  // say both.
  it('says what else contacts the network when asked how to stop all unrequested traffic', async () => {
    const guide = text(await readFile('site/guide.html', 'utf8'));
    const start = guide.indexOf('Intern must not contact anything on its own');
    expect(start).toBeGreaterThan(0);
    const answer = guide.slice(start, guide.indexOf('NAMING AND THE HOSTED MODEL', start));
    expect(answer).toContain('Check for updates automatically');
    expect(answer).toMatch(/hosted/i);
    expect(answer).toMatch(/provisioned[\s\S]*Microsoft|Microsoft[\s\S]*provisioned/);
    expect(guide).not.toContain('makes no request it was not asked to make');
  });

  // Every build compiles the Microsoft sign-in and Graph client; no feature
  // flag leaves it out. What keeps the published build off Microsoft's servers
  // is the deployment it bundles, which is switched off. The guide once said
  // the published build "has no such code" and "no Graph API client", which a
  // reviewer running `strings` on the installer would find to be false.
  it('says the published build has no deployment, not that it lacks the Microsoft code', async () => {
    const [auth, deployment] = await Promise.all([
      readFile('crates/intern-intake/src/microsoft/auth.rs', 'utf8'),
      readFile('src-tauri/resources/sharepoint-deployment.json', 'utf8').then(JSON.parse),
    ]);
    expect(auth).toContain('login.microsoftonline.com');
    expect(deployment.enabled).toBe(false);
    const absent = /no such code|no graph api client|no (microsoft|graph|sign-in|upload) code/i;
    for (const { name, html } of await sitePages()) {
      expect(absent.exec(text(html))?.[0], name).toBeUndefined();
    }
    expect(absent.exec(await readFile('README.md', 'utf8'))?.[0], 'README.md').toBeUndefined();
    const guide = text(await readFile('site/guide.html', 'utf8'));
    expect(guide).toMatch(/published here never talks to Microsoft's servers: it ships with no deployment/);
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
