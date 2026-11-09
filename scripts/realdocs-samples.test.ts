import { readdir, readFile } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';

// intern-bench scores a real-document sample's gold by comparing the engine's
// date role and party relation with the gold's as plain strings, and reads
// expected_readiness only when it is one of the values it knows. A gold value
// outside those vocabularies is never matched and never refused: the field
// just scores wrong (or not at all) on every run, and no other test notices.
// Words such as "meeting", "signature" or "period_end" read naturally as a
// date's role but are none of the engine's. So every sample is held to the
// vocabularies the engine declares in intern-engine's domain.rs, which the
// model's grammar is built from; when an enum there gains or loses a member,
// this follows it.

const SAMPLES = 'scripts/realdocs-eval';

/** The strings an engine enum's `as_str` maps its members to. */
async function vocabulary(name: string): Promise<string[]> {
  const source = await readFile('crates/intern-engine/src/domain.rs', 'utf8');
  const start = source.indexOf(`impl ${name} {`);
  expect(start, `impl ${name} in domain.rs`).toBeGreaterThanOrEqual(0);
  const body = source.slice(start, source.indexOf('\n}\n', start));
  const asStr = body.slice(body.indexOf('pub const fn as_str'));
  const values = [...asStr.matchAll(/=> "([a-z_]+)"/g)].map((match) => match[1]);
  expect(values.length, `${name}::as_str maps its members`).toBeGreaterThan(1);
  return values;
}

interface PartySet {
  parties: string[];
  relation?: string | null;
}

interface Document {
  id: string;
  gold: {
    date_role?: string | null;
    party_relation?: string | null;
    acceptable_party_sets?: PartySet[];
    expected_readiness?: string | null;
  };
}

async function samples(): Promise<[string, Document[]][]> {
  const names = (await readdir(SAMPLES)).filter((name) => /^sample.*\.json$/.test(name)).sort();
  expect(names).toContain('sample-heldout4.json');
  return Promise.all(
    names.map(async (name): Promise<[string, Document[]]> => {
      const sample = JSON.parse(await readFile(`${SAMPLES}/${name}`, 'utf8'));
      return [name, sample.documents];
    }),
  );
}

describe('the real-document samples speak the engine vocabulary', () => {
  it('every gold date role is one the engine can propose', async () => {
    const roles = await vocabulary('DateRole');
    expect(roles).toEqual(expect.arrayContaining(['issuance', 'invoice', 'other']));
    const outside = (await samples()).flatMap(([name, documents]) =>
      documents
        .filter(({ gold }) => gold.date_role != null && !roles.includes(gold.date_role))
        .map(({ id, gold }) => `${name} ${id}: ${gold.date_role}`),
    );
    expect(outside).toEqual([]);
  });

  it('every party relation, reviewed or acceptable, is one the engine can propose', async () => {
    const relations = await vocabulary('PartyRelation');
    expect(relations).toEqual(expect.arrayContaining(['from', 'to', 'none']));
    const outside = (await samples()).flatMap(([name, documents]) =>
      documents.flatMap(({ id, gold }) =>
        [gold.party_relation, ...(gold.acceptable_party_sets ?? []).map((set) => set.relation)]
          .filter((relation) => relation != null && !relations.includes(relation))
          .map((relation) => `${name} ${id}: ${relation}`),
      ),
    );
    expect(outside).toEqual([]);
  });

  it('every expected readiness is one intern-bench reads', async () => {
    const outside = (await samples()).flatMap(([name, documents]) =>
      documents
        .filter(({ gold }) => !['ready', 'needs_review', 'either'].includes(gold.expected_readiness ?? ''))
        .map(({ id, gold }) => `${name} ${id}: ${gold.expected_readiness}`),
    );
    expect(outside).toEqual([]);
  });

  it('no sample lists a document twice, and no two samples share an id', async () => {
    const ids = (await samples()).flatMap(([, documents]) => documents.map(({ id }) => id));
    expect(ids.length).toBeGreaterThan(250);
    expect(ids.filter((id, index) => ids.indexOf(id) !== index)).toEqual([]);
  });
});
