# InternBench reports of record

Each pair is one report: the Markdown is for reading, the JSON for
`intern-bench compare`.

| Report | What it is |
| --- | --- |
| `2026-10-07-baseline` | The baseline: Intern as InternBench found it, before any work it was built to measure. Scores replayed from `bench/recording.json` against today's gold; timings and memory are the recording's. |
| `2026-10-07-phase2-before` | All 72 documents through the reader as it was before document routing and PP-OCR (Tesseract, page text as the PDF gives it), recorded live and replayed against the current gold and scorer. |
| `2026-10-07-phase2-after` | The same 72 documents through the routing reader with PP-OCR, replayed from today's `bench/recording.json`, the recording of record since. |
| `2026-10-07-phase2-compare` | `intern-bench compare` of the two: what document routing, layout and PP-OCR changed, document by document. |
| `2026-10-07-phase3-before` | All 77 documents through the digest pipeline, as Intern read them before phase 3: the phase 2 recording of record plus the five long documents, replayed from `bench/recording-digest.json` with `--pipeline digest`. |
| `2026-10-07-phase3-after` | The same 77 documents through the evidence pipeline, the default since, replayed from today's `bench/recording.json`. |
| `2026-10-07-phase3-compare` | `intern-bench compare` of the two, with the phase 3 scorecard: what retrieval, the facts reply and validation changed, document by document. |

## The phase 3 runs

"Before" is the digest pipeline's recording: the phase 2 recording of
record (72 documents, recorded live at `bd48e98`, eight scans again at
`ad6e661`) and the five long documents added for retrieval, recorded live
at `22520ea`, committed as `bench/recording-digest.json` (written
compacted; its documents and header equal the uncompacted copy the reports
first cited, sha256 `5efea19a`). "After" is the evidence pipeline's
recording of record, all
77 documents recorded live at `902c7fe` with the stable ids, the compact
reply and its instructions in the system turn. Every recording was made on
the same otherwise idle 4-core Xeon, the pinned model at 4 threads with an
8,192-token context, with the routing reader and PP-OCR. Both were replayed
at `50bc017` with the same scorer and gold, and again at `f7ee8de`, after
the reviewed fixes, to carry the context measurements: every score is
unchanged.

The phase 3 scorecard, from the two reports (replayed scores; timings and
tokens as recorded on that machine, all but `index_ms` and `retrieval_ms`,
which each replay measures; `compare` compares no timings between
replays):

| Figure | Digest (before) | Evidence (after) |
| --- | ---: | ---: |
| Filename, all 77 | 26/77 (33.8%) | 54/77 (70.1%) |
| Long-document filename accuracy (10+ pages, 12) | 66.7% | 75.0% |
| Complex-document filename accuracy (56) | 33.9% | 73.2% |
| Description completeness | 53.5% | 53.7% |
| Unsupported-fact rate (documents with an `*_UNSUPPORTED` reason) | 27.3% | 11.7% |
| Unsupported description claims | 1 of 264 | 0 of 290 |
| Review rate | 32.9% | 15.6% |
| Unsafe ready / trap dates | 30 / 11 | 15 / 1 |
| Evidence recall | 51.7% | 66.0% |
| Total latency p50 / p95 | 38.94 s / 96.03 s | 19.21 s / 33.88 s |
| Generation latency p50 / p95 | 10.69 s / 16.18 s | 6.53 s / 8.82 s |
| Generated tokens p50 / p95 | 141 / 204 | 87 / 116 |
| Prompt tokens p50 | 2,144 | 914 |

Description completeness is the one figure that barely moved, and its
margin is inside live drift: the same code scored 50.9% and 55.1% on the
72 documents from two live recordings whose prompts differed only in one
instruction line, so read it as about ±2 points of drift between live
runs, not as a measured gain. The digest pipeline's run failed one
document (`watershed-monitoring-report-100p`, a reply cut off at the token
cap), counted as a miss. `docs/pipeline-bottlenecks.md` (Phase 3:
measured) reads these reports.

Both reports also carry what the evidence context holds and costs. In the
evidence pipeline's report it is the context its prompt carried at full
retrieval scale: `context_tokens` p50 876 and p95 1,915, `context_units`
p50 25 and p95 59, from an index of `index_units` p50 28 and p95 1,339.
Indexing took p50 4.2 ms and retrieval 0.05 ms on the replaying machine.
In the digest pipeline's report the same keys measure the context
retrieval would build from the same text, not anything the digest pipeline
sent, so they are not a before and after: document by document the two
reports' values are nearly equal by construction. Its distributions leave
out the one document its run failed, the 100-page report, so its p95 and
maximum are lower.

## The phase 2 runs

Both were recorded live one after the other on the same otherwise idle
4-core Xeon, with the pinned model at 4 threads and an 8,192-token context.
"Before" used a reader built from the code before routing, with the
Tesseract runtime; "after" the routing reader with the PP-OCR runtime. Eight
scans were recorded again with the reader as it ships, after a layout fix that
stopped OCR pages from pairing headings, names and addresses as labels and a
change that keeps OCR's memory arena while a document is read, and merged
into the "after" recording. The other 64 documents' page text is
byte-identical under both changes.
Both reports were then replayed with the same scorer and gold, so every score
is computed the same way. `docs/pipeline-bottlenecks.md` (Phase 2: measured)
reads them.

## The baseline run

The live run behind it was made on 2026-10-07:
* all 52 documents;
* the pinned model through llama.cpp at 4 threads with an 8,192-token
  context, the app's flags otherwise;
* an otherwise idle 4-core Xeon with 16 GB.

One document (`scan-upside-down-po`) was recorded again after the generator
stopped printing a real company's name on it, and merged into the recording
with `intern-bench merge-recordings`. The container had meanwhile moved to a
host with a slower clock, so that one document's timings come from a
different machine than the rest.

`docs/pipeline-bottlenecks.md` reads this report: where the time goes, where
the facts are lost, and which changes are worth most.

To see what a change does, run it the same way and compare:

```sh
intern-bench compare --before bench/reports/2026-10-07-baseline.json --after report.json --markdown diff.md
```

Compare timings only between live runs made on the same machine at the same
thread count.
