# The eighth real set: what will be measured, written before it is read

Written on 2026-11-15, before any model has read a document of
`scripts/realdocs-eval/sample-heldout7.json` (SHA-256
`4e564f4909759f57045929ec00e49e9ac071bfda934a474b9cdf8fe3d822f5e5`), and
committed with it, so the commit's time is the time this was fixed.

## The set

55 public documents (18 letters, 12 notices, 6 invoices, 5 forms, 4
memoranda, 3 contracts, 3 sets of minutes, 2 statements, 1 purchase order, 1
report) from 45 hosts in seven countries and bodies of the European Union,
none of whose URLs, hosts or files an earlier sample used. Gold was written
from each document alone, read again blind by a second reader, and
adjudicated against the document's own text (117 points raised on 64
documents; one document, real8-contract-03, came in at adjudication and had
no blind second reading). 22 documents expect to be ready, 32 either, 1 to go
to review, so readiness is judged on 23. It was gathered after every rule
alpha.18 carries; no rule, threshold or prompt has been written from it.

## What it is for

It judges alpha.19 on documents nothing in alpha.19 was fitted to. It does
not choose anything: no threshold, rule or flag is picked by its results.

## What the seventh set taught, and what is done differently

The seventh set's report had to disclose that the coordinator of a later
round had read its results, and that the brief written for that round's
analysts described the class of its one wrong filed name and quoted two of
its readings. For this set:

- It is run once, on the alpha.19 release candidate, after every rule of
  alpha.19 is final and its verifiers have ruled. No rule, threshold or
  prompt changes for alpha.19 after the run, whatever it shows; the one
  exception is the clause "If alpha.19 does worse" below.
- The people and processes that build and review alpha.19's rules are told
  not to open this set, and the briefs written for them say nothing of its
  documents: they were written before the set was assembled.
- After the run the round's coordinator will have read the results. From
  then on the set is not blind to the coordinator. A later round must not
  describe or quote this set, its documents or its results in any brief; a
  rule from a later round that changes a reading of this set is reported
  with that caveat, by document, as the seventh set's report does.

## The runs

`candidate-bench.yml` with `sets: real8` on the alpha.19 release candidate
(the pull request's head), Qwen3.5 9B Q4_K_M as the model manifest pins it,
`--no-repack` as the app launches it, four threads, token traces kept.

The recording is replayed through both engines, alpha.19's and alpha.18's
(`v0.1.0-alpha.18`), on the same replies at the gate of 0.8, so a difference
between the two releases is the engine's. A document whose extracted text
alpha.18's worker would not give (a replay of it is stale) is counted for
alpha.19 alone, and the report says how many.

## What will be reported, whatever it shows

In the alpha.19 release notes and in this directory, for both engines:
documents fetched, read and scored; names right; names filed without review
at the gate of 0.8, right and wrong; and the type, date and party rates. A
wrong name filed without review is listed by document with its least token
probability.

## If alpha.19 does worse

If the alpha.19 candidate files a wrong name without review where
alpha.18's engine, given the same reply, would have held it for review, the
cause is found before the release ships. A change written from that finding
is reported as fitted to this set, and the set no longer counts as held out
for it.

## What it cannot show

55 documents cannot certify a rate; a handful of filings either way is
within what two runners' replies differ by
(`speed-linux-cpu-2026-11-11`). alpha.19's engine is planned to differ from
alpha.18's by at most one rule, which holds a date a spreadsheet formula
computes; the set has one spreadsheet, so it says little about that rule
either way. What it measures is where the engine stands on documents it has
never seen.
