// @vitest-environment node
import { execFile } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

/**
 * scripts/realdocs-eval/fetch.py builds the real sets' gold for CI's replays
 * (--gold-only --recording) and fetches their documents for a live run, where
 * one flaky source would otherwise cost a document. Both run here against the
 * real script: the gold from the committed sample, the fetch against a local
 * server that misbehaves the way sources and runners' networks do.
 */

const exec = promisify(execFile);
const FETCH = 'scripts/realdocs-eval/fetch.py';
// The real-document tooling runs on Linux (CI's corpus-replay job, the
// candidate benchmark); a Windows runner has no python3 command.
const POSIX = process.platform !== 'win32';
// Never through a proxy: the server is this process.
const env = { ...process.env, NO_PROXY: '*', no_proxy: '*' };

interface Gold {
  documents: { id: string; source?: unknown }[];
}

describe.runIf(POSIX)('fetch.py --gold-only', () => {
  let out: string;
  beforeAll(async () => {
    out = await mkdtemp(join(tmpdir(), 'realdocs-fetch-'));
  });
  afterAll(async () => {
    await rm(out, { recursive: true, force: true });
  });

  it('writes the sample gold for exactly the documents a recording holds, and fetches nothing', async () => {
    const sample = JSON.parse(await readFile('scripts/realdocs-eval/sample.json', 'utf8')) as Gold;
    await exec('python3', ['-I', FETCH, '--gold-only', join(out, 'all'), 'sample.json'], { env });
    const all = JSON.parse(await readFile(join(out, 'all', 'gold.json'), 'utf8')) as Gold;
    expect(all.documents.map((document) => document.id)).toEqual(sample.documents.map((document) => document.id));
    expect(all.documents.every((document) => document.source === undefined)).toBe(true);
    expect(existsSync(join(out, 'all', 'corpus'))).toBe(false);

    // Listed out of the sample's order: the gold keeps the sample's.
    const held = [sample.documents[5].id, sample.documents[2].id];
    await writeFile(join(out, 'recording.json'), JSON.stringify({ documents: held.map((id) => ({ id })) }));
    const { stdout } = await exec(
      'python3',
      ['-I', FETCH, '--gold-only', '--recording', join(out, 'recording.json'), join(out, 'kept'), 'sample.json'],
      { env },
    );
    expect(stdout).toContain("2 documents' gold");
    const kept = JSON.parse(await readFile(join(out, 'kept', 'gold.json'), 'utf8')) as Gold;
    expect(kept.documents).toEqual(all.documents.filter((document) => held.includes(document.id)));
    expect(kept.documents.map((document) => document.id)).toEqual([sample.documents[2].id, sample.documents[5].id]);
  });

  it('refuses a recording of documents the sample does not hold', async () => {
    await writeFile(join(out, 'stray.json'), JSON.stringify({ documents: [{ id: 'not-in-the-sample' }] }));
    await expect(
      exec('python3', ['-I', FETCH, '--gold-only', '--recording', join(out, 'stray.json'), join(out, 'stray'), 'sample.json'], {
        env,
      }),
    ).rejects.toMatchObject({ code: 1, stderr: expect.stringContaining('holds documents sample.json does not: not-in-the-sample') });
  });
});

