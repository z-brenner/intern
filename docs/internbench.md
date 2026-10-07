# InternBench

InternBench is Intern's second evaluation suite. It sits beside the
clean-room corpus in `fixtures/` ([`evaluation.md`](evaluation.md)) and
replaces none of it. The corpus there was built to prove the redesign that
chose today's pipeline. InternBench exists so that every later change can be
shown to improve Intern rather than merely sound better. It measures the
cases the original corpus covers thinly or not at all: long documents, dense
layouts, competing dates, scans of varying quality, and the time every stage
takes.

Everything in it is synthetic and fictional: every person, company,
address, identifier and figure is invented.

## What it measures

A run answers four questions per document, and per group of documents:

1. **Is the name right?** The whole filename, and each fact in it: the
   type, the defining date, the parties and the role each plays, and the
   word that joins them. Each is scored against the reviewed answer, and
   separately against the traps a careless reading falls into.
2. **Is the sentence right?** Every fact the description asserts must be in
   the document (factuality). It should cover the facts that matter
   (completeness) and say something only this document could
   (specificity).
3. **Did Intern know when to ask?** Readiness against the reviewed
   routing. The number that matters most is **unsafe ready**: a document
   filed without review under a wrong name.
4. **Where did the time go?** Every stage, from snapshot to naming, with
   OCR split into page encoding and Tesseract, and inference split into
   prefill and generation by the server's own count. Latency is reported at
   p50 and p95 by page count, by document kind and by text layer, with
   sampled peak memory.

For scanned documents it also scores OCR itself, against the exact text
drawn on each page: character and word error rates, and whether the dates,
names and identifiers on the page survived.

## The corpus

`bench/generate.mjs` builds 52 documents into `bench/generated/`
(gitignored, about 7 MB, about 9 s). The reviewed answers are in
`bench/gold.json`, and a SHA-256 of every file is in `bench/manifest.json`.
Both are committed. The generator is deterministic: fixed seeds, fixed
timestamps, a fixed zlib level, and no clock or locale. Run on the pinned
Node it reproduces both files byte for byte, which
`bench/generate.test.ts` checks in CI.

`--only id,id` rebuilds just those documents and leaves the others' files
in place; the output's `manifest.json` then says `"partial": true` unless it
still lists every document. `--out DIR` writes somewhere else. A full build
replaces the files the directory's earlier `manifest.json` lists and removes
nothing else, and the generator refuses a directory that is not empty and
holds no InternBench manifest. `scripts/run-internbench.sh` generates the
default corpus again whenever `bench/generated/manifest.json` is not the
committed `bench/manifest.json`.

| Group | Documents |
| --- | --- |
| Digital PDFs | one-page notice; invoices with the date in a header table and with the issuer only in the layout; account statement with 25+ transaction dates; purchase order; master services agreement; second amendment; SOW issued under an MSA, effective date on page 3 of 5; notice of default; offer letter; prior authorization; explanation of benefits; promissory note; capital call; engagement letter; letterhead-only letter; two-column lease; row-interleaved two-column declarations page; 8-page annual-report excerpt; vendor registration form; change order; board minutes with ~20 names; museum condition report; aircraft maintenance record; assignment of lease |
| Long, information-dense PDFs | 10-page data processing addendum (20+ sub-processors); 25-page asset purchase agreement; 50-page credit agreement dated only by the definition of "Closing Date" on page 10; 100-page annual report dated on page 41, under the auditors' report, and again only in a later note |
| Office, sheets, mail, text | separation agreement, demand letter, written consent (`.docx`); quarterly business review, launch plan (`.pptx`); payroll register with a date serial cell, harvest log (`.xlsx`); AP aging (`.csv`); approval email quoting an older message (`.eml`); hearing notice (`.txt`) |
| Scans | clean 2- and 10-page image-only PDFs; a 25-page 200-DPI lease; a mixed PDF whose signature page is scanned; rotated 90° and 180°; 3° skew; 100 DPI; speckle noise and blur; faint uneven light; an invisible OCR text layer full of OCR errors; a two-frame TIFF fax; a scanned intake form |

