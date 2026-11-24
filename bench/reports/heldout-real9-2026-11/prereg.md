# The ninth real set: what will be measured, written before it is read

Written on 2026-11-24, before any model has read a document of
`scripts/realdocs-eval/sample-heldout8.json` (SHA-256
`1a0ea119e18ea5dae30efca5b499ec19a82fe7308d85e97361e980177cfc998d`), and
committed with it, so the commit's time is the time this was fixed.

## The set

39 public documents (9 letters, 1 e-mail, 7 contracts, 5 notices, 4
memoranda, 3 forms, 3 sets of minutes, 3 invoices, 2 purchase orders, 1
statement, 1 report) from 19 hosts, none of whose URLs, hosts or files an
earlier sample used. Gold was written from each document alone, read again
blind by three second readers (each a third of the set), and adjudicated
against the document's own text (118 points on 29 documents). 25 documents
expect to be ready, 10 either and 4 to go to review, so readiness is judged on
29. It was gathered after every rule alpha.21 carries; no rule, threshold or
prompt has been written from it. No document in it was generated.

## What it is for

It judges a release candidate on documents nothing in that candidate was
fitted to. It does not choose anything: no threshold, rule, flag or gate is
picked by its results.

## Sealed until a run is preregistered

Nothing is run on this set until a short addendum to this file, committed
before the run, names the one candidate it judges (a release candidate's
pull request head, or a draft change such as pull
request 93), the engine it is compared with, and the gate. Until then:

- No model reads it, no engine replays it, and no change, draft or
  otherwise, is scored on it.
- The briefs written for whoever builds or reviews rules say nothing of its
  documents, and they are told not to open it.
- After its first run, the coordinator of that round has read the results and
  the set is no longer blind to them; a later round must not describe or
  quote its documents or results in any brief, and a rule from a later round
  that changes one of its readings is reported with that caveat, by
  document, as the eighth set's report does.

## The run, once preregistered

`candidate-bench.yml` with `sets: real9` on the named candidate, the model as
its manifest pins it, `--no-repack` as the app launches it, four threads,
token traces kept, and the recording replayed through the comparison engine
on the same replies at the named gate, so a difference between the two is
the engine's.

## What will be reported, whatever it shows

For both engines: documents fetched, read and scored; names right; names
filed without review at the gate, right and wrong; and the type, date and
party rates. A wrong name filed without review is listed by document with its
least token probability. Results are also given with the FARA filings
(real9-contract-01 to -05, real9-form-01 to -03) and without them, and with
the seven www.ed.gov documents and without them, because those two sources
are 15 of the 39.

## If the candidate does worse

If the candidate files a wrong name without review where the comparison
engine, given the same reply, would have held it for review, the cause is
found before the candidate ships. A change written from that finding is
reported as fitted to this set, and the set no longer counts as held out for
it.

## What it cannot show

- 39 documents cannot certify a rate. One wrong filing is 2.6 points on the
  whole set and 3.4 on the 29 that judge readiness; a few filings either way
  is within what two runners' replies differ by
  (`speed-linux-cpu-2026-11-11`).
- It is concentrated: 8 FARA filings and 7 Office for Civil Rights
  documents, two families from two sites, and 4 documents from one town. A
  rule that helps or hurts one family moves the totals by several documents.
- Invoices are thin (3, one of them a council measure bundle), and there is
  one statement and one report.
- The FARA gold accepts both the filing and the attached agreement, and the
  scorer accepts any acceptable type with any acceptable date and party set,
  so a mixed name scores right there. The set is lenient on those five.
- The second readers were AI readers of the same family as the first, not
  people; agreement between them is weaker evidence than a human second
  read.
- www.ed.gov refuses fetch.py (HTTP 403 from its Azure Front Door), so its
  seven documents must be saved by hand into the corpus before a run; a run
  without them scores 32. www.hingham-ma.gov reset one connection while the
  pins were checked and served on retry.
