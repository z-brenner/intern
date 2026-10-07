# InternBench reports of record

Each pair is one report: the Markdown is for reading, the JSON for
`intern-bench compare`.

| Report | What it is |
| --- | --- |
| `2026-10-07-baseline` | The baseline: Intern as InternBench found it, before any work it was built to measure. Scores replayed from `bench/recording.json` against today's gold; timings and memory are the recording's. |

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
