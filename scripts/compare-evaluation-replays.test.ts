import { expect, it } from 'vitest';
import { compareReplays } from './compare-evaluation-replays.mjs';

const record = { file: 'contract.pdf', status: 'completed', filename: '2026-04-01 Contract.pdf',
  description: 'Contract for Example LLC.', proposal: { document_date: '2026-04-01' },
  token_confidence: { min: 0.9, mean: 0.95, tokens: 3 },
  readiness: 'needs_review', review_reasons: ['description_unsupported'], stale: false };
const report = (row = record) => ({ mode: 'replay', records: [row] });

it('shows a review improvement with identical emitted facts and probabilities as diagnostics only', () => {
  const result = compareReplays(report(), report({ ...record, readiness: 'ready', review_reasons: [] }));
  expect(result.diagnostic_only).toBe(true);
  expect(result.emissions_unchanged).toBe(true);
  expect(result.readiness_regressions).toEqual([]);
  expect(result.changes.map((row: { file: string }) => row.file)).toEqual(['contract.pdf']);
});

it('identifies changed emitted dates and confidence instead of hiding them behind readiness', () => {
  const result = compareReplays(report(), report({ ...record, proposal: { document_date: '2026-05-01' },
    token_confidence: { min: 0.8, mean: 0.95, tokens: 3 } }));
  expect(result.emissions_unchanged).toBe(false);
  expect(result.changes[0].changed_emissions).toEqual(['proposal', 'token_confidence']);
});

it('exposes ready-to-review regressions', () => {
  const result = compareReplays(report({ ...record, readiness: 'ready', review_reasons: [] }), report());
  expect(result.readiness_regressions).toEqual(['contract.pdf']);
});

it('refuses stale, live, duplicate and mismatched fixture inputs', () => {
  expect(() => compareReplays(report(), report({ ...record, stale: true }))).toThrow(/stale/);
  expect(() => compareReplays({ ...report(), mode: 'live' }, report())).toThrow(/replay/);
  expect(() => compareReplays({ mode: 'replay', records: [record, record] }, report())).toThrow(/unique/);
  expect(() => compareReplays(report(), report({ ...record, file: 'other.pdf' }))).toThrow(/same fixtures/);
});
