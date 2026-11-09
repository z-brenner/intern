# Real-document regression gate

The four real-document sets (`scripts/realdocs-eval/sample*.json`: letters, notices, invoices
and office files from public sources) are replayed in CI on every pull request and every push to
main, from what the accurate model read and replied on a runner, and held to a baseline document
by document, as InternBench is. A change to validation, evidence, inference or naming that turns a
right name wrong or files a wrong name as ready fails the `Corpus replay` job in seconds, with no
model and no document fetched. A change to the prompt, indexing or retrieval fails it too, as
`stale_prompt`, until the sets are recorded again
([When the prompt changes](#when-the-prompt-changes)).

| File | What it is |
| --- | --- |
| `recordings/real{1..4}-qwen35-9b.json` | Qwen3.5 9B (Q4_K_M, the manifest's accurate model) recordings of the sets, made by the candidate benchmark workflow on GitHub-hosted runners (run 41718479983 at `c427aff` for real1-3, run 41749074250 at `0295a09` for real4), with only the documents whose licence allows committing their text |
| `baseline-real{1..4}.json` | What the code scores on them: each document's status, boolean scores and evidence recalls (`prompt_recall`, `context_*`), at the app's review gate, 0.7 |
| `SOURCES.md` | Every document of the four samples: why it was committed or left out, its source and licence, and the attribution OGL v3.0 and CC BY require |

## What is committed, and what it measures

A recording holds the text the parser worker extracted from each document, the hash of each
prompt and the model's reply: extracted text of real documents. It is committed only for
documents whose source licence plainly permits redistribution (US federal works, CC0, a Public
Domain Mark or dedication, the UK OGL v3.0, CC BY), and nothing unclear; `SOURCES.md` gives the
rule and each document's reason. The documents themselves are never committed.

| Set | Sample | Cleared | Committed | Named right | Named wrong | No filename gold | Wrong names filed ready | Gold date or party not in the prompt |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| real1 | `sample.json` (60) | 26 | 23 | 9 | 12 | 2 | 0 | 6 (5 named wrong) |
| real2 | `sample-heldout.json` (60) | 36 | 36 | 15 | 21 | 0 | 0 | 4 (4) |
| real3 | `sample-heldout2.json` (59) | 25 | 24 | 8 | 16 | 0 | 0 | 5 (5) |
| real4 | `sample-heldout3.json` (58) | 29 | 29 | 9 | 17 | 3 | 0 | 7 (6) |
| total | 237 | 116 | 112 | 41 | 66 | 5 | 0 | 22 (20) |

Four cleared documents are not committed because the runner never fetched them (ftc.gov answered
403 to two, archive.org 500 to one, tile.loc.gov 429 to one). The committed documents are a
harder half than the whole sets: the same recordings name 92 of 219 scored documents right
across all four sets, 41 of 107 here. Replaying a set takes under 5 s in a release build: 0.7 to
3.0 s each on GitHub runners (CI runs 42578520445 and 42594755612: 1.4, 1.4, 0.7 and 1.7 s, then
2.2, 2.5, 1.3 and 3.0 s), 2.0 to 4.4 s on a loaded 4-core box.

## How CI runs it

For each set, in the `corpus-replay` job of `.github/workflows/ci.yml`:

```bash
python3 -I scripts/realdocs-eval/fetch.py --gold-only \
  --recording bench/realdocs/recordings/real1-qwen35-9b.json "$RUNNER_TEMP/realdocs/real1" sample.json
cargo run --release --locked -p intern-bench -- \
  run --corpus "$RUNNER_TEMP/realdocs/real1/corpus" --gold "$RUNNER_TEMP/realdocs/real1/gold.json" \
  --replay bench/realdocs/recordings/real1-qwen35-9b.json \
  --baseline bench/realdocs/baseline-real1.json --min-token-confidence 0.7 \
  --output realdocs-real1-report.json --markdown realdocs-real1-report.md
```

`--gold-only` writes the sample's gold and fetches nothing; `--recording` keeps the gold to the
documents the recording holds, so the replay scores every one of them and a baseline document the
recording lost fails as missing. The corpus directory does not exist: a replay needs the recorded
text, not the documents, and says that nothing checks the documents' bytes (the recording holds
each one's SHA-256). Exit 2 is a regression; the reports go up with the job's artifact.
`scripts/realdocs-gate.test.ts` checks, on every `npm test`, that each recording holds exactly the
documents `SOURCES.md` marks as recorded and that each baseline covers exactly the recording.

## What it catches that InternBench does not

Measured on 2026-11-09 by running the job's steps exactly as `ci.yml` writes them, on a copy of the
tree with one change each (never committed), in a cold release build on a 4-core box (2 min 6 s
to build, then 1.5 s for the fixtures, 0.4 s for the digest fixtures, 11 s for InternBench and
3.4, 3.3, 1.9 and 3.9 s for the real sets). real1 then held 30 documents; the seven Enron records
left out since (`SOURCES.md`) are none of the documents named below, and a replay scores each
document on its own, so the outcomes stand:

- Unchanged: every step passes.
- The rule "a letter is to its client" in `compose.rs` reverted (`To` back to `For`): the fixtures,
  the digest fixtures and InternBench pass; real1, real3 and real4 fail on three names lost
  (`real-letter-05`, `real3-office-03`, `real4-letter-02`) and two more relations
  (`real-letter-07`, `real3-letter-10`).
- Retrieval's date and party budgets cut to 60 tokens: every replay but the digest fixtures fails,
  each changed prompt `stale_prompt`, as a prompt change should (2 fixtures, 19 InternBench
  documents and 10, 12, 8 and 10 real ones; 9 in real1 now). That is all CI shows of it: a strict
  replay does not compare a stale document's recalls. With `--allow-stale`, run locally, the
  recall gate fails them instead: `prompt_recall` falls on 6 real documents and 3 InternBench
  ones, and InternBench's `data-processing-agreement-10p` fails on `context_subject_recall`
  alone.

## Changing the baselines

A replay fails on any document that gets worse; one that gets better is reported as an
improvement and passes. To accept a new state, an improvement or a deliberate trade, write the
baseline again with the CI commands above plus `--write-baseline bench/realdocs/baseline-real1.json`
(and the same for the other sets), and say in the commit what moved and why. The baseline diff is
the review: every flipped score and every recall is a line.

## When the prompt changes

A change to `prompt.rs`, `index.rs`, `retrieve.rs` or the grammar makes the recorded replies
answer prompts the engine no longer builds: the documents replay as `stale_prompt` and the step
fails, as InternBench's does. CI never passes `--allow-stale`. A change to what the worker extracts
is not seen at all, since a replay reads the recorded text; only a new recording measures it. Such
a change carries re-recorded real sets and new baselines in the same pull request:

1. Push the branch and dispatch `candidate-bench.yml` on it with the manifest's accurate model
   (its default) and `sets` `real1,real2,real3,real4`.
2. Download each job's `recording.json` and keep only the cleared documents:
   `python3 -I scripts/realdocs-eval/commit-recording.py real1 recording.json bench/realdocs/recordings/real1-qwen35-9b.json`
   (it also says which cleared documents the new recording lacks, or now holds).
3. Write the four baselines as above, and commit recordings and baselines together.

Until then, `--allow-stale` on a local replay scores the stale documents from their recorded
replies, and the evidence recalls are still measured on the prompts the engine builds now, so a
retrieval change can be checked for lost dates and parties before the live run. Run it against
the committed baselines before writing new ones: once the new baselines are written, a recall the
change lost is no longer a failure, only a line of their diff.
