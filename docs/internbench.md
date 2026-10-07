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
names and identifiers on the page survived. Where the page's layout says
something plainly - the order its columns are read in, its tables, its
labelled values, the route each page should take - it scores whether the
text Intern reads keeps it ([Structure scores](#structure-scores)).

Extraction can also be measured on its own, without a model
([Extract-only](#extract-only)): fast enough to run on every change to the
parser worker.

## The corpus

`bench/generate.mjs` builds 77 documents into `bench/generated/`
(gitignored, about 9 MB, about 14 s). The reviewed answers are in
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
default corpus again whenever a file in it differs from the committed
`bench/manifest.json`, and checks the documents against that manifest; for
a corpus of your own (`CORPUS=…`) it checks against `MANIFEST` if you set
one, and nothing otherwise. A live run given `--manifest` refuses to start
when any selected document's bytes disagree with it or it does not list one.

| Group | Documents |
| --- | --- |
| Digital PDFs | one-page notice; invoices with the date in a header table and with the issuer only in the layout; account statement with 25+ transaction dates; purchase order; master services agreement; second amendment; SOW issued under an MSA, effective date on page 3 of 5; notice of default; offer letter; prior authorization; explanation of benefits; promissory note; capital call; engagement letter; letterhead-only letter; two-column lease; row-interleaved two-column declarations page; 8-page annual-report excerpt; vendor registration form; change order; board minutes with ~20 names; museum condition report; aircraft maintenance record; assignment of lease |
| Long, information-dense PDFs | 10-page data processing addendum (20+ sub-processors); 25-page asset purchase agreement; 50-page credit agreement dated only by the definition of "Closing Date" on page 10; 100-page annual report dated on page 41, under the auditors' report, and again only in a later note |
| Office, sheets, mail, text | separation agreement, demand letter, written consent (`.docx`); quarterly business review, launch plan (`.pptx`); payroll register with a date serial cell, harvest log (`.xlsx`); AP aging (`.csv`); approval email quoting an older message (`.eml`); hearing notice (`.txt`) |
| Scans | clean 2- and 10-page image-only PDFs; a 25-page 200-DPI lease; a mixed PDF whose signature page is scanned; rotated 90° and 180°; 3° skew; 100 DPI; speckle noise and blur; faint uneven light; an invisible OCR text layer full of OCR errors; a two-frame TIFF fax; a scanned intake form |
| Long documents for retrieval (`"recording": "pending"`) | evidence that decides the name, deep inside: a 12-page master services agreement whose Effective Date is stated only in its definitions schedule on page 9; a 25-page industrial lease dated only in its Lease Particulars on page 23, after a letter of intent and an old lease dated on page 1; a 40-page term loan whose lender and borrower are named only on page 1 and on the signature page; a 60-page commercial property policy whose declarations - and policy period - sit on page 30 of its renewal packet; a 100-page watershed monitoring report whose cover dates the review draft and whose final issue date is in the certification on page 88 |
| Layout and OCR (`"recording": "pending"`) | a three-column newsletter; a two-column agreement with a full-width title and footnotes; one meeting notice written into the content stream column by column, row by row across both columns, and back to front (the same page to the eye); a freight rate confirmation whose landscape page is stored portrait with `/Rotate 90`; a ruled inspection log split across two pages; an unruled price list aligned only by position; invoices whose header facts sit under small captions, across the page from their labels (the date a table cell away), and in a grid of boxes; a check-box benefits form; a boxed-field loss notice; a freight claim whose landscape page was scanned sideways inside the PDF; a cancellation notice scanned at 150 DPI; a remittance advice at 120 DPI, blurred and grainy; a lease renewal with a scanned exhibit between two digital pages; a license amendment whose signature block - and with it the only signature dates - is a pasted scan; a certificate of liability insurance (300 DPI) and a bill of lading (200 DPI) dense with dates, organisations and identifiers |

Every document carries category tags (`multi_column`, `date_in_table`,
`referenced_agreement`, `middle_fact`, `pages_50`, `key_value`,
`stream_order`, `rotated_page`, `image_region`, `ocr_critical_fields`, ...),
and every report is also sliced by tag. A `pages_N` tag is the page-count
bucket the document falls in, from N pages up to the next bucket (5, 10,
25, 50, 100): the 40-page loan is `pages_25`, the 60-page policy
`pages_50`. The tests prove that the corpus
covers the required categories, that every gold string occurs in the text
the document carries,
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
  names its parties; optionally `date_anchor`, the words that define the
  date where the document defines it by a term or a label rather than
  stating it plainly (`"Effective Date" means`, `Date of this Lease`,
  `Policy Period`), printed on the same page as the date; `type_text`, the
  title as printed; and `identifier_text`, the document's own number. The
  optional ones appear only where given and are not scored yet: the
  evidence-retrieval measures read them.

`party_roles` use the scorer's roles (`issuer`, `sender`, `subject`,
`recipient`, `counterparty`, which decide `party_role_correct`) and roles
that say what a party is. The second set includes the phase 3 role list -
`client`, `contractor`, `employer`, `employee`, `buyer`, `seller`,
`landlord`, `tenant`, `issuer`, `recipient`, `vendor`, `customer`,
`borrower`, `lender`, `licensor`, `licensee`, `sender`, `addressee`,
`other` - alongside the older `patient`, `payer`, `provider`, `firm`,
`fund`, `investor`, `assignor` and `assignee`; a party may hold several.

A scanned document also has `ocr_truth`: the exact text of each scanned page,
and the dates, names and identifiers on it. A digital page with a pasted
scan in it has the whole page's text, top to bottom, the scan's included.

A document whose layout plainly says something also has a `structure`
block, every part optional:

* `reading_order`: distinctive phrases in the order a person reads them,
  each printed exactly once. Most are the last words of one line and the
  first of the next (`readingSnippets` in `bench/docs/common.mjs`), so a
  reading that puts anything between those two lines loses the phrase.
* `tables`: each table's rows, header first, each a list of cells as
  printed (`""` for a blank cell). A check-box group is a table of two
  columns, the mark (`X` or blank) and the option; a blank mark is scored
  as an empty box, so an `X` read beside that option is wrong.
* `key_values`: `{key, value}` pairs, the label as printed without its
  colon. The generator refuses a pair whose value is printed under two
  occurrences of its label (after it on its line, before the next label,
  or on the line below a label standing alone): the scorer could credit
  either, so the pair would not say which one is meant.
* `expected_routes`: `{"<page>": "fast" | "layout" | "ocr" | "ocr_regions"}`,
  only for the pages where the route is not a judgement call.

Thirty-three documents have one: the twenty added for it, the five long
documents added for phase 3, and the two-column lease, the interleaved
declarations page, both header-table invoices, the purchase order, the
vendor registration form, the change order and the scanned intake form.
Adding or changing a `structure` block changes no recorded document's other
scores.

A document added before anyone recorded it says `"recording": "pending"`.
Replay leaves it unscored (and a baseline may not hold it); extract-only
reads it like any other.

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
* a document whose bytes changed is `stale_fixture`, and so is one a
  `--manifest` does not list when there is no file to hash either (with no
  manifest and no corpus, nothing is checked, and the run warns). So is a
  recorded document that does not say which bytes it was made from, or that
  was recorded under another file name (the worker picks its reader by
  extension);
* a document the recording lacks is `unrecorded`.

Each one fails the run (exit 2). `--allow-stale` lets only `stale_prompt`
through: the document is scored from the reply the recorded run ended on,
marked `"stale": true`, and the report says those scores do not measure this
code. A `stale_fixture` or `unrecorded` document always fails the run and
has to be recorded again (see [Working with it](#working-with-it)). Replay
reports the recording's timings and says so (`timings_source: recorded`).
It cannot measure a change to extraction or to the prompt; those need a
live run.

### Extract-only

```sh
cargo build --release --locked -p intern-worker --features windows-native
INTERN_RUNTIME_DIR=/path/to/runtime \
  cargo run --release --locked -p intern-bench -- run --extract-only \
  --worker target/release/intern-worker --corpus bench/generated \
  --gold bench/gold.json --manifest bench/manifest.json \
  --output extract.json --markdown extract.md
```

The parser worker alone, no endpoint, no model, no recording. Each document
goes through `SupervisedWorker` exactly as a live run sends it - the same
timeouts, the worker started on a throwaway text document first
(`--no-warmup` skips that), its stages timed by the worker itself
(`extract_timed`) - and is scored on what extraction alone decides: OCR
against the drawn text, the [structure scores](#structure-scores), and
`digest_recall`, the share of the gold's date and party evidence in the
digest the engine would build from the text. A document is `completed` when
the worker read it and `extraction_failed` when it did not; a failure is a
miss on every score its gold defines (0 for each accuracy, an error rate of
1, every structure item missed), except `route_correct`: nothing was read,
so nothing was routed, and it is left unscored as for a worker that sends no
layouts. Documents whose recording is pending are read like any other:
extraction needs no reply.

The run refuses a corpus that lacks a selected document or, given
`--manifest`, holds bytes the manifest does not vouch for. `--only`,
`--baseline`, `--write-baseline` and `--latency-gate` work as for the other
modes; an extract-only run writes and is held to an extract-only baseline
(see [Gates](#gates)). The report gives the run's wall time. Over the 72
documents of the phase 2 corpus, on the shared 4-core development container
it is about 45 s with the routing worker and about 70 s with the one before
it, which read a PDF's scanned pages one at a time; Tesseract takes nearly
all of it (the 25-page scanned lease alone 14 s), and the 52 documents that
are not scans take under a second together. It can be run on every change
to the worker, or with `--only` on the documents a change touches.

The scoring is `extract::extraction_record`, a pure function of the gold,
what the worker returned and the timings; the worker loop around it only
feeds it.

### Comparing two runs

```sh
intern-bench compare --before before.json --after after.json --markdown diff.md
```

This aligns the two reports by document. Every rate, mean and count is
computed again over the documents both runs scored, and each score over the
documents that have it in both, so a subset run, a document added since or
one that went stale moves no figure; the comparison says when the two runs
cover different documents or gold. It lists every document that flipped on
every score (fixed or broken), and every value the extract-only gate holds
(structure scores, OCR's accuracies and its character and word edit
distances, digest recall) that moved further than the gate tolerates, worse
or better. Latency, as the change in p50 and p95 of every stage, is
compared over the documents both runs completed, and only between two runs
that measured their timings (live or extract-only): a replay reports its
recording's timings, so a comparison involving one shows no latency change
and says why. The comparison opens with the [phase 3
scorecard](#the-phase-3-scorecard) before and after, and gives each slice
(`long`, `complex`) its filename, date, parties and routing rates, its
unsafe-ready count and its total p50 and p95, over the slice's documents
both runs scored (latency over those both completed). Each side names its
machine and, for a replay, its recording. `intern-bench report
--input report.json --markdown report.md` re-renders a report's Markdown.

## Reading a report

`report.json` has these sections:

* `summary`: every boolean score as `{correct, total, rate}` and every
  fractional score as a mean, plus counts of unsafe-ready, trap-date,
  forbidden-party and unsupported-claim outcomes.
* `groups`: the same summary sliced by `kind`, `text_layer`, `format`, page
  bucket, route class, category and `slice` (`long` and `complex`, see
  [the phase 3 scorecard](#the-phase-3-scorecard)).
* `scorecard`: the phase 3 comparison list, each figure with the documents
  it is over (absent for an extract-only run, which names nothing).
* `latency`: p50, p90, p95, max and mean of every timing, overall and by
  page bucket, kind, text layer, route class and slice, over the documents
  that completed (a failed document's time is only how long it took to fail). A
  document's route class is the most expensive route any of its pages took
  - `ocr`, then `ocr_regions`, then `layout`, then `fast` - or `unrouted`
  when the worker sent no layouts (every worker before the router, and every
  recording made with one).
* `ocr`: OCR figures per scanned document and pooled.
* `structure`: the structure figures per document and pooled by item
  (absent when no document has a structure block).
* `routes`: pages per route, documents per route class, the pages the
  gold gives a route for (`expected_pages`), and the confusion of expected
  route against the route taken over those that could be judged. It is
  there whenever the gold expects a route or a page came with a layout - a
  run of a worker that sends no layouts still shows the expected routes
  went unjudged - and absent only when neither holds.
* `wall_ms`: the whole run, worker start included (live and extract-only).
* `memory`, and one record per document holding the name, the
  description, the review reasons, the scores, the claims checked, the traps
  sprung, the structure items missed, the route each page took and the
  timings.

Keys are sorted and floats rounded, so two runs diff cleanly.

`report.md` is the same report for a person. It opens with a scorecard, the
phase 3 scorecard and a safety table, then the per-group tables (a slice
table among them, with generation time), OCR, structure and routes, the
stage-by-stage latency table with tokens per second, memory, and
**Misses**. Misses lists every wrong filename with the expected name, the
trap it sprang and why, and whether it was filed without review. An
extract-only report opens instead with an extraction scorecard (each score
pooled by item and as a mean per document), then structure, routes, OCR,
the worker's time by route class, page count and kind with its stages, and
Misses: the documents the worker failed on and every structure item not
found.

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
| `unsupported_fact_doc` | Validation sent the document to review because something the model gave is not supported by the document: any review reason ending `_UNSUPPORTED` (`DATE_UNSUPPORTED`, `TYPE_UNSUPPORTED`, `PARTY_UNSUPPORTED`, `DESCRIPTION_UNSUPPORTED` today), whichever fact it was about, so the old and new pipelines are counted alike. Good when false; false for a document that failed, which asserted nothing. Its rate is the scorecard's unsupported-fact rate, by document; `unsupported_fact_rate` in the summary stays the share of description claims. |
| `ocr_cer`, `ocr_wer`, `ocr_date_accuracy`, `ocr_name_accuracy`, `ocr_identifier_accuracy` | OCR against the drawn text. Levenshtein is computed per page, and the totals are pooled. Whitespace and the `|` rules a layout writes around a table row are set aside on both sides first, so a page whose text is its blocks is not charged for its tables' rules. A colon is text and is kept: one the page printed and the reading dropped is a miss, and the colon a layout writes after a label (`Label: value`) where the page printed none costs a character, and its word. Typographic quotes and apostrophes (`‘ ’ “ ”` and their low forms) are folded to `'` and `"` on both sides: a glyph style that carries no filing information, and PP-OCR, whose recognition dictionary has no curly quotes, emits every one straight. Nothing else is folded, so a misread such as `0ccurrence` still counts. A scanned page the extractor did not return (a TIFF frame it does not read), or every page of a scan whose extraction failed, counts as read empty. |

A score the gold does not define is omitted rather than counted as false,
so a rate's denominator is the documents that could be right or wrong about
it: every document for most scores, and for the date role, the relation word
and the party's role, the documents whose answer gave them something to
judge. A document that failed is a miss on every score its reviewed answer
would be judged on, those three included.

### The phase 3 scorecard

The figures phase 3 (evidence retrieval and fact-only replies) is judged
by, in the report, its Markdown and `compare`:

| Figure | What it is |
| --- | --- |
| Long-document filename accuracy | `filename_correct` over the `long` slice: documents of 10 pages or more. |
| Complex-document filename accuracy | `filename_correct` over the `complex` slice (below). |
| Description completeness | The mean of `description_completeness`. |
| Unsupported-fact rate | The share of documents with `unsupported_fact_doc`. |
| Review rate | Of the documents that completed and were named, the share sent to review. |
| Evidence recall | The mean of `evidence_recall`. |
| Total latency p50, p95 | `total_ms` over the documents that completed. |
| Generation latency p50, p95 | `generation_ms`, likewise. |
| Generated tokens p50, p95 | `generated_tokens`, likewise. |
| Prompt tokens p50 | `prompt_tokens`, likewise. |

A replay's latency and tokens are its recording's, and `compare` shows them
only between two runs that measured their own.

The `complex` slice is every document with any of these categories:
`referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`,
`irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`,
`key_value`, `complex_pdf`. Each makes what the document says something to
find or to reason out: a fact in the middle, a referenced agreement's own
date and parties, names that are not parties, columns or a content-stream
order that scramble the reading, a date or labelled value inside a table,
parties placed only by the layout, a page dense with figures. Three groups
are left out on purpose. `competing_dates` is on 62 of the 72 documents of
the corpus as it was when the slice was drawn, so it would make the slice
the corpus. `table` is on half of them, mostly routine header tables;
`date_in_table` and `key_value` keep the tables that decide the name. And
the scan conditions (`rotated_scan`, `noisy_scan`, `low_resolution_scan`
and the like) measure OCR, which `text_layer` already slices. On that corpus
the slice holds 51 documents (14 of them by `referenced_agreement` alone)
and `long` holds 7, 6 of them also complex. The five long documents added
for phase 3 are in both, so the 77-document corpus has 56 complex and 12
long; until they are recorded a replay scores 33 and 7 of them.

### Structure scores

Each is computed over the page text the engine receives - the text the
digest and the prompt are built from - never over the worker's blocks, so a
worker that sends no layouts is scored on the same footing as one that does.
Only `route_correct` reads the layouts. The code is
`crates/intern-bench/src/structure.rs`.

Text is compared normalised: every run of whitespace one space, typographic
quotes straight, dashes hyphens. Case counts. A gold string is found only
where it stands on its own: one that begins or ends with a letter or digit
may not continue a word there, and one that begins or ends with a digit may
not continue a number (`4` is not found in `14`, `1,4` or `4.5`). The text
is read as *lines*: each page's text split at line breaks, in page order,
normalised, blank lines dropped; a table row linearised as `| a | b |` is
one line.

| Score | Exact definition |
| --- | --- |
| `reading_order_accuracy` | Each snippet's position is its first occurrence in the whole text (pages joined by a line break, normalised). The snippets in order are the most of the found ones whose positions rise in the gold's order (a longest increasing subsequence); the score is their number over the number of gold snippets, so a snippet not found is out of order. Two whole columns read the wrong way round keep only the longer column in order: of eleven snippets, seven and four, the score is 7/11 (about 0.64), where counting consecutive pairs would have lost one pair in ten. |
| `table_row_accuracy` | A gold row (its non-empty cells) is found when one line of its table's region (see `table_cell_recall`) holds every cell in order, each starting after the end of the one before, with no cell of another row of the table wholly between two of them, so two rows read across each other are neither found. A blank leading cell counts too: when some rows of a table leave it blank and others do not (a check-box group, its chosen options marked `X`), the values the others hold there are the table's marks, and a row with a blank leading cell is not found on a line where a mark stands between the cell of another row before it (or the line's start) and its first cell - that `X` is beside an option the gold leaves unmarked. The share of rows found, over every table of the document. |
| `table_cell_recall` | The share of non-empty gold cells found in their table's region: the lines from the first one holding a cell of the table's first row (or the first line, if none does) to the last one, from there on, holding a cell of its last row (or the last line, if none does). Only cells of two characters or more place the region - a lone `X` or digit is printed all over a page, so it neither anchors nor widens a table - and a first or last row without one gives way to the nearest row with one. A table split over two pages spans the break. A value the table prints k times must be found k times, without overlap. |
| `kv_accuracy` | A labelled value is found when a line holds the label and, after it but before the next gold label on the line, the value (a value after another label is that label's); or a line holds the label and nothing else (colons, bars, dashes, full stops aside) and the next line holds the value; or a table row holds the label as a whole cell (a colon after it aside; `CONTRACT DATE` is not the cell of `DATE`) and the next line, a row of the same table, holds the value in the cell of the same column (a label set over its value, linearised as a table). The share of pairs found. |
| `route_correct` | The share of pages with an expected route whose layout took that route. A page sent without a layout while others have one took none and is wrong. Not scored when the worker sent no layouts at all, nor for a document whose extraction failed (nothing was read, so nothing was routed). |

Every count behind them is in the record (`structure`), so a corpus figure
pools items rather than averaging documents: the report's `structure`
aggregate divides found items by gold items over every document. The
`reading_order`, `table_row`, `kv` and route scores move by whole items; the
OCR error rates are the only continuous extraction scores.

Two consequences worth knowing. A table cell that wraps, read on the fast
route (the native text gives each line of a row in turn), is not one
contiguous string, so neither its row nor the cell is found; that is
deliberate, since the engine sees the cell broken too. And a label printed
more than once is credited if any occurrence carries its value.

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

`--write-baseline bench/baseline.json` records a run as the baseline. A
run limited with `--only` refuses to overwrite a baseline that covers
other documents: the documents left out would read as new, and new
documents gate nothing. A replay that could not score every document, or
scored some from stale replies, never writes a baseline; nor does a run of
any mode in which no document completed (a worker that would not start,
say), which would hold every later run to having read nothing. Either
refusal exits 2.
`--baseline bench/baseline.json` compares a run with it:

* **Replay is gated per document.** A score that was good and is now bad is
  a regression, as in `intern-evaluate`. Trap, forbidden and unsafe scores
  are good when false. Replay is deterministic, so any flip is real. A
  change of status is a regression too, except a document that failed in
  the baseline and completes now: that is an improvement, and its scores
  are still held to the baseline's.
* **A full run that drops a baseline document fails**, live or replay: a
  gold entry deleted by accident would otherwise take its coverage with it.
  So does a baseline document whose gold entry says `"recording":
  "pending"`, which replay leaves unscored; pending is only for a document
  nobody has recorded yet. Write a new baseline to drop a document on
  purpose.
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
  A stage the baseline measured for such a document and the run did not
  (a worker that reports no timings) fails the gate rather than shrinking
  its sample. Use it only between runs on the same machine.
* **Extract-only is its own mode.** Its baseline (`"mode": "extract"`) also
  keeps each document's fractional extraction scores, and a run is held to
  it document by document, as replay is. Extraction is deterministic for one
  worker build on one machine - two runs over the corpus produce the same
  scores, routes and texts - so a fall is a real change. Every structure
  score, OCR's date, name and identifier accuracy and `digest_recall` count
  whole items, so any fall fails. OCR's error rates are held as the edit
  distances they are made of, which the baseline keeps per document
  (`ocr_char_distance`, `ocr_char_distance_ci`, `ocr_word_distance`): a
  document regresses when its character edit distance rises by more than
  three characters, or its word edit distance by more than one word. A
  tolerance on the rate would be a share of the page, so a long scan could
  lose a line (0.005 of 6,000 characters is thirty) where a short one could
  lose three characters; a count gives every page the same room - a
  character or two misread by another Tesseract build - while the date,
  name and identifier accuracies still catch any critical field lost. The
  rates themselves, and `ocr_mean_confidence`, are reported, never gated.
  A status that changes fails as in replay, and latency is gated only with
  `--latency-gate`, over the worker's stages. An extract-only run is never
  held to a live or replay baseline, nor the other way round.

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
