# The seventh real set, held out, November 2026

What [`prereg.md`](prereg.md) fixed before any model read the set, measured.
No rule, threshold, prompt or flag in alpha.18 was written from these
documents, and nothing was chosen by these results.

## The run

`candidate-bench.yml` run 43302007759, job "a18-9b on real7": the alpha.18
release candidate at `343ee39` (the pull request's head after the four
alpha.18 packages were merged), Qwen3.5 9B Q4_K_M as the model manifest pins
it (`03b74727...`), `--no-repack` as the app launches it, four threads, token
traces kept, on an AMD EPYC 9V45 runner; 1,597 s for the set.

54 of the 55 documents were fetched and read. `real7-invoice-05` was not
fetched: its source no longer matches the size and SHA-256 the sample pins.
51 of the 54 have a scored filename (gold sends three to review with no
name).

The recording was replayed through both engines on the same replies, at the
gate of 0.8: alpha.18's (`0c5f3e9`) and alpha.17's (`v0.1.0-alpha.17`,
`e47a756`). No replay was stale. The documents' text is alpha.18's worker's
in both: the replay reads the recording's own extracted text, so a document
alpha.17's worker would have read differently is not told apart by
staleness; alpha.18's changes to extraction (the re-read of somebody else's
OCR layer, a blank page, the orientation stop) are the only ways they could
differ.

## Results

Both engines read every document of the set identically: the same name,
readiness and reasons on all 54.

| | alpha.18 | alpha.17 |
| --- | ---: | ---: |
| Documents fetched / read / scored | 54 / 54 / 51 | 54 / 54 / 51 |
| Names right | 19 of 51 (37%) | 19 of 51 |
| Type right | 46 of 54 (85%) | 46 of 54 |
| Date right | 47 of 51 (92%) | 47 of 51 |
| Parties right | 33 of 54 (61%) | 33 of 54 |
| Filed without review at 0.8, right / wrong | 3 / 1 | 3 / 1 |
| Held for review | 50 of 54 (93%) | 50 of 54 |

Names right by kind (alpha.18): letters 5 of 15, notices 4 of 9, invoices 2
of 6, forms 2 of 6, minutes 2 of 5, memoranda 1 of 6, contracts 1 of 2,
receipts 1 of 2, statements 1 of 2, purchase orders 0 of 1.

### The names filed without review

| Document | Least token probability | Name | Right |
| --- | ---: | --- | --- |
| real7-minutes-05 | 0.994 | 2026-08-20 Board Meeting Minutes from Cedar Valley Water Conservancy | yes |
| real7-letter-07 | 0.933 | 2024-03-19 Review of Large Energy Users Connection Policy for Uisce Éireann | **no** |
| real7-notice-08 | 0.885 | 2026-08-12 Notice Of Conclusion of Audit from Noak Bridge Parish Council | yes |
| real7-contract-01 | 0.879 | 2025-10-01 Lease of City Real Property between City of San Leandro and As Kneaded Bakery | yes |

`real7-letter-07` is Uisce Éireann's response to a consultation of the
Commission for Regulation of Utilities. The model took the consultation's
title for the document's type and called the respondent a "client"; the
engine named it "for" the respondent as a record's counterpart. The gold
names it a response or letter from Uisce Éireann, or to the Commission.
alpha.17's engine files the same name from the same reply, so the
pre-registration's condition (alpha.18 filing a wrong name alpha.17 would
hold) is not met. Nothing was changed for it: a change written from it would
be fitted to this set.

### The review gate, out of sample

The gate of 0.8 was chosen on the sixth set (`bench/reports/review-gate-2026-11`).
On this one, at the 0.7 alpha.16 used, 8 names would have been filed
without review, 5 right and 3 wrong (`real7-letter-13` at 0.750 and
`real7-minutes-04` at 0.754 besides `real7-letter-07`); at 0.8, 3 right and
1 wrong. The gate held two of the three wrong names and two of the five
right ones.

## What this does not show

- 51 scored documents cannot certify a rate; one or two filings either way is
  within what two runners' replies differ by.
- alpha.18 and alpha.17 differ here only through the engine, which reads this
  set the same; alpha.18's other changes (the quicker model's launch,
  startup, review choices, re-validation, the OCR layer re-read) are not
  measured by it. 15 of the 54 were read by OCR, wholly or in part; the bench
  records each page's route but not why it went to OCR, so whether any of
  them took the re-read of somebody else's OCR layer is not known from this
  run.
- The names right (37%) are lower than on the sixth set under alpha.17 (19 of
  45, 42%): different documents, not a change in the engine, which reads both
  sets identically to alpha.17.

## Reproducing

Download the run's `candidate-a18-9b-real7-43302007759` artifact, write the
gold with `scripts/realdocs-eval/fetch.py OUT sample-heldout6.json` (or from
the sample's own entries for the 54 documents read), and replay:

    intern-bench run --corpus <empty dir> --gold gold.json \
      --replay recording.json --min-token-confidence 0.8 --allow-stale \
      --output report.json
