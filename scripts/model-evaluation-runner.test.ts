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
  // One text model per tier, and the runner scores one of them, asked the
  // way the app asks it.
  for (const model of manifest.models as { files: { role: string }[] }[]) {
    expect(model.files.filter((file) => file.role === 'model')).toHaveLength(1);
  }
  expect(script).toContain('$_.tier -eq $Tier');
  expect(script).toContain('--prompt-profile');
});

// Each tier is scored on the llama.cpp build the app runs it on: upstream
// llama.cpp from the runtime's llama-upstream folder for both Qwen3.5 models;
// the release and QA runs score the accurate one. The runtime's root holds no
// llama-server. Both workflows hand over the runtime root.
it('scores each tier on the llama-server the app runs it on', async () => {
  const [script, release, qa] = await Promise.all([
    readFile('scripts/run-model-evaluation.ps1', 'utf8'),
    readFile('.github/workflows/release.yml', 'utf8'),
    readFile('.github/workflows/qa.yml', 'utf8'),
  ]);
  expect(script).toMatch(/\[ValidateSet\("fast", "accurate"\)\]\[string\]\$Tier = "accurate"/);
  expect(script).toContain('$ServerRelative = "llama-upstream/llama-server.exe"');
  expect(script).toContain('$Server = [IO.Path]::GetFullPath((Join-Path $Runtime $ServerRelative))');
  expect(script).not.toContain('Join-Path $Runtime "llama-server.exe"');
  expect(script).toContain('Start-Process -FilePath $Server ');
  for (const workflow of [release, qa]) {
    expect(workflow).toMatch(/-RuntimeDirectory \$env:INTERN(?:_QA)?_RUNTIME_STAGE `/);
    expect(workflow).not.toMatch(/run-model-evaluation\.ps1[^\n]*-Tier\b/);
  }
});

it('the evaluator drives the shipping extraction, distillation, and validation path', async () => {
  const source = await readFile('crates/intern-engine/src/bin/intern-evaluate.rs', 'utf8');
  expect(source).toContain('SupervisedWorker');
  expect(source).toContain('Engine::new');
  // The digest pipeline is fitted and retried as the app's engine does it.
  expect(source).toContain('prepare_digest');
  expect(source).toContain('refit_digest_halved');
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

// Each model is launched with flags of its own
// (tiers::tier_server_flags in crates/intern-engine/src/tiers.rs). Every
// script that starts a model the way the app does must pass that tier's
// flags, no more and no fewer, or the QA and release scores, the
// recordings of record and the memory probe describe a server the app
// never runs.
it('starts each model with its own flags wherever the app launch is copied', async () => {
  const tiers = await readFile('crates/intern-engine/src/tiers.rs', 'utf8');
  const quoted = (text: string) => [...text.matchAll(/"([^"]+)"/g)].map((match) => `"${match[1]}"`).join(', ');
  const rows: Record<string, string> = {};
  for (const [tier, variant] of [['accurate', 'Accurate'], ['fast', 'Fast']] as const) {
    const row = tiers.match(new RegExp(`ModelTier::${variant} => &\\[([^\\]]*)\\]`));
    expect(row, `the ${tier} tier has a row in tier_server_flags`).not.toBeNull();
    rows[tier] = quoted(row![1]);
  }
  expect(rows.accurate).toContain('"--no-repack"');
  const map = `$TierFlags = @{ accurate = @(${rows.accurate}); fast = @(${rows.fast}) }`;
  for (const path of ['scripts/run-model-evaluation.ps1', 'scripts/record-corpus.ps1', 'scripts/probe-windows-memory.ps1']) {
    const script = await readFile(path, 'utf8');
    expect(script, `${path} carries both tiers' flags`).toContain(map);
    expect(script, `${path} passes the tier's flags`).toMatch(/\$\w+ \+= \$TierFlags\[\$Tier\]/);
  }
});
