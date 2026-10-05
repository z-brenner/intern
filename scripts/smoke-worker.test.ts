import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';

interface ExpectedFixture { file: string; expected_error?: string | null }

/**
 * The Windows smoke run is the only place the release worker meets the
 * fixtures with PDFium behind it, and it runs on a Windows runner nothing on
 * Linux executes. When encrypted.pdf began failing as PASSWORD_PROTECTED,
 * fixtures/expected.json and the corpus recording moved with it and the
 * smoke script did not: it still demanded PARSE_FAILED, which would have
 * failed the Windows job on every pull request and the release after it.
 * Every code the script expects a rejected fixture to fail with is the code
 * expected.json records for that fixture.
 */
it('expects each rejected smoke fixture to fail with the code expected.json records', async () => {
  const [script, expected] = await Promise.all([
    readFile('scripts/smoke-worker.ps1', 'utf8'),
    readFile('fixtures/expected.json', 'utf8').then(JSON.parse) as Promise<{ fixtures: ExpectedFixture[] }>,
  ]);
  const declared = /function\s+Assert-RejectedFixture\s*\{[\s\S]*?\[string\]\$Code\s*=\s*"([A-Z_]+)"/.exec(script);
  expect(declared, 'Assert-RejectedFixture declares a default code').not.toBeNull();
  const defaultCode = declared![1];

  const calls = [...script.matchAll(/^\s*Assert-RejectedFixture\s+"([^"]+)"(?:\s+(?:-Code\s+)?"([A-Z_]+)")?\s*$/gm)];
  expect(calls.map(([, file]) => file).sort()).toEqual(['encrypted.pdf', 'malformed.pdf']);

  for (const [, file, code] of calls) {
    const fixture = expected.fixtures.find((entry) => entry.file === file);
    expect(fixture, `${file} is in fixtures/expected.json`).toBeDefined();
    expect(code ?? defaultCode, file).toBe(fixture!.expected_error);
  }
});