Every document carries category tags (`multi_column`, `date_in_table`,
`referenced_agreement`, `middle_fact`, `pages_50`, ...), and every report is
also sliced by tag. The tests prove that the corpus covers the required
categories, that every gold string occurs in the text the document carries,
and that the long documents are information-dense rather than padded:
almost every page is distinct after digits are masked, and five-word
phrases rarely repeat.

Scans are drawn with glyphs rasterised from an open-licence typeface
(`bench/assets/raster-fonts.json`; see `bench/assets/OFL.txt`). OCR is
therefore measured on text that looks like print, not on the 5×7 bitmap
font `fixtures/` uses.

### The gold

Each entry in `bench/gold.json` gives the document's `kind`, `format`,
`text_layer`, `pages` and `categories`, and a `gold` block:

* `document_type`, `acceptable_types`
* `document_date`, `acceptable_dates`, `date_role`, `forbidden_dates` (each
  with the reason it is a trap)
* `parties`, `party_relation`, `acceptable_party_sets` (other defensible
  answers, each with its own relation), `party_roles`, `forbidden_parties`
  (each with its reason)
* `description_facts`: the facts a good description states, each a list of
  spellings of that one fact, every one specific enough that a description
  containing it states the fact (a loan number, not "Loan No")
* `description_forbidden`: what a careless reading would assert that the
  document does not say. A value the document prints is never one.
* `subject_terms`
* `expected_readiness`: `ready`, `needs_review` or `either`
* `evidence`: the verbatim forms in which the document states its date and
  names its parties

A scanned document also has `ocr_truth`: the exact text of each scanned page,
and the dates, names and identifiers on it.

To change an answer, edit the builder in `bench/docs/`, review the
regenerated document, and run
`npm exec --package=node@24.15.0 -- node bench/generate.mjs --update-gold`.
Do not use `--update-gold` in CI.

## Running it

### Live

Live needs the parser worker built with the native features, a runtime
directory holding PDFium, Tesseract and tessdata, `llama-server`, and the
pinned model.

```sh
cargo build --release --locked -p intern-worker --features windows-native
cargo build --release --locked -p intern-bench
MODEL=/path/to/Qwen3.5-2B-Q4_K_M.gguf LLAMA_SERVER=/path/to/llama-server \
INTERN_RUNTIME_DIR=/path/to/runtime THREADS=4 \
  scripts/run-internbench.sh target/internbench -- --baseline bench/baseline.json
```

The script starts `llama-server` with the app's flags: one slot, CPU only,
8,192-token context, the model's own chat template, no projector. It waits
until the server is healthy and runs `intern-bench run` with a recording.
It stops the server however the run ends, and never prints the API key it
generated. Before the corpus, one short synthetic text document goes through
the worker and the engine and is discarded, so the first scored document is
not charged for starting the worker or for an empty prompt cache. The first
PDF a worker process reads still loads PDFium, which is counted as parse;
the worker is started again after a timeout, a cancellation or a crash.
`--only id,id` runs a subset.

Each document goes the way it goes in the app: `SupervisedWorker` extracts
it, then `Engine::analyze` distils it, fits the prompt to the context, asks
the model, validates the reply and composes the name. A recording proposer
sits in front of the model client and keeps every prompt's SHA-256, the
reply, and the server's timings.

### Replay

```sh
cargo run --release --locked -p intern-bench -- run --corpus bench/generated \
  --gold bench/gold.json --manifest bench/manifest.json \
  --replay bench/recording.json --baseline bench/baseline.json \
  --output report.json --markdown report.md
```

Replay re-runs everything after the model (distillation, validation, role
and type inference, naming, scoring) from what the worker read and what the
model replied. It needs no worker, model or generated corpus: fixture
staleness is checked against the manifest. Prompts are built as the app
builds them today, with today's digest budget and context size; when either
differs from the recording's, the run says so on standard error, in the
report and on its page. It follows the same rules as the existing corpus:

