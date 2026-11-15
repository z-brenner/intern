# The eighth real set, held out, November 2026

What [`prereg.md`](prereg.md) fixed before any model read the set, measured.
No rule, threshold, prompt or flag in alpha.19 was written from these
documents, nothing was chosen by these results, and nothing in alpha.19's
engine changed after the run.

## The run

`candidate-bench.yml` run 43666145989, job "a19-9b on real8": the alpha.19
release candidate at `6e8061f`, the pull request's head once its one engine
rule was final and its verifier had ruled. Qwen3.5 9B Q4_K_M as the model
manifest pins it (`03b74727...`), `--no-repack` as the app launches it, four
threads, token traces kept, on an Intel Xeon Platinum 8573C runner; 2,793 s
for the set. It was run once.

What landed in the pull request after `6e8061f` is the review window's
(TypeScript) and documents; no file under `crates/` or `src-tauri/` differs
between that commit and the release, so the engine and the worker that read
this set are the ones that ship.

All 55 documents were fetched and read. 54 have a scored filename (gold sends
one, real8-notice-11, to review with no name).

The recording was replayed through both engines on the same replies, at the
gate of 0.8: alpha.19's (`6e8061f`) and alpha.18's (`v0.1.0-alpha.18`,
`15b5ec9`). No replay was stale.

## Results

Both engines read every document of the set identically: the same name,
readiness and reasons on all 55.

| | alpha.19 | alpha.18 |
| --- | ---: | ---: |
| Documents fetched / read / scored | 55 / 55 / 54 | 55 / 55 / 54 |
| Names right | 16 of 54 (30%) | 16 of 54 |
| Type right | 43 of 55 (78%) | 43 of 55 |
| Date right | 41 of 54 (76%) | 41 of 54 |
| Parties right | 40 of 55 (73%) | 40 of 55 |
| Joining word right | 34 of 53 | 34 of 53 |
| Filed without review at 0.8, right / wrong | 3 / 1 | 3 / 1 |
| Held for review | 51 of 55 (93%) | 51 of 55 |

Names right by kind: notices 6 of 11, letters 3 of 18, invoices 2 of 6, forms
2 of 5, memoranda 1 of 4, minutes 1 of 3, reports 1 of 1, contracts 0 of 3,
statements 0 of 2, purchase orders 0 of 1.

13 of the 16 right names waited for review. Of the 22 documents the
reference expects to be ready, 1 was filed and 8 more were named right and
held.

### The names filed without review

| Document | Least token probability | Name | Right |
| --- | ---: | --- | --- |
| real8-form-01 | 0.936 | 2018-03-31 Off Application for Marine and General Mutual Life Assurance Society | **no** |
| real8-form-02 | 0.935 | 2026-01-12 Confirmation Statement from The Brass Band Summer School Limited | yes |
| real8-invoice-03 | 0.911 | 2023-09-30 Tax Invoice from Kaipara Refuse Limited | yes |
| real8-form-05 | 0.855 | 2021-12-17 Certificate of Registration for CGI 1100 OPO MANAGEMENT, LP | yes |

`real8-form-01` is a Companies House form DS01, "Striking off application by
a company". The model gave that title with the registry's name run on
("Striking off application by a company Companies House"); the engine cut it
to "Off Application", every check passed, and the name was filed. Its date
and its party are among those the reference accepts; its type is wrong.
alpha.18's engine files the same name from the same reply, so the
pre-registration's condition (alpha.19 filing a wrong name alpha.18 would
hold) is not met. Nothing was changed for it: a change written from it would
be fitted to this set. It is a fault in how the engine shortens a type, and a
later rule that repairs it must be reported as fitted to this document.

### The review gate, out of sample

The gate of 0.8 was chosen on the sixth set. On this one, at the 0.7
alpha.16 used, 6 names would have been filed without review, 4 right and 2
wrong (`real8-purchase_order-01` at 0.792, named "Order Form for Partner
Press", besides `real8-form-01`; the right one the stricter gate holds is
`real8-notice-08` at 0.718). At 0.8: 3 right and 1 wrong. The gate held one
of the two wrong names and one of the four right ones.

## What this does not show

- 54 scored documents cannot certify a rate; one or two filings either way is
  within what two runners' replies differ by.
- alpha.19's engine differs from alpha.18's by one rule, which holds a date
  that a spreadsheet's TODAY or NOW formula works out. The set's one
  spreadsheet (real8-statement-01) carries no such date, so the set says
  nothing about that rule; it reads the same under both engines, as every
  other document does.
- alpha.19's other changes (the review window) are not measured by it.
- 24 of the 55 were read by OCR, wholly or in part; the bench records each
  page's route but not why it went to OCR.
- The names right (30%) are fewer than on the seventh set under alpha.18's
  engine (19 of 51, 37%): different documents, not a change in the engine.

## After this run

The round's coordinator has now read these results. As the pre-registration
says, from here on the set is not blind to the coordinator: a later round
must not describe or quote this set, its documents or its results in any
brief, and a rule from a later round that changes a reading of this set is
reported with that caveat, by document.

## Reproducing

Download the run's `candidate-a19-9b-real8-43666145989` artifact, write the
gold with `scripts/realdocs-eval/fetch.py OUT sample-heldout7.json` (or from
the sample's own entries), and replay:

    intern-bench run --corpus <empty dir> --gold gold.json \
      --replay recording.json --min-token-confidence 0.8 --allow-stale \
      --output report.json
