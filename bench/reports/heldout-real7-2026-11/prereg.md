# The seventh real set: what will be measured, written before it is read

Written on 2026-11-12, before any model has read a document of
`scripts/realdocs-eval/sample-heldout6.json` (SHA-256
`83c1879e77f96512fae635acc031230b5a0c1f6dda5b82d0d57779d32c9dd13c`), and
committed with it, so the commit's time is the time this was fixed.

## The set

55 public documents (15 letters, 9 notices, 7 invoices, 6 forms, 6
memoranda, 5 minutes, 2 statements, 2 contracts, 2 receipts, 1 purchase
order) from 42 hosts in ten countries, none of whose URLs, hosts or files an
earlier sample used. Gold was written from each document alone, read again
by three independent readers, and adjudicated against the document's own
text (125 points of disagreement). 34 documents expect to be ready, 18
either, 3 to go to review. It was gathered after every rule alpha.17 carries
and after the review gate was chosen on the sixth set; no rule, threshold or
prompt has been written from it.

## What it is for

It judges alpha.18 on documents nothing in alpha.18 was fitted to. It does
not choose anything: no threshold, rule or flag is picked by its results.

## The runs

`candidate-bench.yml` with `sets: real7` on the alpha.18 release candidate
(the pull request's head), Qwen3.5 9B Q4_K_M as the model manifest pins
it, `--no-repack` as the app launches it, four threads, token traces kept.

The recording is replayed through both engines, alpha.18's and alpha.17's
(`v0.1.0-alpha.17`), on the same replies, so a difference between the two
releases is the engine's. A document whose extracted text alpha.17's worker
would not give (a replay of it is stale) is counted for alpha.18 alone, and
the report says how many.

## What will be reported, whatever it shows

In the alpha.18 release notes and in this directory, for both engines:
documents fetched, read and scored; names right; names filed without review
at the gate of 0.8, right and wrong; and the type, date and party rates. A
wrong name filed without review is listed by document with its least token
probability.

## If alpha.18 does worse

If the alpha.18 candidate files a wrong name without review where
alpha.17's engine, given the same reply, would have held it for review,
the cause is found before the release ships. A change written from that
finding is reported as fitted to this set, and the set no longer counts as
held out for it.

## What it cannot show

55 documents cannot certify a rate; a handful of filings either way is
within what two runners' replies differ by (`speed-linux-cpu-2026-11-11`).
