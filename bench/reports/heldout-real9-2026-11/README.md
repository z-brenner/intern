# The ninth real set, run once: strict_sides fails its preregistered bar

Run 2026-11-24, 04:05 to 04:40 UTC, under [`prereg.md`](prereg.md), Addendum 1, committed at 04:01
(commit 87d8779f) before the model read any document.

## Result

| Level | Filed without a person | right / minor / really wrong | Really wrong share |
|---|---:|---|---:|
| strict (baseline) | 5 of 39 | 3 / 2 / 0 | 0% |
| strict_sides (PR 93) | 7 of 39 | 4 / 2 / 1 | **14.3%** |

**Pass rule 1** (no more than 10% of the names it files really wrong): **fails**, 1 of 7.
**Pass rule 2** (no more than 1 really wrong among the names it files that strict holds): passes, 1 of 2.
Under the prereg, strict_sides is withdrawn as it stands and is not tuned on this set. Nothing
was merged and no default changed.

The names strict_sides files that strict holds, each held at strict by `PARTY_ROLE_UNCONFIRMED`
alone:

| Document | Name filed | Least token probability | Class |
|---|---|---:|---|
| real9-contract-01 (FARA Exhibit B) | 2025-12-26 Exhibit B to Registration Statement for Ministry of Foreign Affairs of the Government of Japan | 0.861 | right |
| real9-notice-05 (San Diego Prop 218 notice) | 2026-10-06 Notice of Public Hearing from City of San Diego | 0.904 | **really wrong** |

real9-notice-05 prints no date of issue; 2026-10-06 is the hearing date and protest deadline, a
date the gold forbids, and the gold expects the document to go to review. At strict the side was
the only thing holding it: the engine took the hearing date as the notice's own date with no date
reason raised. The fault is the date, not the side; strict_sides removed the one hold that happened
to catch it. This is the same shape as PR 93's one really wrong development name (a trap date on
an xlsx memo).

Filed names that are not exactly right, read by hand (script and hand agree on all three):

- real9-minutes-03, `2025-06-12 Minutes of the from Magna Water District`: minor (type cut short; date
  and body right). Filed at both levels.
- real9-purchase_order-01, `2025-10-20 Purchase Order Change Notice for Texas State Board of Pharmacy`:
  minor (the revision date and the change-notice type are both accepted; the joining word is "for"
  where the gold has "from"). Filed at both levels.
- real9-notice-05: really wrong, above.

## Cuts the prereg asked for

| Cut | strict | strict_sides |
|---|---|---|
| All 39 | 5: 3 / 2 / 0 | 7: 4 / 2 / 1 |
| Without the 7 www.ed.gov documents | 5: 3 / 2 / 0 | 7: 4 / 2 / 1 (none of them filed at either level) |
| Without the 8 FARA documents | 5: 3 / 2 / 0 | 6: 3 / 2 / 1 |

Bench headline (identical for both levels apart from routing): filename 16/38, type 28/39, date
36/38, parties 34/39; routing 8/29 at strict, 7/29 at strict_sides.

## What this can and cannot say

One really wrong name decides it. 1 of 7 has a 95% interval of roughly 0.4% to 58%, so this set
does not show strict_sides is worse than 10% in general; it shows that on the one held-out set it
was given, the side hold was the only thing stopping a wrong date, and the preregistered bar was
missed. Pooled with PR 93's development replay (3 of 63 really wrong), it is 4 of 70 (5.7%), but
the development sets were read before, so that pooled figure is not a held-out result and the
prereg does not use it.

## The run

- Qwen3.5 9B Q4_K_M (SHA-256 `03b74727…52b7e8`), the pinned Linux runtime (llama.cpp b11946),
  `--no-repack`, 4 threads, token traces on, one live run of all 39 documents (the 7 www.ed.gov files
  saved by hand with their pinned SHA-256), built from PR 93's head `27a97a19`, recorded at gate 0.7,
  strict. The recording (SHA-256 `fe08419f…b9866b`) was replayed through the same build at 0.7 with
  `--review-level strict` and `--review-level strict_sides`; the strict replay matches the live run.
- A 4-vCPU, 15 GB cloud container, not a GitHub runner, as the addendum says. Per document: p50
  49.5 s, p95 88.1 s; 35 minutes in all; server peak memory 6.0 GiB.
- Severity by `bench/reports/review-levels-2026-11/policy/severity.py`, then every filed name not
  exactly right read by hand.
- The recording and reports hold document text and are not committed.

## Status of the set

The coordinator and this thread have now read the results, so real9 is no longer blind to them.
A later round must not describe or quote its documents or results in a brief, and a rule written
later that changes one of its readings is reported with that caveat, by document.

## After the run: why strict held 34 of 39 (development analysis)

Written after the results were read, so **real9 is development data from here**; nothing below
is a held-out result. No new model run: the same recording, at strict and 0.7.

The 34 held names, by the severity script, then read by hand (the hand count moves
real9-letter-01 and -05, "Letter from U.S. Department of Education, Office for Civil Rights",
from minor to right: the comma variant is the gold issuer):

| | right | minor | really wrong |
|---|---:|---:|---:|
| Held at strict (34), script | 13 | 12 | 9 |
| Held at strict (34), by hand | 15 | 10 | 9 |
| Filed at strict (5) | 3 | 2 | 0 |

So on fresh documents the 9B's name was exactly right on 18 of 39, and the gate held 15 of those
18. Both causes are large. The model gave 19 names that deserve a look (10 minor and 9 really wrong).
The gate held 15 good names, five times as many as it let through.

**Reasons on the held names** (a name can carry several; script classes):

| Reason | right | minor | really wrong | Sole reason on |
|---|---:|---:|---:|---|
| LOW_CONFIDENCE (token gate, below 0.7) | 11 | 7 | 8 | 5 right, 1 minor |
| PARTY_ROLE_UNCONFIRMED | 5 | 2 | 3 | 1 right, 1 really wrong |
| TYPE_MISSING | 3 | 2 | 1 | 1 right |
| PARTY_UNSUPPORTED | 2 | 7 | 2 | 2 minor |
| TYPE_UNSUPPORTED | 1 | 6 | 1 | 0 |
| TYPE_INFERRED | 1 | 2 | 2 | 0 |
| DATE_AMBIGUOUS | 0 | 2 | 0 | 1 minor |
| PARTY_MISSING | 0 | 1 | 1 | 1 minor |
| others (PARSER_WARNING, DATE_UNSUPPORTED, DESCRIPTION_*) | 0 | 3 | 4 | 0 |

**The token gate holds the most right names.** It is on 11 of the 13 right names that were held
(13 of 15 by hand). It alone holds 6 names: real9-contract-04, notice-01, notice-02, minutes-01 and
invoice-01, which are right (least token probability 0.47 to 0.69), and memo-03, which is minor
(addressed to the City Manager by name). It also sits on 8 of the 9 really wrong names, but each of
those has another engine reason that would hold it anyway. Without it, here, 5 right and 1 minor
would be filed and nothing really wrong.

Removing the token gate together with the side check would file 13 names here, 3 of them really
wrong (23%), so the two do not combine.

Caveats:
- 39 documents, one model run.
- Earlier development work found the 0.7 gate worth keeping: at 0.6 the 9B filed 9 wrong names of
  28 on real sets 1 to 4.
- The learned verifier of PR 91 did worse than the plain token minimum.
- This is evidence about where the right names go, not a case for lowering the gate.

The type reasons (TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING) and PARTY_UNSUPPORTED mostly sit
on names that deserve holding: their really wrong and minor names outnumber their right ones,
7 right against 23, counting a name once for each reason it carries.
