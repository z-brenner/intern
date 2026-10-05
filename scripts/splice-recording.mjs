import { readFile, writeFile } from 'node:fs/promises';
import { isDeepStrictEqual } from 'node:util';
import { pathToFileURL } from 'node:url';

/**
 * Splices freshly recorded fixtures into the committed corpus recording.
 *
 * A change to how the worker reads scans changes what the OCR fixtures'
 * prompts are built from, so their recorded text and replies have to be made
 * again, live. The rest of the recording cannot simply be made again with
 * them: its text fixtures were edited by hand after they were recorded (the
 * counterparty renamed throughout, hashes recomputed), and a live run of the
 * edited fixtures does not reproduce those replies byte for byte. Re-recording
 * everything would change scores nobody's change touched. So the scans are
 * recorded live into a scratch file and only their entries are carried over.
 *
 * The recording is spliced as text. `intern-evaluate` writes it with serde's
 * pretty printer, which never puts a raw newline inside a string, so each
 * fixture entry is a run of whole lines: replacing those lines leaves every
 * other byte as it was. Parsing and re-serialising it here would not -
 * `1.0` would come back as `1` across every entry. What the splice produced
 * is then parsed and checked against both inputs before anything is written.
 *
 * Nothing is spliced if a fixture that was not asked for was read differently
 * by the live run: the change that prompted the re-record was meant to touch
 * only the fixtures named, and a difference anywhere else is a finding to
 * investigate, not a recording to commit. A reply that differs on its own is
 * only reported - the hand-edited fixtures' replies are expected to.
 */
export function spliceRecording(committedText, liveText, { files, note }) {
  if (!files.length) throw new Error('name at least one fixture to splice');
  if (!note) throw new Error('a spliced recording needs a note saying what was re-recorded');
  const committed = JSON.parse(committedText);
  const live = JSON.parse(liveText);
  for (const key of ['schema_version', 'model_id', 'budget_characters']) {
    if (committed[key] !== live[key]) {
      throw new Error(`the recordings differ in ${key}: ${committed[key]} and ${live[key]}`);
    }
  }
  const byFile = (recording) => new Map(recording.fixtures.map((entry) => [entry.file, entry]));
  const committedEntries = byFile(committed);
  const liveEntries = byFile(live);
  for (const file of files) {
    const before = committedEntries.get(file);
    const after = liveEntries.get(file);
    if (!before) throw new Error(`${file} is not in the committed recording`);
    if (!after) throw new Error(`${file} is not in the live recording`);
    if (before.sha256 !== after.sha256) {
      throw new Error(`${file} was recorded from different bytes; the fixture changed, so the whole corpus needs re-recording`);
    }
  }

  const differing = [];
  for (const [file, before] of committedEntries) {
    if (files.includes(file)) continue;
    const after = liveEntries.get(file);
    if (!after) continue;
    const extraction = !isDeepStrictEqual(before.extraction, after.extraction);
    const prompt = before.prompt_sha256 !== after.prompt_sha256;
    const reply = !isDeepStrictEqual(before.reply, after.reply);
    if (extraction || prompt || reply) differing.push({ file, extraction, prompt, reply });
  }
  const misread = differing.filter((entry) => entry.extraction);
  if (misread.length) {
    throw new Error(
      `the live run read fixtures that were not asked for differently: ${misread.map((entry) => entry.file).join(', ')}`,
    );
  }

  const liveBlocks = new Map(entryBlocks(liveText).map((block) => [block.file, block]));
  const lines = committedText.split('\n');
  const replaced = [];
  for (const block of entryBlocks(committedText)) {
    if (!files.includes(block.file)) continue;
    const source = liveBlocks.get(block.file);
    // Keep the committed block's trailing comma, which says where it sits.
    const body = source.lines.map((line, index) => (
      index === source.lines.length - 1 ? line.replace(/,$/, '') + (block.lines.at(-1).endsWith(',') ? ',' : '') : line
    ));
    replaced.push({ start: block.start, end: block.end, body });
  }
  for (const { start, end, body } of replaced.sort((left, right) => right.start - left.start)) {
    lines.splice(start, end - start + 1, ...body);
  }
  const noteLine = lines.findIndex((line) => line.startsWith('  "note": '));
  if (noteLine < 0) throw new Error('the committed recording has no note line');
  lines[noteLine] = `  "note": ${JSON.stringify(note)},`;
  const text = lines.join('\n');

  const spliced = JSON.parse(text);
  const expected = {
    ...committed,
    note,
    fixtures: committed.fixtures.map((entry) => (files.includes(entry.file) ? liveEntries.get(entry.file) : entry)),
  };
  if (!isDeepStrictEqual(spliced, expected)) {
    throw new Error('the spliced recording does not hold what was asked for; it was not written');
  }
  return { text, differing };
}

/**
 * Every fixture entry of a pretty-printed recording: the lines from its
 * opening `{` to its closing `}`, at the four-space indent the `fixtures`
 * array puts them at, and the fixture each one records.
 */
function entryBlocks(text) {
  const lines = text.split('\n');
  const opening = lines.indexOf('  "fixtures": [');
  if (opening < 0) throw new Error('not a pretty-printed recording: no fixtures array');
  const blocks = [];
  let start = null;
  for (let index = opening + 1; index < lines.length; index += 1) {
    const line = lines[index];
    if (line === '  ]' || line === '  ],') break;
    if (line === '    {') start = index;
    if ((line === '    }' || line === '    },') && start !== null) {
      const blockLines = lines.slice(start, index + 1);
      const { file } = JSON.parse(blockLines.join('\n').replace(/,$/, ''));
      blocks.push({ file, start, end: index, lines: blockLines });
      start = null;
    }
  }
  return blocks;
}

async function runCli() {
  const args = process.argv.slice(2);
  const noteAt = args.indexOf('--note');
  const note = noteAt >= 0 ? args[noteAt + 1] : '';
  const positional = args.filter((_, index) => noteAt < 0 || (index !== noteAt && index !== noteAt + 1));
  const [committedPath, livePath, ...files] = positional;
  if (!committedPath || !livePath || !files.length || !note) {
    throw new Error('usage: splice-recording.mjs <committed.json> <live.json> --note TEXT <fixture>...');
  }
  const { text, differing } = spliceRecording(
    await readFile(committedPath, 'utf8'),
    await readFile(livePath, 'utf8'),
    { files, note },
  );
  await writeFile(committedPath, text);
  process.stdout.write(`${JSON.stringify({ spliced: files, differing_elsewhere: differing })}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) await runCli();