* a document whose prompt the engine no longer builds is `stale_prompt`;
* a document whose bytes changed is `stale_fixture`;
* a document the recording lacks is `unrecorded`.

Each one fails the run (exit 2). `--allow-stale` lets only `stale_prompt`
through: the document is scored from the reply the recorded run ended on,
marked `"stale": true`, and the report says those scores do not measure this
code. A `stale_fixture` or `unrecorded` document always fails the run and
has to be recorded again (see [Working with it](#working-with-it)). Replay
reports the recording's timings and says so (`timings_source: recorded`).
It cannot measure a change to extraction or to the prompt; those need a
live run.

### Comparing two runs

```sh
intern-bench compare --before before.json --after after.json --markdown diff.md
```

This aligns the two reports by document. Every rate, mean and count is
computed again over the documents both runs scored, and each score over the
documents that have it in both, so a subset run, a document added since or
one that went stale moves no figure; the comparison says when the two runs
cover different documents or gold. It lists every document that flipped on
every score (fixed or broken). Latency, as the change in p50 and p95 of
every stage, is compared over the documents both runs completed, and only
between two live runs: a replay reports its recording's timings, so a
comparison involving one shows no latency change and says why. Each side
names its machine and, for a replay, its recording. `intern-bench report
--input report.json --markdown report.md` re-renders a report's Markdown.

## Reading a report

`report.json` has these sections:

* `summary`: every boolean score as `{correct, total, rate}` and every
  fractional score as a mean, plus counts of unsafe-ready, trap-date,
  forbidden-party and unsupported-claim outcomes.
* `groups`: the same summary sliced by `kind`, `text_layer`, `format`, page
  bucket and category.
* `latency`: p50, p90, p95, max and mean of every timing, overall and by
  page bucket, kind and text layer, over the documents that completed (a
  failed document's time is only how long it took to fail).
* `ocr`, `memory`, and one record per document holding the name, the
  description, the review reasons, the scores, the claims checked, the traps
  sprung and the timings.

Keys are sorted and floats rounded, so two runs diff cleanly.

`report.md` is the same report for a person. It opens with a scorecard and a
safety table, then the per-group tables, OCR, the stage-by-stage latency
table with tokens per second, memory, and **Misses**. Misses lists every
wrong filename with the expected name, the trap it sprang and why, and
whether it was filed without review.

### Scores

| Score | Meaning |
| --- | --- |
| `filename_correct` | The name equals one composed by the engine's own naming from an acceptable type × date × party set, compared the way Windows compares names. A two-party `between` name is right in either order. Scored where the gold has a date and a type. |
| `type_correct`, `date_correct`, `date_exact` | The facts in the name. |
| `date_role_correct` | The date's role, judged only when the reviewed date was chosen: the gold's role is that date's, and another acceptable date can rightly have another. |
| `date_forbidden` | A trap date was chosen. It is good when false. |
| `parties_correct` | Exactly one acceptable set of parties, nobody else. `parties_spurious` counts extras. |
| `party_forbidden` | A party the gold marks as a trap was named. |
| `relation_correct`, `party_role_correct` | The joining word, and whether the party it points at holds that role: `from` an issuer or sender, `for` a subject, `to` a recipient, `with` a counterparty. For `between`, every role-holder must be named. Judged when parties were named. |
| `description_completeness` | Fraction of the gold facts the sentence states. |
| `description_factual` | Every checkable claim (dates, amounts, percentages, identifiers, capitalised names) occurs in the document, and no forbidden fact is asserted. The extraction is a heuristic that leans towards "supported", so a failure is worth reading. |
| `description_specificity` | The mean of three marks: names a party or gold fact; carries a concrete detail; 10–42 words. |
| `evidence_recall` | The model's quoted evidence contains the date and each party. |
| `digest_recall`, `prompt_recall` | The gold evidence is in the distilled digest, or in the prompt actually sent. This is deterministic, so a distillation change can be measured in replay without a model. |
| `readiness_match`, `unsafe_ready`, `needless_review` | Routing against the gold; ready with a wrong name; review although the name was right and the gold says ready. |
| `ocr_cer`, `ocr_wer`, `ocr_date_accuracy`, `ocr_name_accuracy`, `ocr_identifier_accuracy` | OCR against the drawn text. Levenshtein is computed per page, and the totals are pooled. |

A score the gold does not define is omitted rather than counted as false,
so a rate's denominator is the documents that could be right or wrong about
it: every document for most scores, and for the date role, the relation word
and the party's role, the documents whose answer gave them something to
judge. A document that failed is a miss on every score its reviewed answer
would be judged on, those three included.

### Timings

Worker figures come from the worker itself (`ExtractionTimings`): snapshot,
parse, page analysis, render, image decode, OCR (split into page encoding
and Tesseract, with page and pass counts), and the unused page image. Engine
figures come from `AnalysisTelemetry`: distillation, prompt construction,
validation and naming, plus how many times the prompt was condensed to fit.
Prefill and generation times, and the evaluated, cached and generated token
counts, are llama-server's own (`timings` in its reply). Wall times around
extraction and analysis are measured by the runner.

Memory is the peak resident set of the worker, the model server and the
runner itself, sampled every 25 ms from `/proc`. It is Linux only, and a
sampled peak can miss a spike shorter than the interval.

Latency depends on the machine and on what else it is doing. Quote ranges
across runs, compare runs made on the same machine at the same thread
count, and read p95 as "the slow documents" rather than as a guarantee.

## Gates

`--write-baseline bench/baseline.json` records a run as the baseline.
`--baseline bench/baseline.json` compares a run with it:

* **Replay is gated per document.** A score that was good and is now bad is
  a regression, as in `intern-evaluate`. Trap, forbidden and unsafe scores
  are good when false. Replay is deterministic, so any flip is real.
* **Live is gated in aggregate** over the documents both runs share. A rate
  may fall by at most one document's worth. The trap-date, forbidden-party
  and unsafe-ready counts may not rise. A shared document that failed counts
  in them as the miss it is, and a document that completed in the baseline
  and no longer does fails the run on its own. Live inference is not
  bit-identical across machines, which is why single-document score flips
  are reported but not fatal.
* **Latency is gated only live and only when asked**, with
  `--latency-gate RATIO`. Then p95 of the total and of every stage, over the
  documents that completed in both the run and the baseline, must stay
  within RATIO times the baseline's; a subset run is held to the same subset.
  Use it only between runs on the same machine.

A regression exits 2. There are deliberately no absolute thresholds:
the baseline is what Intern does today, and the gates stop it getting
worse while the work makes it better.

## Working with it

Reports of record, the baseline first, are in [`bench/reports/`](../bench/reports/).

The recording and baseline of record are `bench/recording.json` and
`bench/baseline.json`. A live run writes them (`--record`,
`--write-baseline`), and they are committed with the change that produced
them; replay and the gates read them.

For a change to anything after the model (validation, inference, naming,
house style), replay against `bench/recording.json`, and check the
compare against the baseline before and after the change.

For a change to extraction, distillation or the prompt, run live before
and after on the same machine at the same thread count, then `compare` the
two reports. If the change is accepted, commit the new recording and
baseline with it, so the review diff shows exactly which documents moved.

When a builder change alters a few documents' bytes, replay reports them
`stale_fixture`. Record only those live (`-- --only id,id` to the script,
which writes `target/internbench/recording.json`), then merge them into the
recording of record:

```sh
intern-bench merge-recordings --base bench/recording.json \
  --add target/internbench/recording.json --output bench/recording.json
```

Every other document keeps its recorded reply. The merge refuses a
recording made with another model file, digest budget or context.

[`pipeline-bottlenecks.md`](pipeline-bottlenecks.md) quotes its measured
figures from a live run's report and says which.
