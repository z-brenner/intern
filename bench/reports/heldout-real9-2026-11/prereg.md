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

## Addendum 1: the strict_sides review level (written 2026-11-24, before the run)

Written before any model has read a document of the set, and committed
before the run starts, so the commit's time is the time this was fixed. The
coordinator chose the candidate under Zack's go-ahead ("Once it's done, get
going"); the other candidate considered, Clef-Flash, failed its own screen
(draft PR 95) and is not run here. This is the set's only planned use; any
later run needs its own addendum and is reported as made with this run's
results in view.

**Candidate.** The `strict_sides` review level of draft PR 93 (head
`27a97a19983cc109fa80cefb212d1dd134f048cc`): strict, except that
`PARTY_ROLE_UNCONFIRMED` alone does not hold a name.

**Baseline.** `strict`, today's level, on the same replies.

**The run.** One live run of the shipped accurate model (Qwen3.5 9B
Q4_K_M, SHA-256 `03b74727…52b7e8` as `model-manifest.json` pins it) over all
39 documents, the seven www.ed.gov files saved by hand into the corpus with
their pinned SHA-256, on a 4-processor Linux machine with the pinned Linux
runtime, `--no-repack`, four threads, token traces kept, built from PR 93's
head, recorded at the shipped gate of 0.7 and strict. The recording is then
replayed once through the same build with `--review-level strict_sides` at
0.7. The replies are the same for both levels, so any difference is the
level's. The run is made in a cloud container, not on a GitHub runner,
because the runner cannot fetch the seven www.ed.gov documents and the set's
documents are not committed.

**Scoring.** Each filed name (ready, not held) is classed right, minor or
really wrong by `bench/reports/review-levels-2026-11/policy/severity.py`,
as that report defines the classes, and every filed name that is not
exactly right is then read by hand against the gold. The number used is the
larger of the script's count and the hand count.

**Pass rule.** The bar is the one the review-levels report fixed for a
looser level: no more than 10% of the names it files really wrong.
`strict_sides` passes on this set if:

1. no more than 10% of all the names it files are really wrong, and
2. no more than 1 of the names it files that `strict` holds is really
   wrong.

Rule 2 exists because the newly filed names may be very few, and a share
over three or four names means little; one really wrong name among them is
tolerated, two are not. If `strict_sides` files nothing that `strict` holds,
the set says nothing for or against it and that is the result.

A pass does not make `strict_sides` the default or merge PR 93; that is
Zack's decision. A fail means the level is withdrawn as it stands, not tuned
on this set.

**Reported whatever it shows.** For both levels: documents read; names
filed; filed right / minor / really wrong by script and by hand; the names
`strict_sides` files that `strict` holds, listed by document with each
one's reasons and least token probability; the same with the seven
www.ed.gov documents left out and with the eight FARA documents left out;
and seconds per document.

**What 39 documents can show.** Prior measurement (PR 93, sets 1 to 8, not
held out) found 19 extra filings in 459 documents, so this set may give
about 1 or 2. One really wrong name among a handful is consistent with
anything from 0% to well over 25%. The set can catch a level that files
several really wrong names; it cannot certify a rate, and a pass here is
weak evidence added to PR 93's own measurement, not a replacement for it.
