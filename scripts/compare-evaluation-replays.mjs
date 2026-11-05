import { readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

/** Diagnostics for the same recording under two validators; never release acceptance. */
export function compareReplays(before, after) {
  for (const report of [before, after]) {
    if (report.mode !== 'replay' || !Array.isArray(report.records)) throw new Error('comparison requires replay reports');
    if (report.records.some((row) => row.stale || ['stale', 'unrecorded', 'source_changed'].includes(row.status))) {
      throw new Error('comparison refuses stale or missing replay inputs');
    }
    const names = report.records.map((row) => row.file);
    if (!names.length || names.some((name) => typeof name !== 'string') || new Set(names).size !== names.length) {
      throw new Error('comparison requires unique fixture names');
    }
  }
  const right = new Map(after.records.map((row) => [row.file, row]));
  if (before.records.length !== right.size || before.records.some((row) => !right.has(row.file))) {
    throw new Error('comparison requires the same fixtures');
  }
  const emissionFields = ['status', 'filename', 'description', 'proposal', 'token_confidence'];
  const changes = [];
  for (const left of before.records) {
    const next = right.get(left.file);
    const changedEmissions = emissionFields.filter((key) => JSON.stringify(left[key]) !== JSON.stringify(next[key]));
    const readinessChanged = left.readiness !== next.readiness;
    const reasonsChanged = JSON.stringify(left.review_reasons) !== JSON.stringify(next.review_reasons);
    if (changedEmissions.length || readinessChanged || reasonsChanged) {
      changes.push({ file: left.file, changed_emissions: changedEmissions,
        before_readiness: left.readiness, after_readiness: next.readiness,
        before_reasons: left.review_reasons, after_reasons: next.review_reasons });
    }
  }
  return { diagnostic_only: true, records: before.records.length, changes,
    emissions_unchanged: changes.every((row) => row.changed_emissions.length === 0),
    readiness_regressions: changes.filter((row) => row.before_readiness === 'ready' && row.after_readiness !== 'ready').map((row) => row.file) };
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [before, after, output] = process.argv.slice(2);
  if (!before || !after || !output) throw new Error('usage: compare-evaluation-replays.mjs before.json after.json output.json');
  const result = compareReplays(JSON.parse(await readFile(before, 'utf8')), JSON.parse(await readFile(after, 'utf8')));
  await writeFile(output, `${JSON.stringify(result, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(result)}\n`);
  if (!result.emissions_unchanged || result.readiness_regressions.length) process.exitCode = 1;
}
