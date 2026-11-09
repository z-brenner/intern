import { readFile } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';

// The fifth real set's report explains each wrong name with classes written
// by hand in readings.json, and its README counts them in a table. A class
// written by hand can say a field was wrong that intern-bench scored right,
// and a table row can count names that readings.json does not class that way;
// no replay would notice either. So the classes that speak of one field are
// held to the scorer's own verdict on that field, which readings.json records
// beside them, and the README's table and headline are held to readings.json.

const REPORT = 'bench/reports/realdocs-heldout4-2026-11';

interface Reading {
  id: string;
  filename: string | null;
  scorer: boolean | null;
  adjudicated: boolean | null;
  readiness_at_0_5: string | null;
  type_correct: boolean | null;
  date_correct: boolean | null;
  parties_correct: boolean | null;
  relation_correct: boolean | null;
  classes: string[];
}

/** The row a class is counted in: its words before the first " (", as the README's table names it. */
function row(label: string): string {
  return label.split(' (')[0].trim().toLowerCase();
}

async function wrongNames(): Promise<Reading[]> {
  const readings: Reading[] = JSON.parse(await readFile(`${REPORT}/readings.json`, 'utf8')).readings;
  expect(readings.length).toBe(57);
  return readings.filter((reading) => reading.scorer === false);
}

/** The README's failure-class table: the count in its heading, and each row's label and two counts. */
async function classTable(): Promise<{ heading: number; rows: [string, number, number][] }> {
  const readme = await readFile(`${REPORT}/README.md`, 'utf8');
  const heading = /\*\*Why the (\d+) wrong names are wrong\*\*/.exec(readme);
  expect(heading, 'the README has its failure-class table').not.toBeNull();
  const after = readme.slice(heading!.index).split('\n');
  const start = after.findIndex((line) => line.startsWith('| Class |'));
  const rows: [string, number, number][] = [];
  for (const line of after.slice(start + 2)) {
    if (!line.startsWith('|')) break;
    const cells = line.split('|').slice(1, -1).map((cell) => cell.trim());
    expect(cells, line).toHaveLength(3);
    rows.push([cells[0], Number(cells[1]), Number(cells[2])]);
  }
  expect(rows.length).toBeGreaterThan(5);
  return { heading: Number(heading![1]), rows };
}

describe("the fifth real set's failure classes", () => {
  it('class every wrong name and no right one', async () => {
    const readings: Reading[] = JSON.parse(await readFile(`${REPORT}/readings.json`, 'utf8')).readings;
    expect(readings.filter((reading) => reading.scorer === false && reading.classes.length === 0)).toEqual([]);
    expect(readings.filter((reading) => reading.scorer === true && reading.classes.length > 0)).toEqual([]);
    expect(readings.flatMap((reading) => reading.classes).filter((label) => label === '?')).toEqual([]);
  });

  it('say a field is wrong only where the scorer marks it wrong', async () => {
    // Each class that speaks of one field, and what the scorer must have found for it.
    const claims: Record<string, (reading: Reading) => boolean> = {
      'type missing': (reading) =>
        reading.type_correct === false && /^(\d{4}-\d{2}-\d{2} )?Document\b/.test(reading.filename ?? ''),
      'no date read': (reading) => reading.date_correct === false,
      'no party named': (reading) => reading.parties_correct === false,
      'a person named instead of the organization': (reading) => reading.parties_correct === false,
      // A reversed relation names the right body the wrong way round.
      'relation reversed': (reading) => reading.parties_correct === true && reading.relation_correct === false,
    };
    const wrong = await wrongNames();
    for (const key of Object.keys(claims)) {
      expect(wrong.some((reading) => reading.classes.some((label) => row(label) === key)), key).toBe(true);
    }
    const contradicted = wrong.flatMap((reading) =>
      reading.classes
        .filter((label) => claims[row(label)] && !claims[row(label)](reading))
        .map((label) => `${reading.id}: ${label}`),
    );
    expect(contradicted).toEqual([]);
  });

  it("are counted in the README's table as readings.json holds them", async () => {
    const wrong = await wrongNames();
    const { heading, rows } = await classTable();
    expect(heading).toBe(wrong.length);
    const listed = rows.map(([label]) => row(label)).filter((key) => key !== 'other');
    expect(rows.map(([label]) => row(label))).toContain('other');
    const counted = rows.map(([label, names, ready]) => {
      const key = row(label);
      const members = wrong.filter((reading) =>
        reading.classes.some((className) =>
          key === 'other' ? !listed.includes(row(className)) : row(className) === key,
        ),
      );
      const membersReady = members.filter((reading) => reading.readiness_at_0_5 === 'ready').length;
      return { label, table: [names, ready], readings: [members.length, membersReady] };
    });
    expect(counted.filter(({ table, readings }) => table[0] !== readings[0] || table[1] !== readings[1])).toEqual([]);
  });

  it("give the README's headline count", async () => {
    const readme = await readFile(`${REPORT}/README.md`, 'utf8');
    const headline = /\*\*Main names (\d+) of (\d+) right/.exec(readme);
    expect(headline, 'the README states the headline count').not.toBeNull();
    const readings: Reading[] = JSON.parse(await readFile(`${REPORT}/readings.json`, 'utf8')).readings;
    const scored = readings.filter((reading) => reading.adjudicated !== null);
    expect(Number(headline![1])).toBe(scored.filter((reading) => reading.adjudicated).length);
    expect(Number(headline![2])).toBe(scored.length);
  });
});