describe.runIf(POSIX)('fetch.py get()', () => {
  const requests: Record<string, number> = {};
  let server: Server;
  let base: string;

  beforeAll(async () => {
    server = createServer((request, response) => {
      const path = (request.url ?? '').slice(1);
      const seen = (requests[path] = (requests[path] ?? 0) + 1);
      if (path === 'busy' && seen === 1) {
        response.writeHead(503, { 'Retry-After': '0' }).end();
      } else if (path === 'limited') {
        response.writeHead(429, { 'Retry-After': '0' }).end();
      } else if (path === 'gone') {
        response.writeHead(404).end();
      } else if (path === 'reset' && seen === 1) {
        request.socket.destroy();
      } else if (path === 'slow' && seen === 1) {
        // Never answered: the client's timeout has to end it.
      } else {
        response.writeHead(200).end(`${path}, try ${seen}`);
      }
    });
    await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
    base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  });
  afterAll(async () => {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  });

  it('asks again after a 429, a 5xx, a reset or a timeout, three tries in all, and never after a 404', async () => {
    const driver = `
import json, sys
sys.path.insert(0, "scripts/realdocs-eval")
import fetch
slept, results = [], {}
for path in ["busy", "limited", "gone", "reset", "slow"]:
    try:
        results[path] = fetch.get(sys.argv[1] + "/" + path, wait=7, sleep=slept.append, timeout=1).decode()
    except Exception as error:
        results[path] = " ".join(str(part) for part in (type(error).__name__, getattr(error, "code", "")) if part != "")
print(json.dumps({"results": results, "slept": slept}))
`;
    const { stdout, stderr } = await exec('python3', ['-I', '-c', driver, base], { env });
    const { results, slept } = JSON.parse(stdout);
    expect(results).toEqual({
      busy: 'busy, try 2',
      limited: 'HTTPError 429',
      gone: 'HTTPError 404',
      reset: 'reset, try 2',
      slow: 'slow, try 2',
    });
    expect(requests).toEqual({ busy: 2, limited: 3, gone: 1, reset: 2, slow: 2 });
    // Retry-After 0 is honoured; a network failure waits the default.
    expect(slept).toEqual([0, 0, 0, 7, 7]);
    expect(stderr).toContain(`${base}/busy: HTTP 503, try 1 of 3; asking again in 0 s`);
    expect(stderr).toContain(`${base}/reset: `);
    expect(stderr).not.toContain('/gone:');
  });

  // Each of these fails the same way on every try, so asking again would only
  // hold the document up 40 s before it fails.
  it('never asks again after a failure that is not the network\'s', async () => {
    const driver = `
import json, socket, ssl, sys, urllib.error
sys.path.insert(0, "scripts/realdocs-eval")
import fetch
slept, results = [], {}
# TLS spoken to a server that answers plain HTTP, and a scheme urllib has no handler for.
for name, url in [("tls", sys.argv[1].replace("http://", "https://") + "/tls"), ("scheme", "gopher-x://example.invalid/")]:
    try:
        fetch.get(url, wait=7, sleep=slept.append, timeout=5)
        results[name] = "fetched"
    except Exception as error:
        results[name] = type(error).__name__
transient = {
    "refused": fetch.transient(urllib.error.URLError(ConnectionRefusedError(111, "Connection refused"))),
    "reset": fetch.transient(ConnectionResetError(104, "Connection reset by peer")),
    "timeout": fetch.transient(TimeoutError("timed out")),
    "cut short": fetch.transient(fetch.http.client.IncompleteRead(b"")),
    "name lookup for now": fetch.transient(urllib.error.URLError(socket.gaierror(socket.EAI_AGAIN, "Temporary failure in name resolution"))),
    "no such name": fetch.transient(urllib.error.URLError(socket.gaierror(socket.EAI_NONAME, "Name or service not known"))),
    "certificate": fetch.transient(urllib.error.URLError(ssl.SSLCertVerificationError(1, "certificate verify failed"))),
    "unknown scheme": fetch.transient(urllib.error.URLError("unknown url type: gopher-x")),
}
print(json.dumps({"results": results, "slept": slept, "transient": transient}))
`;
    const { stdout, stderr } = await exec('python3', ['-I', '-c', driver, base], { env });
    const { results, slept, transient } = JSON.parse(stdout);
    expect(results).toEqual({ tls: 'URLError', scheme: 'URLError' });
    expect(slept).toEqual([]);
    expect(stderr).not.toContain('asking again');
    expect(transient).toEqual({
      refused: true,
      reset: true,
      timeout: true,
      'cut short': true,
      'name lookup for now': true,
      'no such name': false,
      certificate: false,
      'unknown scheme': false,
    });
  });
});
