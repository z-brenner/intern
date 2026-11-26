# Real set 10, run once: the whole-value score at 0.60 is rejected

Run on 2026-11-24 under prereg Addendum 1 (committed in `9f9b23a9` before the run).
Candidate: commit `95f4ad55` (draft PR 92), `--confidence-score whole-value` at its
threshold of 0.60. Comparison: today's least-token gate, `--min-token-confidence 0.7`.
Both are replays of the same live recording at `--review-level strict`.

## Result

| Gate | Filed | Right | Minor | Really wrong | Really wrong / filed |
|---|---:|---:|---:|---:|---:|
| Least token 0.7 (today) | 6 | 2 | 3 | 1 | 16.7% |
| Whole value 0.60 (candidate) | 9 | 2 | 5 | 2 | 22.2% |

The pass rule needs all three to hold:

1. The candidate files no more really-wrong names than today's gate: **fails** (2 against 1).
2. Really-wrong names are at most 10% of the names the candidate files: **fails** (22.2%).
3. The candidate files at least 2 more right names than today's gate: **fails** (+0).

Rules (1) and (2) fail, so the candidate is **rejected**, and 0.60 is not tuned on this set.

Over all 50 documents, both replays name the same: 10 right, 19 minor and 21 really wrong
(severity script). The score changes only which names are filed. The live run at strict
matches the 0.7 replay on every document.

## Every filed name that is not exactly right, read by hand

| Document | Filed by | Name | Script | By hand | Least token | Whole value |
|---|---|---|---|---|---:|---:|
| invoice-03 | both | `2025-10-30 Invoice from Sri Venugopalan` | really wrong | really wrong: he is the customer's engineer, and the issuer is American Structurepoint | 0.717 | 0.628 |
| contract-01 | candidate | `2025-11-21 Professional Services Agreement between City of Everett and Consor North America, Inc.` | really wrong (date) | **not really wrong**: see the gold gap below | 0.674 | 0.623 |
| contract-04 | candidate | `2025-09-22 Commercial Lease Agreement between Collin County, County Clerk and Collin County Healthcare Foundation` | minor | minor | 0.657 | 0.653 |
| statement-01 | candidate | `2026-04-28 Treasurer Monthly Report from Angelina County` | minor | minor | 0.638 | 0.634 |
| form-01 | both | `2025-01-28 Lobbyist Registration Form for Mr. William G. (BUD) Courson` | minor | minor | 0.710 | 0.708 |
| form-04 | both | `2025-10-20 State of Arizona Committee Campaign Finance Report for Play it Forward Casa Grande` | minor | minor | 0.950 | 0.950 |
| notice-05 | both | `2025-06-13 Notice of Public Hearing ... for County of EL Dorado Planning Commission` | minor | minor | 0.721 | 0.657 |

**Gold gap in contract-01.** The gold takes 2025-11-20, the date beside the mayor's signature,
and says it is the only signature date. That was wrong. The PDF ends with an Adobe Sign audit
report ("Final Audit Report 2025-11-21"). In it, the mayor signs at 2025-11-21 01:51 GMT, which
is 20 November in Everett. The last signature, Marista Jorve's, comes at 2025-11-21 16:50 GMT
("Agreement completed"), which is 21 November locally. The agreement is effective "as of the date
of last signature", so 2025-11-21 is defensible, and this name is not really wrong.

Under the addendum, where the script and the hand reading differ, the larger really-wrong count
decides, so the table above counts 2. With the hand reading instead, the candidate files 1 really
wrong and 3 right names out of 9. That passes (1) but still fails (2) at 11.1%, and fails (3)
at +1. The verdict does not depend on this document. The committed gold is left as sealed. If
2025-11-21 is accepted later, that is a correction made after the run and must be reported as
one.

## What else the run shows

- Today's gate also misses the 10% bar on this set (1 of 6). The really-wrong name both gates
  file, invoice-03, scores 0.717 on the least token and 0.628 on the whole value. It is above
  both thresholds.
- The candidate's three extra filings score between 0.623 and 0.653 on the whole value, just
  above 0.60. Today's gate held all three on their least token (0.638 to 0.674).
- Of the 8 right names strict still holds, 2 are held by the token gate: contract-05 at 0.591
  and notice-02 at 0.498 on the whole value. A threshold low enough to file them would have
  filed more of the names above too. No threshold was tried; this is an observation, not a
  tuning.
- 10 right names out of 50 (49 scored on the name) is low next to earlier sets. Of the 21
  really-wrong names, most are not filed at either gate.

## How it was run

- **Model and runtime.** Qwen3.5 9B Q4_K_M (manifest SHA-256 `03b74727…`), the pinned Linux
  runtime, `--no-repack`, 4 threads, token traces on (`INTERN_BENCH_TOKEN_TRACE=1`, which asks
  for two alternatives per token).
- **Build.** intern-bench and intern-worker were built from `95f4ad55` (the worker with
  `--features windows-native`).
- **Machine.** A 4-vCPU, 15 GB cloud container, as the addendum says.
- **Corpus.** All 50 documents fetched and matched their pins. notice-01 needed a second fetch.
- **Timing.** Wall time was 2,806 s. Per document: p50 53 s, p95 81 s, max 115 s.
- **Recording.** SHA-256 `2c5ca8b7…23b4ee`, kept with the replays in the project files
  (`heldout10/run1/`), not committed.
- **Two starts failed before any document reached the model:**
  - In the first, llama-server was not healthy within the script's wait (the model was loading
    from a cold cache).
  - In the second, the worker had been built without its native PDF and OCR features, so all
    50 extractions failed with `NATIVE_ASSETS_MISSING`, and the model saw only its warm-up
    prompt.

  Neither produced a reading. The run above is the only one in which the model read the set.

## After this run

Real set 10 is no longer blind to this round. The rules of `heldout-real10-2026-11/prereg.md`
apply: a later round must not describe or quote its documents or results in any brief.
