import { readFile, readdir } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';

/**
 * The real-document regression gate replays bench/realdocs/recordings/ against
 * bench/realdocs/baseline-*.json in CI. Those recordings are extracted text of
 * real documents, committed only where the source's licence allows it, so what
 * they hold has to agree with bench/realdocs/SOURCES.md, which says per
 * document why it was cleared or left out, and with the baselines, which say
 * what the gate holds: a document the recording lost would otherwise drop out
 * of the gate unnoticed, and one it gained would be committed unreviewed.
 */

const SETS = {
  real1: 'sample.json',
  real2: 'sample-heldout.json',
  real3: 'sample-heldout2.json',
  real4: 'sample-heldout3.json',
} as const;
type SetName = keyof typeof SETS;

/** The gate's recording of each set: the accurate model's, recorded on a runner. */
const recordingOf = (set: SetName) => `bench/realdocs/recordings/${set}-qwen35-9b.json`;

/** The licences the rule admits, as SOURCES.md names them. */
const ADMITTED =
  /^(US federal government work \(17 U\.S\.C\. 105\)|Public Domain Mark 1\.0|Public-domain dedication|CC0 1\.0|UK Open Government Licence v3\.0|CC BY 4\.0|CC BY 3\.0 US)$/;

/**
 * Words a sample's licence statement uses when it says someone other than the
 * licensor wrote, produced or repackaged the document: a licence, mark or
 * dedication covers only what its giver could license (SOURCES.md, "The
 * rule"), so a cleared document's statement never says so.
 */
const SOMEONE_ELSE = /written by|authored by|third.party|uploader|produced by|repackag|underlying|made public by/i;

/**
 * A Public Domain Mark says a work is free of copyright; the marks here were
 * applied by the records' holders, not their authors, so one is accepted only
 * for a record made before this year, which no US term can still cover: an
 * unpublished work made for hire or of unknown author is protected for at most
 * 120 years from its making (SOURCES.md, "The rule"). Each such row's title
 * gives the record's year.
 */
const MARKED_BEFORE = 1906;

interface Cleared {
  id: string;
  recorded: boolean;
  title: string;
  url: string;
  licence: string;
  attribution: string;
}

interface Sources {
  cleared: Cleared[];
  excluded: string[];
}

/** SOURCES.md, set by set: its table rows (cleared) and its bullets (excluded). */
async function sources(): Promise<Record<SetName, Sources>> {
  const text = await readFile('bench/realdocs/SOURCES.md', 'utf8');
  const result = {} as Record<SetName, Sources>;
  for (const section of text.split(/^## /m).slice(1)) {
    const set = /^(real\d): /.exec(section)?.[1] as SetName | undefined;
    if (!set) continue;
    const cleared: Cleared[] = [];
    const excluded: string[] = [];
    for (const line of section.split('\n')) {
      if (line.startsWith('| `')) {
        const cells = line.split(/(?<!\\)\|/).slice(1, -1).map((cell) => cell.trim());
        expect(cells, line).toHaveLength(6);
        const [id, recorded, title, source, licence, attribution] = cells;
        cleared.push({
          id: id.replaceAll('`', ''),
          recorded: recorded.startsWith('yes'),
          title,
          url: /<([^>]+)>$/.exec(source)?.[1] ?? '',
          licence,
          attribution,
        });
      } else if (line.startsWith('- `')) {
        excluded.push(/^- `([^`]+)`: \S/.exec(line)![1]);
      }
    }
    result[set] = { cleared, excluded };
  }
  return result;
}

const json = async (path: string) => JSON.parse(await readFile(path, 'utf8'));

describe('the committed real-document recordings', () => {
  it('account for every document of every sample, cleared or excluded with a reason', async () => {
    const listed = await sources();
    expect(Object.keys(listed).sort()).toEqual(Object.keys(SETS));
    for (const [set, file] of Object.entries(SETS) as [SetName, string][]) {
      const sample = await json(`scripts/realdocs-eval/${file}`);
      const ids = sample.documents.map((document: { id: string }) => document.id);
      const { cleared, excluded } = listed[set];
      expect([...cleared.map((row) => row.id), ...excluded].sort(), set).toEqual([...ids].sort());
      for (const row of cleared) {
        const document = sample.documents.find((candidate: { id: string }) => candidate.id === row.id);
        expect(row.url, row.id).toBe(document.source.url);
        expect(row.licence, row.id).toMatch(ADMITTED);
        expect(document.source.license, row.id).not.toMatch(SOMEONE_ELSE);
        if (row.licence === 'Public Domain Mark 1.0') {
          const years = [...row.title.matchAll(/\b(1[5-9]\d\d|20\d\d)\b/g)].map((match) => Number(match[1]));
          expect(years.length, row.id).toBeGreaterThan(0);
          expect(Math.max(...years), row.id).toBeLessThan(MARKED_BEFORE);
        }
        // A CC BY row names the version the source states.
        if (row.licence.startsWith('CC BY ')) {
          expect(document.source.license, row.id).toContain(row.licence.slice('CC BY '.length));
        }
        // The attribution OGL v3.0 and CC BY require.
        if (row.licence.includes('Open Government Licence')) {
          expect(row.attribution, row.id).toContain('Contains public sector information licensed under the Open Government Licence v3.0');
        }
        if (row.licence.includes('CC BY')) {
          expect(row.attribution, row.id).toMatch(/licensed under CC BY (4\.0|3\.0 US) \(https:\/\/creativecommons\.org\/licenses\/by\//);
        }
      }
    }
  });

  it('hold exactly the cleared documents SOURCES.md says they hold', async () => {
    const listed = await sources();
    const files = (await readdir('bench/realdocs/recordings')).sort();
    expect(files).toEqual((Object.keys(SETS) as SetName[]).map((set) => recordingOf(set).split('/').pop()));
    for (const set of Object.keys(SETS) as SetName[]) {
      const recording = await json(recordingOf(set));
      const recorded = recording.documents.map((document: { id: string }) => document.id).sort();
      const cleared = listed[set].cleared.filter((row) => row.recorded).map((row) => row.id).sort();
      expect(recorded, set).toEqual(cleared);
      expect(recording.pipeline, set).toBe('evidence');
    }
  });

  it('are each held to a baseline over exactly their documents, recalls included', async () => {
    for (const set of Object.keys(SETS) as SetName[]) {
      const recording = await json(recordingOf(set));
      const baseline = await json(`bench/realdocs/baseline-${set}.json`);
      expect(baseline.mode, set).toBe('replay');
      const recorded = recording.documents.map((document: { id: string }) => document.id).sort();
      // CI scores every document the recording holds (fetch.py --gold-only
      // --recording), so a baseline document the recording lost fails the
      // replay as missing; this says the same before anything is built.
      expect(Object.keys(baseline.documents).sort(), set).toEqual(recorded);
      for (const [id, document] of Object.entries(baseline.documents) as [string, { values?: Record<string, number> }][]) {
        expect(document.values?.prompt_recall, `${set} ${id}`).toBeTypeOf('number');
      }
    }
  });
});
