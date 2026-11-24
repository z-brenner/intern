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
