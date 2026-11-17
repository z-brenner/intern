# Evidence retrieval

Phase 3 replaces "one heuristic compressor decides what text the model
sees" with an index over the document's blocks and retrieval per field.
This page describes the index and the retriever (Stages 3 and 4 of the
phase), how they were tuned offline on InternBench and the fixture corpus,
and what the measurements say. It was written when the retriever was
measured beside the digest, before anything sent it; since Stage 8 the
evidence pipeline built on it is the default for the local model
([`evidence-pipeline.md`](evidence-pipeline.md)), and the digest is what a
hosted model reads. The acceptance table at the end records Stage 4 as it
stood. In November 2026 the evidence was widened - larger budgets, and
documents up to 3,000 tokens sent whole - because the model was reading
about a tenth of every long document; [the chosen
configuration](#the-chosen-configuration) gives the measurements, and the
sections after it describe Stage 4's configuration, which
`RetrievalConfig::stage4()` keeps.

## The index

`crates/intern-engine/src/index.rs` builds an `EvidenceIndex` from a
`DocumentSource`. It reads the document through `structure::structured`,
so a page the parser worker sent with a layout keeps the worker's blocks,
and a page without one (a stored analysis from before layouts, a plain-text
source, an old recording) is segmented exactly the way the worker segments
text it has no geometry for. `structure::segment` now mirrors the worker's
`blocks_from_text` - Markdown and capital-line headings (not a line ending
in a colon), `|` tables with their header row, runs of `Key: value` lines
as labeled values, list items, paragraphs split after twelve lines at a
sentence end - and the worker's contract test
(`the_engine_segments_text_exactly_as_the_worker_does`) holds the two
equal, block for block.

A unit is:

| Block | Units |
| --- | --- |
| Heading | one unit |
| Paragraph, list item, caption, other | the block, or sentence-bounded chunks of at most `max_unit_chars` (`.s1`, `.s2`, ...) |
| Table | one unit per row (`.r1`, ...), its `\| a \| b \|` line; the header row (all cells headers, or a first row with no figures) is a `TableHeader` unit every row points to; a two-cell row's first cell is its label |
| Key-value | one `Field` unit per labeled value (`.f1`, ...), its `Key: value` line, the key as its label |
| Page header or footer | a running unit |

Every unit keeps its block id and its own id, its page, its kind, its
verbatim text (always a piece of its page's text), the headings it falls
under, its table's header row, whether its text is native or OCR and how
confident OCR was. A repeated page header or footer, and any short line
whose digit-masked shape recurs on half the pages (the rule distillation
collapses running lines by), is `running` after its first instance and
never retrieved.

Features are read once per unit:

* **Dates**, each with the wording before it read by validation's own
  functions: the role (`role_from_wording` over `window_before`), whether
  it is labeled a deadline (`labels_a_deadline`) or the issue date
  (`labels_the_issue_date`), whether it is another document's date
  (`reference_introduced`), and whether it defines a dated term (`"Closing
  Date" means`). Numeric dates are read in the order the whole document
  settles (`numeric_date_order_of`). A test holds these readings equal to
  validation's on the corpus fixtures' date lines. Beside them, a date in
  a table row also takes its column's header as a label (`| Invoice Date
  | Due Date |` over the row's dates), which nothing on the row's own line
  says, and a date standing alone on its line is a dateline.
* **Defined terms**: every `"X" means` entry in a unit, so the definition
  of the Services, the Products or the Premises reads as subject evidence.
* **Money** (amounts, and whether a label such as Total, Rent or Principal
  makes one a key fact), **likely organizations** (capitalized runs ending
  in a legal form or an institution word), **likely people** (after a
  courtesy title, "Dear", a party label, `/s/`), **identifiers** (letters
  and figures, or figures a number label introduces).
* **Cue counts** from the lists distillation and inference already use,
  moved unchanged into `cues.rs` and re-imported there.
* **BM25 terms**: folded, split, stopwords dropped, lightly stemmed; the
  unit's own words count twice, its heading's and its table header's once.
* **Position and structure**: first and last page, the document's opening
  and closing units, the first page's letterhead, a signature block, the
  first two headings (where a title is), the first paragraph, and the
  section tags of its headings (definitions, term, parties, recitals,
  invoice, bill-to, signature, notices, schedule, payment, scope).

Sections are the spans under top-level headings (one per page where no
heading covers the text).

## Retrieval

`crates/intern-engine/src/retrieve.rs` retrieves for six fields
independently: document type, date, parties, subject, identifier and key
facts. For each field every unit gets an integer score, the sum of six
components, each weighted in percent:

| Component | What it reads |
| --- | --- |
| cues | distillation's cue lists: type, party, date-role, subject, signature cues, the boilerplate penalty |
| bm25 | BM25 (k1 1.2, b 0.75, fixed-point IDF) against the field's lexicon, scaled 0-100 to the best unit |
| structure | headings and titles, labeled values with a matching label, letterhead, signature block, table header context |
| heading | the section's tags: definitions for a date, parties and recitals for parties, scope for the subject |
| position | the document's opening and closing, first and last page |
| features | dates with their roles (deadline -60, reference -80, defined term +60), organizations and people, identifiers, money |

BM25 is implemented and measured but weighted 0 in the default: the sweep
below found it adds nothing the cue lists do not, and costs subject recall.

Each field takes units best-first until its own budget of estimated tokens
(`engine::estimated_tokens`, so digits and CJK count fully) is spent. The
parties take units by what they add - a name not yet taken, worth less
after the first six names - so five copies of one clause buy nothing and a
schedule of lenders cannot fill the budget. A date is carried by at most `units_per_date`
units, so a statement of dated rows cannot fill the date budget. Repeats
are dropped by distillation's own keys (clause number stripped, digits
masked; dates exempt from the digit-masked key, because two lines that
differ in their digits may differ in the date). A unit brings its context,
paid from the field's budget and capped at `expansion_pct`: its table's
header row, the heading over it, and for a short unit the unit after it.
The context is the union, in document order, one line per unit:

```text
[p1.b1] CREDIT AGREEMENT
[p1.b2] This Credit Agreement is dated as of the Closing Date by and between ...
[p20.b1] ARTICLE I DEFINITIONS
[p20.b2] "Closing Date" means June 12, 2026.
```

Because every field has its own budget, type evidence cannot crowd date
evidence out, and parties cannot crowd out the subject; a test holds the
date and subject selections unchanged when the type budget is multiplied
by sixteen.

A document whose units fit `whole_document_tokens` goes whole (the `Whole`
tier), still as `[id] text` lines. Otherwise the tier is `Normal`, or
`Dense` (budgets x1.5) when a trigger fires: six or more candidate dates
whose two best scores are within 15%, five or more organizations named
where parties are, or mean OCR confidence under 80.

Hierarchical retrieval scores sections per field first (best unit + a
quarter of the best three + a tag bonus), keeps the best
`sections_per_field`, the first page and - for parties and the date - the
signature section, and scores units only inside them.

Everything is an integer and every collection ordered: the same index and
configuration give the same context byte for byte, and
`RetrievalConfig::fingerprint()` (SHA-256 of the configuration's canonical
JSON) names a configuration for recordings. Index and retrieval run inside
`prepare_evidence`, which turns a panic into `ANALYSIS_FAILED` the way the
analysis guard does; `tests/retrieval_fuzz.rs` throws seeded random pages
(multi-byte text, dates in every spelling, tables, labels, list markers,
running lines) at both.

## Measuring without a model

`intern-bench` computes, from a document's extracted text alone:

| Score | Meaning |
| --- | --- |
| `context_type_recall` | a context unit holds at least 60% of the significant words of the gold type or an acceptable one |
| `context_date_recall` | a reviewed form of the date, or the date or an acceptable one in any spelling, is in a context unit |
| `context_party_recall` | share of gold parties with a reviewed form in a context unit |
| `context_recall` | the mean of those three: the filename's items |
| `context_fact_recall` | share of description facts with a form in the context |
| `context_subject_recall` | share of subject terms in the context |

and, with the timings, `context_tokens`, `context_units`, `index_units`,
`index_ms` and `retrieval_ms`. Extract-only runs report them next to
`digest_recall` (and an extract baseline gates them with no tolerance).
Replay and live runs report all of them too. In a replay, `index_ms` and
`retrieval_ms` are measured when the report is made, from the recorded
text; the rest of the timings are the recording's.

`intern-bench retrieval` sweeps configurations over recorded sources,
needing no worker and no model:

```sh
intern-bench retrieval --recording RECORDING.json --gold bench/gold.json \
  --fixtures fixtures/corpus-recording.json --expected fixtures/expected.json \
  [--config mine.json] [--sweep] --output sweep.json --markdown sweep.md
```

Documents are split into a tuning half and a held-out half by the parity
of the first byte of the SHA-256 of their id. Configurations are ranked on
the tuning half - fewest violations (a document whose digest carried the
date and every party, and whose context lacks the type, the date or a
party), then filename recall, fact recall, subject recall, then fewest
tokens on documents of ten pages or more - and the held-out half is
reported beside it. `--dump DIR` writes every document's units with their
six field scores, the context marked by the field that chose each unit;
that is how the failures below were read.

## What was measured

The sources are those of the live InternBench run of 2026-10-07 made at
`bd48e98` with the phase 2 routing worker: all 72 documents, every page
with its layout. To them are added the 19 fixtures of `fixtures/` that the
committed fixture recording holds a parsed source for (two are expected
extraction errors), with `fixtures/expected.json` as their gold: the type,
the date, the parties and each description fact. That is 91 documents, 47
in the tuning half and 44 held out. As a check of the fallback path, the
same configurations also ran over two recordings without layouts:
`bench/recording.json` (52 documents, recorded at `0974ab9`) and a live
run of all 72 documents with the pre-router worker. None of this needs a
model; the whole sweep of 91 configurations takes about ten seconds.

Most of the corpus is short: 75 of the 91 documents fit the whole-document
threshold and go whole, so they say little about how retrieval chooses.
Every component ablation was therefore also run with retrieval forced on
every document (`whole_document_tokens` 0), which is where the
components' worth shows.

One caution. The features added during tuning - column headers as date
labels, datelines, defined terms, subtitles, line items - came from
reading the failures of documents in both halves. The split guards the
choice of configuration (weights, budgets, thresholds), not the design of
the features. The two recordings without layouts and the fixture corpus
were not used to design anything and show the same results.

## The chosen configuration

`RetrievalConfig::default()`, from November 2026:

| Setting | Chosen | Stage 4 (`RetrievalConfig::stage4()`) | Plan start (`plan_start()`) |
| --- | --- | --- | --- |
| Components | cues, structure, heading, position, features at 100; BM25 at 0 | same | all six at 100 |
| Budgets (tokens) | **type 180, date 675, parties 525, subject 450, identifier 180, key facts 300** (Stage 4's x1.5) | type 120, date 450, parties 350, subject 300, identifier 120, key facts 200 | parties 450 |
| Dense tier | 150% of the budgets, on the triggers | same | 175% |
| Whole document | **up to 3,000 tokens** | up to 1,800 | 1,800 |
| Strategy | hierarchical from 300 units, flat below | same | same |
| Units | paragraphs split at 400 characters; 2 units per date; expansion up to 25% of a field's budget; stable ids | same | same |

### Why it was widened

Stage 4 chose the fewest tokens that kept every filename item
([below](#stage-4s-choice)). That left the model reading little: of the 96
documents measured here, one of more than 1,800 tokens was sent a median
12% of its own text (10% from four pages up; the 60-page property policy
about 2%), and the median prompt carried 758 tokens of evidence in a
context of 8,192. The type, the date and the parties survived that. What
did not is what a description needs: Stage 4's context lacked twelve of
the gold's subject terms and facts, on eight documents, and on seven of
them the digest carried at least one of those items.

**What a token costs.** On a CPU the cost of a prompt is its prefill, and
the instructions are read once and kept in the slot (`slots.rs`), so what a
document pays for is its own user turn, which is its evidence. Measured:

| Model | Machine | Prefill | Per 1,000 prompt tokens | From |
| --- | --- | --- | --- | --- |
| 9B | AMD EPYC 7763, 4 threads, repacked weights | 70 ms a token, linear from 4 to 2,080 tokens (fit over the 77 documents, intercept about 0) | 70 s | the timings in `bench/recording.json` |
| 9B | AMD EPYC 7763, 2 threads, `--no-repack` | 70 s of prefill a request over the 13 battery requests | about 70 s | [`speed-linux-cpu-2026-11-12`](../bench/reports/speed-linux-cpu-2026-11-12/README.md) |
| 9B | AMD EPYC 9V45 / 9V74, AVX2 path, `--no-repack` | 27 / 19 tokens a second | 37 / 52 s | [`speed-linux-cpu-2026-11-11`](../bench/reports/speed-linux-cpu-2026-11-11/README.md) |
| 4B | AMD EPYC 7763, 2 threads, `--no-repack` | 508 s of prefill over the 13 battery requests, against the 9B's 911 s on the same processor: 0.56 of the 9B | about 39 s | [`speed-linux-cpu-2026-11-12`](../bench/reports/speed-linux-cpu-2026-11-12/README.md) |

An estimated token (`engine::estimated_tokens`, which counts every digit
as one) is 0.91 of the model's over the same documents (the server's
prompt tokens against the context's estimate: 0.91 x estimate + 113), so
1,000 estimated tokens of evidence cost about 64 s on the 9B and 36 s on
the 4B on the slowest processor measured, and roughly half that on the
faster ones. The reply - about 90 tokens, 16 s on the 9B - does not change.
The seconds below are the EPYC 7763's: the worst case among the runners,
and about what a laptop's AVX2 path does.

The rule applied this time: send more wherever it carries more of what a
description needs, as long as an excerpted document's model time grows by
about 30% or less on the 9B; and send whole any document that fits in what
a long document is now sent.

### Measured

`intern-bench retrieval` over the sources of `bench/recording.json` (the 77
InternBench documents recorded live at `c427aff` with the routing worker,
every page with its layout) and the 19 parsed fixtures of
`fixtures/corpus-recording.json`, with their gold: 96 documents, 48 tuning
and 48 held out. Recall in percent. "Losing" counts the documents whose
context carries less of an item kind than the digest. The share seen is the
context's tokens over the whole document's, at the median, for the 21
documents Stage 4 excerpts. Seconds and percent are model time added on the
9B at the rate above, against Stage 4: at the median over the documents
excerpted under both, and the most any one document gains. Filename recall
(type 99.0, date 97.9, parties 99.5; tuning 99.3, held out 98.2) and
violations (none) are the same in every row.

| Configuration | Facts tune / held out | Subject tune / held out | Facts, subject (all) | Documents losing a fact / a subject term | Tokens p50 / p90 / max | Share of an excerpted document seen | 9B s added: excerpted p50 / most | 9B time added, excerpted p50 (max) | Sent whole |
|---|---|---|---|---|---|---|---|---|---|
| Stage 4 | 97.2 / 96.1 | 98.2 / 95.1 | 96.7, 96.8 | 2 / 5 | 758 / 1,543 / 2,022 | 12% | - | - | 75 |
| Stage 4, budgets x1.25 | 97.9 / 96.1 | 97.6 / 95.8 | 97.0, 96.8 | 1 / 3 | 787 / 1,706 / 2,472 | 15% | 15 / 32 | 16% (28%) | 75 |
| Stage 4, budgets x1.5 | 97.9 / 96.1 | 99.4 / 96.8 | 97.0, 98.2 | 1 / 2 | 801 / 1,806 / 3,010 | 16% | 32 / 66 | 30% (44%) | 75 |
| Stage 4, budgets x2 | 97.9 / 96.1 | 99.4 / 96.5 | 97.0, 98.0 | 1 / 2 | 816 / 2,222 / 4,009 | 22% | 60 / 132 | 57% (90%) | 75 |
| Stage 4, description budgets (subject 600, key facts 400, parties 450, identifier 180) | 97.9 / 96.1 | 99.4 / 96.5 | 97.0, 98.0 | 1 / 2 | 801 / 1,854 / 2,872 | 18% | 35 / 59 | 35% (48%) | 75 |
| Stage 4, always dense | 97.9 / 96.1 | 98.8 / 95.1 | 97.0, 97.1 | 1 / 4 | 787 / 1,543 / 2,022 | 13% | 0 / 33 | 0% (44%) | 75 |
| Stage 4, whole up to 3,000 | 97.9 / 96.1 | 98.2 / 95.1 | 97.0, 96.8 | 1 / 5 | 787 / 1,706 / 2,406 | 12% | 0 / 95 | 0% | 79 |
| Stage 4, whole up to 4,000 | 97.9 / 96.1 | 98.2 / 95.1 | 97.0, 96.8 | 1 / 5 | 801 / 1,832 / 3,789 | 12% | 0 / 182 | 0% | 81 |
| **chosen** (budgets x1.5, whole up to 3,000) | 97.9 / 96.1 | 99.4 / 96.8 | 97.0, 98.2 | 1 / 2 | 837 / 2,047 / 3,010 | 16% | 33 / 95 | 30% (44%) | 79 |
| chosen, always dense | 97.9 / 96.1 | 99.4 / 96.8 | 97.0, 98.2 | 1 / 2 | 868 / 2,047 / 3,010 | 20% | 44 / 95 | 35% (105%) | 79 |
| chosen, dense 175% | 97.9 / 96.1 | 99.4 / 97.7 | 97.0, 98.6 | 1 / 1 | 837 / 2,195 / 3,556 | 19% | 48 / 101 | 44% (67%) | 79 |
| chosen, budgets x1.75 | 97.9 / 96.1 | 98.8 / 97.7 | 97.0, 98.3 | 1 / 2 | 837 / 2,195 / 3,556 | 19% | 48 / 101 | 49% (67%) | 79 |
| chosen, whole up to 4,000 | 97.9 / 96.1 | 99.4 / 96.8 | 97.0, 98.2 | 1 / 2 | 837 / 2,297 / 3,789 | 16% | 37 / 182 | 31% (44%) | 81 |
| chosen, date and parties at Stage 4's | 97.9 / 96.1 | 99.4 / 95.8 | 97.0, 97.7 | 1 / 2 | 837 / 1,942 / 2,442 | 14% | 22 / 95 | 19% (30%) | 79 |
| every document whole | 97.9 / 97.9 | 100.0 / 100.0 | 97.9, 100.0 | 0 / 0 | 876 / 9,033 / 116,181 | 100% | - / 7,294 | - | 96 |

What it says:

* **Budgets x1.5 is the knee.** It brings back five of the twelve items
  Stage 4's context lacked - "service levels" in the services agreement,
  "earn-out" in the asset purchase agreement, "term loan" in the credit
  agreement, "dock" in the industrial lease and `FE-032` on the inspection
  log - and loses none, for 30% more model time on the median excerpted
  document (at most 44%). x1.25 found three of them and lost three subject terms of the
  services agreement on the way. x2, or the increase spent on the
  description's fields alone, found as many (GDPR in place of dock) at
  twice the cost. Holding the date and the parties at Stage 4's budgets
  lost "dock": the lines those fields bring carry subject words too. A
  dense tier at 175%, or budgets x1.75, found one more ("warehouse") for
  44-49%.
* **Always dense adds cost, not evidence.** The triggers already make
  every long InternBench document dense except two scans; forcing the rest
  found nothing more.
* **Whole up to 3,000** sends four 2- and 3-page documents of 1,832 to
  2,406 tokens whole, where Stage 4 read a quarter to a half of each; one
  of them, the
  inspection log, carries the `FE-032` only the digest had. It costs those
  four 69 to 95 s more on the 9B (+68% to +174%), and nothing to any other
  document. 3,000 is about the most the widened budgets send from a long
  document at the dense tier (3,010 measured), so no document costs more
  whole than a long one costs excerpted, and the slowest document is no
  slower for the threshold. Up to 4,000 adds two documents of 3,078 and
  3,789 tokens, at up to 182 s each, for no item.
* **What is still missed** is what Stage 4 found missed: the data
  processing agreement's "GDPR", "student data" and "forty-eight hours"
  (in its definitions and Annex I), "warehouse" in the industrial lease,
  "tenant improvement" in the 25-page scanned lease, and "Vermont" and
  "brewery" in the 60-page policy - each in the document, and ranked below
  the budget. Only sending them whole recovers all of them, at 9,033
  tokens on the tenth-largest document and 116,181 on the largest.

### What it costs

Over the 96 documents:

| | Stage 4 | Chosen |
| --- | --- | --- |
| Evidence tokens, p50 / p90 / mean / max | 758 / 1,543 / 820 / 2,022 | 837 / 2,047 / 967 / 3,010 |
| Context p50 by pages: 1-3 / 4-9 / 10-24 / 25+ | 676 / 928 / 958 / 1,736 | 685 / 1,193 / 1,259 / 2,312 |
| Documents sent whole / at the normal tier / dense | 75 / 8 / 13 | 79 / 5 / 12 |
| Long documents (10+ pages): context smaller than the digest by | 68% in total | 55% in total; two under 40% (the asset purchase agreement 36%, the credit agreement 34%) |

| Documents | Count | Evidence p50, Stage 4 -> chosen | 9B seconds added, p50 (range) | 9B time added, p50 (range) | 4B seconds added, p50 |
| --- | --- | --- | --- | --- | --- |
| Sent whole by both | 75 | 673 -> 673 | 0 | 0 | 0 |
| Excerpted by both, at the larger budgets | 17 | 1,415 -> 1,844 | 33 (9-66) | 30% (18-44%) | 18 |
| Sent whole now (1,800 to 3,000 tokens) | 4 | 742 -> 2,022 | 79 (69-95) | 109% (68-174%) | 44 |

The median document's time does not change: it went whole and still does.
Over the whole corpus model time rises 12%, the 9B's median from 73 s to
78 s a document and its 90th percentile from 122 s to 149 s (the median
moves because four documents below it are now above it). A set with more
long documents, or more 2- and 3-page documents of 1,800 to 3,000 tokens,
pays more of it than InternBench does. The 40% long-document saving was a
Stage 4 gate on spending tokens; two documents now fall short of it, and
the sweep's ranking, which still puts that gate before recall, places the
chosen configuration 25th of its 98. That is the trade made on purpose: the
gate measured tokens against the digest, and the digest is not what the
local model was short of.

**The context window is not the limit.** The widest evidence the
configuration sends - a whole document at 3,000 tokens, or a long one's
excerpts at the dense tier, at most 3,465 by its budgets and 3,010
measured - fits the local server's 8,192 tokens beside the instructions,
the longest file name a prompt shows, the template and the whole
1,024-token reply: 5,417 estimated tokens at most, 2,775 short of the
8,192 (`the_widest_default_evidence_fits_the_local_context`). So the context
stays at 8,192 (`server::CONTEXT_TOKENS`). The server sizes its KV cache by
`--ctx-size` at load, whatever the prompt, so this change costs no memory;
a larger window would cost the 9B memory, the binding constraint on a
16 GB laptop ([`architecture.md`](architecture.md#two-models-one-at-a-time)),
for evidence nothing here asks for. The fitting that keeps a prompt inside
the context (estimate, retrieve again smaller, at most twice; halve once
more if the server still refuses) is unchanged, for what the estimate
cannot see.

The prompt also shows the file's current name when a person chose it
([`evidence-pipeline.md`](evidence-pipeline.md)): one line, about 25 tokens
and the name, never a line a reply can cite. Neither evaluator passes the
corpus's file names, which are its ids and name the answer, so none of the
figures above include it.

What the model makes of the wider evidence takes the model: the recordings
of record have to be made again with this configuration, and that live run
is the measurement ([`evaluation.md`](evaluation.md#after-a-retrieval-or-prompt-change)).

### Stage 4's choice

Stage 4's configuration (`RetrievalConfig::stage4()`, the default until the
widening) is the middle column of the table above; the plan's values remain
available as `RetrievalConfig::plan_start()`. The sweep keeps both as
`stage4` and `plan_start`. This section and the ones after it measure
Stage 4's configuration - "chosen" in their tables - on Stage 4's 91
documents ([what was measured](#what-was-measured)).

How it was chosen: the sweep ranks configurations on the tuning half by
the Stage 4 gates first - no violation, fact recall not below the
digest's, no document of ten pages or more short of a 40% saving - then
by filename, fact and subject recall, then by long-document tokens. From
the plan's start, turning BM25 off gained subject recall on both halves
at no cost; the dense tier at 150% and the parties at 350 then brought
every long document under 60% of its digest's tokens without losing an
item. After that, the sweep's top places (`tier_dense`, no expansion,
whole up to 3,000 tokens, budgets x1.25) each beat the chosen
configuration on the tuning half by one or two items - `FE-032` on the
inspection log and "service levels" on the services agreement - and on
the held-out half by at most one item, some losing one, while sending more
tokens (always-dense: 777 tokens at the median instead of 729; whole up to
3,000: two to four times as many on three short documents). One or two of
the tuning half's 261 fact and subject items is within noise, and the rule
applied was that a configuration replaces the current one only by more
than that, or by matching it with fewer tokens. The chosen configuration ranks 18th of 91 by the strict key,
level on every gate with the first.

## Ablation

Stage 4's measurements, on its 91 documents, with "chosen" meaning its
configuration (`stage4` in the sweep today). Context recall in percent,
tuning half / held-out half; tokens at the
median over all 91 documents; the saving is the context's tokens against
the digest's summed over documents of ten pages or more, and the last
column counts those documents individually under a 40% saving. A
violation is a document whose digest carried the date and every party and
whose context lacks the type, the date or a party. Every configuration
carries the type, the date and the parties wherever the digest does
unless it shows a violation.

As configured (75 documents go whole):

| Configuration | Violations | Facts | Subject | Tokens p50 | Saving, 10+ pages | Under 40% |
|---|---|---|---|---|---|---|
| **chosen** (all but BM25) | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 66% | 0 |
| chosen without cues | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 715 | 66% | 0 |
| chosen without structure | 0 / 0 | 96.5 / 99.2 | 97.5 / 99.2 | 729 | 65% | 0 |
| chosen without heading | 0 / 0 | 97.2 / 98.5 | 97.5 / 98.4 | 727 | 67% | 0 |
| chosen without position | 0 / 0 | 96.5 / 97.7 | 96.2 / 96.9 | 724 | 63% | 0 |
| chosen without features | 0 / 0 | 96.5 / 97.7 | 98.1 / 97.7 | 724 | 67% | 0 |
| chosen with BM25 (all six) | 0 / 0 | 95.7 / 99.2 | 96.2 / 96.2 | 777 | 66% | 0 |
| cues alone | 1 / 1 | 90.4 / 94.6 | 91.7 / 96.4 | 685 | 71% | 0 |
| bm25 alone | 0 / 2 | 94.3 / 94.6 | 94.2 / 92.8 | 777 | 64% | 0 |
| structure alone | 4 / 2 | 93.6 / 96.1 | 94.4 / 96.9 | 673 | 73% | 0 |
| heading alone | 6 / 8 | 88.6 / 90.7 | 88.1 / 86.2 | 673 | 77% | 0 |
| position alone | 3 / 2 | 95.4 / 96.9 | 98.1 / 97.0 | 673 | 85% | 0 |
| features alone | 0 / 0 | 95.7 / 97.7 | 94.2 / 96.2 | 685 | 72% | 0 |
| cues + BM25 | 0 / 1 | 93.6 / 95.3 | 93.5 / 96.4 | 777 | 62% | 0 |
| cues + BM25 + structure | 0 / 1 | 95.0 / 97.7 | 96.0 / 97.8 | 777 | 62% | 0 |
| plan start (all six, plan budgets) | 0 / 0 | 97.2 / 99.2 | 97.5 / 96.2 | 777 | 58% | 1 |
| plan start without BM25 | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.7 | 777 | 58% | 0 |

Retrieval forced on every document:

| Configuration | Violations | Facts | Subject | Tokens p50 | Saving, 10+ pages | Under 40% |
|---|---|---|---|---|---|---|
| chosen | 0 / 0 | 96.5 / 93.8 | 96.0 / 93.5 | 595 | 66% | 0 |
| chosen with BM25 | 0 / 0 | 95.0 / 93.8 | 94.8 / 90.0 | 613 | 66% | 0 |
| without cues | 0 / 0 | 97.2 / 95.3 | 96.9 / 94.5 | 592 | 66% | 0 |
| without structure | 0 / 0 | 94.3 / 94.6 | 95.2 / 89.6 | 592 | 65% | 0 |
| without heading | 0 / 0 | 96.5 / 92.2 | 95.4 / 93.5 | 592 | 67% | 0 |
| without position | 0 / 0 | 95.7 / 93.0 | 94.8 / 94.5 | 592 | 63% | 0 |
| without features | 0 / 0 | 95.7 / 95.3 | 96.9 / 94.3 | 592 | 67% | 0 |
| cues alone | 5 / 4 | 70.6 / 80.6 | 80.2 / 81.0 | 388 | 71% | 0 |
| bm25 alone | 13 / 8 | 85.5 / 88.0 | 84.6 / 88.9 | 543 | 64% | 0 |
| structure alone | 7 / 4 | 80.8 / 73.6 | 78.8 / 88.8 | 347 | 73% | 0 |
| heading alone | 41 / 36 | 18.1 / 33.3 | 18.3 / 21.7 | 0 | 77% | 0 |
| position alone | 3 / 2 | 93.3 / 93.0 | 96.0 / 85.9 | 510 | 85% | 0 |
| features alone | 0 / 0 | 92.2 / 93.0 | 89.6 / 85.9 | 487 | 72% | 0 |
| cues + BM25 | 4 / 3 | 86.2 / 92.2 | 85.4 / 93.3 | 612 | 62% | 0 |
| cues + BM25 + structure | 0 / 1 | 89.7 / 93.0 | 91.7 / 93.1 | 621 | 62% | 0 |
| plan start | 0 / 0 | 96.5 / 93.8 | 96.0 / 90.0 | 643 | 58% | 1 |
| flat | 0 / 0 | 96.5 / 93.8 | 96.0 / 93.5 | 595 | 65% | 0 |
| hierarchical | 0 / 0 | 96.5 / 93.8 | 96.7 / 92.9 | 595 | 66% | 0 |
| normal tier only (no dense) | 0 / 0 | 94.0 / 92.2 | 95.4 / 91.1 | 592 | 76% | 0 |

What the components do:

* **Filename items** survive any one component's removal (no violation in
  any `without` row, forced or not). Alone, only the features - dates
  with their roles, organizations, people - keep every one: cue logic
  alone loses a date or a party on two documents (nine forced), structure
  alone six, position alone five (dates), BM25 alone two (21 forced), the
  heading component alone fourteen, because it only scores units under
  telling section headings.
* **Facts and subject** lean on structure and position. Forced, without
  structure the subject falls 0.8 / 3.9 points and the tuning half's facts
  2.2; without position the facts fall 0.8 on both halves, and as
  configured 0.7 / 1.5. Without the heading component the held-out facts
  fall 1.6 forced. Without the features component facts and subject move
  by about a point either way; it is the component that keeps the filename
  items on its own.
* **BM25** against a field lexicon is, here, a weaker restatement of the
  cue lists: added to the chosen configuration it lowers subject recall on
  both halves (98.1 / 97.7 to 96.2 / 96.2 as configured; 96.0 / 93.5 to
  94.8 / 90.0 forced), because words like "property", "loan" and "date"
  pull clause boilerplate in. At 25% or 50% it does not help either. It is
  kept, weighted 0, so the comparison can be run again.
* **Cue logic** - distillation's lists - is redundant with layout-derived
  structure on the routed corpus (without it: the same recall and 2% fewer
  tokens; forced, a point better). It is kept because the recordings
  without layouts need it: forced retrieval without cues falls from 96.5 /
  92.0 to 94.2 / 92.0 in facts on `bench/recording.json` and from 97.5 /
  93.2 to 95.8 / 93.2 on the pre-router run, and as configured the
  held-out facts fall from 94.9 to 93.5 and from 95.3 to 94.3. Stored
  analyses and plain-text sources have no layout.
* The **combinations** the plan names - cues with BM25, and with
  structure - each lose a party or a date somewhere until position and
  features join them.

## Parameters

| Configuration | Violations | Facts | Subject | Tokens p50 | Saving, 10+ pages | Under 40% |
|---|---|---|---|---|---|---|
| **chosen**: auto strategy (hierarchical from 300 units), whole up to 1,800 tokens, dense 150%, parties 350, 400-char units, 2 units per date, expansion 25% | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 66% | 0 |
| flat always | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 65% | 0 |
| hierarchical always | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.0 | 729 | 66% | 0 |
| hierarchical from 100 units | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.0 | 729 | 66% | 0 |
| hierarchical, 2 sections per field | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.0 | 700 | 66% | 0 |
| hierarchical, 5 sections per field | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 66% | 0 |
| budgets x0.5 | 0 / 0 | 95.7 / 97.7 | 93.5 / 95.3 | 668 | 82% | 0 |
| budgets x0.75 | 0 / 0 | 96.5 / 98.5 | 94.8 / 97.0 | 685 | 73% | 0 |
| budgets x1.25 | 0 / 0 | 97.9 / 99.2 | 97.5 / 98.4 | 766 | 58% | 0 |
| budgets x1.5 | 0 / 0 | 97.9 / 99.2 | 99.4 / 98.4 | 777 | 50% | 2 |
| parties 250 | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 715 | 68% | 0 |
| parties 450 (plan) | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 63% | 0 |
| dense 125% | 0 / 0 | 95.4 / 98.5 | 97.5 / 97.7 | 729 | 70% | 0 |
| dense 175% (plan) | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.7 | 739 | 61% | 0 |
| dense 200% | 0 / 0 | 97.2 / 99.2 | 98.8 / 98.4 | 741 | 57% | 0 |
| never dense | 0 / 0 | 95.4 / 98.5 | 97.5 / 97.7 | 729 | 76% | 0 |
| always dense | 0 / 0 | 97.9 / 99.2 | 98.8 / 97.7 | 777 | 64% | 0 |
| always small (60%) | 0 / 0 | 95.0 / 96.9 | 94.2 / 94.7 | 662 | 85% | 0 |
| never whole | 0 / 0 | 96.5 / 93.8 | 96.0 / 93.5 | 595 | 66% | 0 |
| whole up to 1,200 | 0 / 0 | 97.2 / 96.9 | 96.9 / 96.9 | 700 | 66% | 0 |
| whole up to 3,000 | 0 / 0 | 97.9 / 99.2 | 98.1 / 97.7 | 777 | 66% | 0 |
| units of 250 characters | 0 / 0 | 96.5 / 99.2 | 96.2 / 97.7 | 729 | 66% | 0 |
| units of 600 characters | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.7 | 708 | 65% | 0 |
| 1 unit per date | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 66% | 0 |
| 3 units per date | 0 / 0 | 97.2 / 99.2 | 98.1 / 97.7 | 729 | 65% | 0 |
| no expansion | 0 / 0 | 97.9 / 99.2 | 98.1 / 96.1 | 745 | 65% | 0 |
| expansion 50% | 0 / 0 | 96.5 / 99.2 | 98.1 / 97.7 | 729 | 66% | 0 |
| ordinal handles | 0 / 0 | 97.2 / 99.2 | 98.8 / 97.7 | 686 | 65% | 0 |
| BM25 at 25% | 0 / 0 | 97.2 / 98.5 | 98.1 / 96.9 | 748 | 65% | 0 |
| BM25 at 50% | 0 / 0 | 97.2 / 99.2 | 97.5 / 96.9 | 729 | 66% | 0 |

## Recall per field, against the digest

Stage 4's configuration, context / digest (the widened one's are under
[the chosen configuration](#what-it-costs)):

| Corpus | Half | Documents | Type | Date | Parties | Facts | Subject | `digest_recall` 1.0 | Violations |
|---|---|---|---|---|---|---|---|---|---|
| InternBench | tune | 40 | 100.0 / 100.0 | 100.0 / 100.0 | 97.5 / 97.5 | 99.2 / 99.2 | 98.1 / 99.4 | 39 | 0 |
| InternBench | holdout | 32 | 100.0 / 100.0 | 100.0 / 100.0 | 100.0 / 100.0 | 99.0 / 97.9 | 97.7 / 100.0 | 32 | 0 |
| InternBench | all | 72 | 100.0 / 100.0 | 100.0 / 100.0 | 98.6 / 98.6 | 99.1 / 98.6 | 97.9 / 99.7 | 71 | 0 |
| fixtures | tune | 7 | 100.0 / 100.0 | 85.7 / 85.7 | 100.0 / 100.0 | 85.7 / 85.7 | – / – | 6 | 0 |
| fixtures | holdout | 12 | 100.0 / 100.0 | 72.7 / 72.7 | 80.0 / 80.0 | 100.0 / 100.0 | – / – | 8 | 0 |
| fixtures | all | 19 | 100.0 / 100.0 | 77.8 / 77.8 | 87.5 / 87.5 | 94.4 / 94.4 | – / – | 14 | 0 |

On every document whose digest carried the date and every party
(`digest_recall` 1.0: 71 of the 72 InternBench documents and 14 of the 19
fixtures), the context carries the type, the date and every party.
Context recall below 1.0 elsewhere is extraction's, and the digest's is
the same: the corrupted text layer of `ocr-corrupted-invoice`, the
fixtures whose scans read "Ledar Finch" and "Pine Echo Courters", and two
fixture images whose OCR lost the date.

Fact recall is at least the digest's: on InternBench 99.1% against 98.6%
(tuning 99.2 / 99.2, held out 99.0 / 97.9), on the fixtures 94.4% both.
On the recordings without layouts too: `bench/recording.json` tuning 97.7
/ 96.5 and held out 94.9 / 93.5; the pre-router run 98.3 / 97.5 and
95.3 / 94.3, the digest second in each pair.

Where the context carries less of something than the digest:

| Document | Pages | Context lacks | Digest lacks | Why |
| --- | --- | --- | --- | --- |
| `inspection-log-ruled-2p` | 2 | fact `FE-032` | - | one of 44 extinguisher ids, in a table row and a long summary line; the identifier budget goes to the letterhead's license number and the first row |
| `services-agreement` | 6 | subject "service levels" | - | the "Service Levels" definition and Exhibit A's heading; the subject takes the recitals and the "Services" definition first |
| `data-processing-agreement-10p` | 10 | subject "GDPR", "student data" | fact "forty-eight hours" (both) | in the definitions and Annex I; the subject takes the Services definition and the processing description rows |
| `asset-purchase-agreement-25p` | 25 | subject "earn-out" | fact "$14,250,000.00" | the earn-out schedule's amounts now lose to the purchase price, which the digest missed |
| `credit-agreement-50p` | 50 | subject "term loan" | fact "June 12, 2031" | in the recital on page 4; the context carries the Maturity Date definition the digest lacked |

Each is a description item, not a filename item, and each is lexically in
the document: more subject budget recovers them (budgets x1.5: subject
99.4 / 98.4) at the cost of the long-document saving (50%, two documents
under 40%).

## Tokens against the digest

Estimated tokens (`engine::estimated_tokens`), Stage 4's configuration:

| Corpus | Pages | Docs | Context p50 | Digest p50 | Context mean | Digest mean | Context / digest (sum) | Smallest saving |
|---|---|---|---|---|---|---|---|---|
| InternBench | 1-3 | 62 | 777 | 888 | 850 | 1048 | 0.81 | -17% |
| InternBench | 4-9 | 4 | 1064 | 3821 | 1128 | 3376 | 0.33 | 8% |
| InternBench | 10-24 | 2 | 1120 | 3996 | 1120 | 3996 | 0.28 | 65% |
| InternBench | 25+ | 4 | 1912 | 4490 | 1797 | 4650 | 0.39 | 52% |
| fixtures | 1-3 | 16 | 86 | 78 | 178 | 202 | 0.88 | -34% |
| fixtures | 4-9 | 1 | 333 | 3605 | 333 | 3605 | 0.09 | 91% |
| fixtures | 10-24 | 1 | 1006 | 3771 | 1006 | 3771 | 0.27 | 73% |
| fixtures | 25+ | 1 | 46 | 37 | 46 | 37 | 1.24 | -24% |
| both | 1-3 | 78 | 679 | 784 | 712 | 875 | 0.81 | -34% |
| both | 4-9 | 5 | 928 | 3736 | 969 | 3422 | 0.28 | 8% |
| both | 10-24 | 3 | 1006 | 3977 | 1082 | 3921 | 0.28 | 65% |
| both | 25+ | 5 | 1908 | 4250 | 1447 | 3728 | 0.39 | -24% |

On documents of ten pages or more the context is 66% smaller than the
digest in total, and every one is at least 52% smaller except
`long-document-100-pages.pdf`, a hundred-page fixture with 37 tokens of
text, which goes whole and costs 46: the handles cost more than its text
saves. That is the general cost of stable handles on short documents:
`[p1.b3] ` is five to seven tokens per line, so 25 of the 75 documents
that go whole cost more than their digest (at most 34% more, all under
1,400 tokens). Ordinal handles (`[3]`, with `--- page N ---` lines) cut
that to 13 documents and 25%, and the median context from 729 to 686
tokens; the id style is Stage 7's decision.

The documents that go to retrieval: of the 91, 75 go whole, 7 are
retrieved at the normal tier and 9 at the dense tier (every long
InternBench document but the ten-page scan triggers it, by close
candidate dates or many organizations), 3 of them hierarchically.

## Flat or hierarchical

On the seven documents of ten pages or more, flat and hierarchical
retrieval lose exactly the same six description items and no filename
item; hierarchical sends 10,434 tokens to flat's 10,474. Forced on every
document, the two are level on facts and within one subject term. Field
budgets and per-field scoring already reach page 10 of the credit
agreement for its defined Closing Date, page 41 of the annual report for
the auditors' date, and the signature pages, without restricting the
search to sections first. The verdict: hierarchical retrieval is not
needed on this corpus. It is at least as good at fewer tokens, so it
stays on from 300 units, as the plan's rule asks; it can be removed
without loss if it ever costs anything.

## Is an embedding model justified?

No. The measured gap between the chosen context and the whole document
(retrieval with an unlimited whole-document threshold) is, over all 91
documents: no filename item; two description facts ("forty-eight hours",
which the digest also lacks, and `FE-032`); six subject terms on five
documents. Every one of them is a phrase printed in the document that
lexical and structural scoring finds but ranks below the budget, and
every one comes back with more subject or identifier budget. An embedding
model would have to know which clause a reviewer's subject comes from to
do better, and it would add a second model to load, memory, and an
embedding pass over every unit of a hundred-page document. Index and
retrieval together take about 4 ms at the median and 141 ms at the most (the
100-page annual report: 2,095 units, 135 ms to index, 6 ms to retrieve).
Revisit only if the larger long-document set the plan's Stage 1 adds shows
a filename item lost to wording that does not match.

## Acceptance (Stage 4)

| Gate | Result |
| --- | --- |
| Type, date and party recall 1.0 wherever `digest_recall` is 1.0 | met on every such document, in both corpora and both recordings without layouts |
| `context_fact_recall` at least the digest's | met: 99.1% / 98.6% InternBench, 94.4% / 94.4% fixtures, and on both halves |
| Context tokens at least 40% below the digest's on documents of ten pages or more | met: 66% in total, every document at least 52%, except the 37-token fixture above |
| Prompts and default behavior byte-identical | met: nothing in the engine calls the index or the retriever yet; the InternBench replay of `bench/recording.json` against `bench/baseline.json` and the fixture replay against `fixtures/corpus-baseline.json` report no change |
