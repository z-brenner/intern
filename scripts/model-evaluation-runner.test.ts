import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';

it('scores the corpus with the exact model the app installs, text-only', async () => {
  const [script, manifest] = await Promise.all([
    readFile('scripts/run-model-evaluation.ps1', 'utf8'),
    readFile('src-tauri/resources/model-manifest.json', 'utf8').then(JSON.parse),
  ]);
  // The evaluation must never pin a model of its own; it reads the manifest the
  // application ships, so evidence always describes what users actually run.
  expect(script).toContain('src-tauri/resources/model-manifest.json');
  expect(script).toContain('$_.role -eq "model"');
  expect(script).toContain('$Spec.sha256');
  expect(script).toContain('intern-evaluate.exe');
  expect(script).not.toContain('.gguf"');
  for (const argument of ['--host', '--api-key', '--parallel', '--ctx-size', '--n-gpu-layers', '--no-mmproj']) {
    expect(script).toContain(`"${argument}"`);
  }
  expect(script).toContain('WorkingSet64');
  expect(script).toContain('docs/qa/model-evaluation.json');
  expect(manifest.files.some((file: { role: string }) => file.role === 'model')).toBe(true);
});

it('the evaluator drives the shipping extraction, distillation, and validation path', async () => {
  const source = await readFile('crates/intern-engine/src/bin/intern-evaluate.rs', 'utf8');
  expect(source).toContain('SupervisedWorker');
  expect(source).toContain('Engine::new');
  expect(source).toContain('analyze_digest');
  // Scoring must compare against the reviewed corpus, including the traps.
  expect(source).toContain('forbidden_dates');
  expect(source).toContain('forbidden_parties');
  expect(source).toContain('acceptable_dates');
  // And it must be able to run the superseded pipeline for comparison.
  expect(source).toContain('legacy_digest');
});

it('the release gate only accepts evidence from the shipping pipeline', async () => {
  const [validator, release] = await Promise.all([
    readFile('scripts/validate-model-evaluation.mjs', 'utf8'),
    readFile('.github/workflows/release.yml', 'utf8'),
  ]);
  expect(validator).toContain("report.pipeline === 'evidence'");
  expect(validator).toContain('date_forbidden');
  expect(release).toContain('intern-evaluate');
  expect(release).toContain('validate-model-evaluation.mjs');
  expect(release.indexOf('run-model-evaluation.ps1')).toBeLessThan(release.indexOf('validate-model-evaluation.mjs'));
});

it('the runner scores with the pipeline the release gate accepts, unless told otherwise', async () => {
  const [script, validator, evaluate, release, qa] = await Promise.all([
    readFile('scripts/run-model-evaluation.ps1', 'utf8'),
    readFile('scripts/validate-model-evaluation.mjs', 'utf8'),
    readFile('crates/intern-engine/src/bin/intern-evaluate.rs', 'utf8'),
    readFile('.github/workflows/release.yml', 'utf8'),
    readFile('.github/workflows/qa.yml', 'utf8'),
  ]);
  const accepted = /report\.pipeline === '([a-z]+)'/.exec(validator)?.[1];
  expect(accepted).toBe('evidence');
  // intern-evaluate writes the --pipeline it was given into the report, so the
  // runner's default is exactly what the gate reads back. A default the gate
  // refuses fails every release only after the whole corpus has been scored.
  const parameter = /\[ValidateSet\(([^)]*)\)\]\[string\]\$Pipeline = "([a-z]+)"/.exec(script);
  expect(parameter).not.toBeNull();
  const [, set, fallback] = parameter!;
  expect(fallback).toBe(accepted);
  expect(set.split(',').map((word) => word.trim().replace(/"/g, ''))).toContain(accepted);
  expect(script).toContain('--pipeline $Pipeline');
  expect(evaluate).toContain(`"${accepted}" => Pipeline::Evidence`);
  // Both workflows that gate on the report rely on that default.
  for (const workflow of [release, qa]) {
    expect(workflow).toContain('run-model-evaluation.ps1');
    expect(workflow).not.toMatch(/-Pipeline\b/);
  }
});
