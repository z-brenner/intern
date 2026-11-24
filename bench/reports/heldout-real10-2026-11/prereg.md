# The tenth real set: what will be measured, written before it is read

Written on 2026-11-24, before any model has read a document of
`scripts/realdocs-eval/sample-heldout9.json` (SHA-256
`5bcf303451b0d4405e4ae22400b456a90514fe13c2af9543fa8ad6d523221214`), and
committed with it, so the commit's time is the time this was fixed.

## The set

50 public documents (8 letters, 1 e-mail, 6 notices, 5 contracts, 3 orders,
5 sets of minutes, 4 memoranda, 5 invoices, 2 purchase orders, 5 forms, 4
statements, 2 reports) from 48 hosts in six countries, none of whose URLs,
hosts or files an earlier sample used. No host gives more than 2 documents.
Gold was written from each document alone, read again blind by three second
readers (each a third of the set), and adjudicated against the document
(`sample-heldout9.json`'s notice says what changed). 28 documents expect to
be ready, 18 either and 4 to go to review, so readiness is judged on 32. No
rule, threshold or prompt has been written from it. No document in it was
generated.

## What it is for

It is sealed for one candidate: the whole-value confidence score being built
as a draft change for over-flagging of good results, which no earlier
held-out set can judge blind because each has been read by the round that
built it or by a run. It judges that candidate on documents nothing in it was
fitted to. It does not choose anything: no threshold, rule, flag or gate is
picked by its results.

## Sealed until a run is preregistered

Nothing is run on this set until a short addendum to this file, committed
before the run, names the candidate (its pull request head), the engine it is
compared with, the gate and the threshold, all fixed on other data. That
addendum is written when the candidate is ready. Until then:

- No model reads it, no engine replays it, and no change, draft or
  otherwise, is scored on it.
- The briefs written for whoever builds or tunes the candidate say nothing of
  its documents, and they are told not to open it.
- After its first run the set is no longer blind to that round's results; a
  later round must not describe or quote its documents or results in any
  brief, and a later rule that changes one of its readings is reported with
  that caveat, by document.

## The run, once preregistered

`candidate-bench.yml` with `sets: real10` on the named candidate, the model
as its manifest pins it, `--no-repack` as the app launches it, four threads,
token traces kept, and the recording replayed through the comparison engine
on the same replies at the named gate, so a difference between the two is
the engine's.

## What will be reported, whatever it shows

For both engines: documents fetched, read and scored; names right; names
filed without review at the gate, right, minor and really wrong on the
severity scale of `review-levels-2026-11`; names held for review, by the
same scale; and the type, date and party rates. A wrong name filed without
review is listed by document with its score. The bar of at most 10% of
filed names really wrong is reported as it stands, with whatever further
rule the addendum names.

## If the candidate does worse

If the candidate files a really wrong name the comparison engine would have
held, the cause is found before the candidate ships. A change written from
that finding is reported as fitted to this set, and the set no longer counts
as held out for it.

## What it cannot show

- 50 documents cannot certify a rate. One wrong filing is 2 points on the
  whole set and about 3 on the 32 that judge readiness.
- Gold is lenient where documents carry several defensible dates
  (contract-02, contract-05, invoice-05 and the period-end statements); the
  scorer takes any acceptable date, so a reading of one of those is right
  whichever it picks.
- Three documents name their issuer only in an image (invoice-01, memo-04,
  statement-02) and one only in form fields absent from its text layer
  (form-02); how the app reads them depends on its text extraction, which
  this set does not vary.
- The second readers were AI readers of the same family as the first, not
  people; agreement between them is weaker evidence than a human second
  read.
- dpw.lacounty.gov (notice-01) refuses about half of requests with HTTP 403,
  so a run may need fetch.py run again or that file saved by hand; a run
  without it scores 49.
