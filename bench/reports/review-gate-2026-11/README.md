# The review gate, decided on a held-out set, November 2026

Status: decided on 2026-11-11. The question was whether the accurate model's
review gate should stay at 0.7 - a reading with no validation reason is filed
without review only when the least probable token of its date and its
parties' names is at least that - or change. It is **0.8** from alpha.17
(`tiers::REVIEW_BELOW_TOKEN_CONFIDENCE_ACCURATE`).

## How it was decided

The rule was written down before any model had read the set it was decided
on: [`policy/prereg.md`](policy/prereg.md), written at 07:35 UTC, with the
SHA-256 of the three scripts that score the candidates
([`policy/`](policy/), unchanged since: `policy/prereg.sha256`, written
with the pre-registration, matches the copies here). In short:

1. A candidate is eligible only if, on the traced development recordings
   (InternBench and the real sets 1-5, Qwen3.5 9B), it files no more wrong
   names than 0.7 does.
2. On the sixth real set, read live and replayed through the release
   candidate's engine, the eligible candidate that files the fewest wrong
   names wins; a tie goes to the one that files the most right names; if
   0.7 is among the tied best, nothing changes.
3. Whatever is chosen, the release notes give the sixth set's right and
   wrong names filed at 0.7 and for the choice, and say that one set of
   this size cannot certify a rate (the alpha.17 notes do).

The candidates: 0.7 and 0.8 as they are (`today`), and each with one extra
rule - `R1` (a one-party "from" name raised only for its party not being in
the text is filed; a name with a party but no joining word never is) and
`C1` (a validation-clean two-party notice "between" its sides is filed
whatever its token probability). Three token-trace variants (`T1`) were
rejected before the held-out set existed, for filing wrong names on the
development sets; they are listed below for scale only.

The criterion counts wrong names filed before right ones: a wrong name is
renamed on disk with no one looking, a right one held for review costs a
click. That ordering favours a higher gate whenever the held-out set has a
wrong name between the two thresholds, and it was chosen knowing that.

## The sixth real set

`scripts/realdocs-eval/sample-heldout5.json`: 55 public documents (19
letters, 12 notices, 6 forms, 6 memos, 6 minutes, 5 invoices, 1 statement),
gold written and second-read before the run, never used to write a rule.
Read live by `candidate-bench.yml` run 43065240563 (Qwen3.5 9B with
`--no-repack`, an AMD EPYC 9V74 runner, engine at `7f34594`); 51 documents
were fetched and read, 45 of them with a scored filename. Replayed through
the engine at `93828f7`, the release candidate (the queue package merged
after it changes no reading: its replays of every development set match).

| Policy | Right names filed | Wrong names filed |
| --- | ---: | ---: |
| today@0.70 (alpha.16) | 4 | 2 |
| **today@0.80** | **3** | **1** |
| R1@0.70 | 4 | 2 |
| R1@0.80 | 3 | 1 |
| C1@0.70 | 4 | 2 |
| C1@0.80 | 3 | 1 |
| T1 (rejected), at 0.70 | 4 | 2-4 |

Each also filed `real6-form-06`, a blank invoice template the gold gives
no name and sends to review, under 2026-03-03, the date its TODAY() cell
held, which the gold forbids, at a least token probability of 0.900. It is
a wrong filing for every candidate, so it does not change the choice, and
the table, which counts what the pre-registration counts (scored names),
leaves it out. Of the 45 scored names 19 are right; 11 readings carry no validation reason and
are judged by the gate alone:

| Document | Least token probability | Name right | 0.7 | 0.8 |
| --- | ---: | --- | --- | --- |
| real6-notice-01 | 0.947 | yes | filed | filed |
| real6-letter-12 | 0.936 | **no** | filed | filed |
| real6-minutes-03 | 0.894 | yes | filed | filed |
| real6-invoice-03 | 0.891 | yes | filed | filed |
| real6-notice-10 | 0.730 | **no** | filed | review |
| real6-minutes-06 | 0.706 | yes | filed | review |
| real6-memo-02 | 0.540 | no | review | review |
| real6-invoice-02 | 0.494 | yes | review | review |
| real6-memo-01 | 0.491 | no | review | review |
| real6-memo-05 | 0.448 | no | review | review |
| real6-letter-02 | 0.077 | yes | review | review |

Three candidates tie at 3 right and 1 wrong, and the pre-registration does
not break that tie. today@0.80 was chosen: R1's and C1's extra rules filed
nothing more than the plain threshold on the held-out set, so nothing
outside the development data speaks for them.

## The development sets, for eligibility

The traced 9B recordings of 2026-11-11, replayed through the same engine
(right / wrong names filed):

| Policy | InternBench | real1 | real2 | real3 | real4 | real5 | All |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today@0.70 | 41/1 | 4/0 | 2/0 | 1/1 | 4/0 | 3/0 | 55/2 |
| today@0.80 | 31/0 | 3/0 | 2/0 | 0/0 | 2/0 | 1/0 | 39/0 |
| R1@0.80 | 31/0 | 6/0 | 3/0 | 0/0 | 3/0 | 1/0 | 44/0 |
| C1@0.80 | 32/0 | 8/0 | 6/0 | 2/0 | 3/0 | 1/0 | 52/0 |

All six candidates were eligible (none filed more than 2 wrong). On the
committed InternBench recording that CI replays (`bench/recording.json`),
0.8 holds 10 more of 77 documents for review (28 ready against 38) and
files no wrong name where 0.7 filed one; no name changes. The three CI
baselines were written again at 0.8, and only readiness scores moved.

## What this does not show

- 51 documents, 1 wrong filing against 2 among the scored names (2
  against 3 with `real6-form-06`): no rate is certified by one set of
  this size, and it is the set the gate was chosen on. Across all six real
  sets, 0.7 filed 18 right names and 3 wrong ones, 0.8 11 and 1 (each one
  more wrong with `real6-form-06`).
- Filing is rare on real documents at either gate: most readings carry a
  validation reason and go to review regardless.
- The cost is in documents a person must open: on InternBench, 10 more of
  77.

## Reproducing

`policy/policies.py <replay report dir> <traced artifacts dir> <set>...`
reads `intern-bench` replay reports (`--min-token-confidence 0.7`, so the
policies can apply their own gate) and the recordings' token traces
(`INTERN_BENCH_TOKEN_TRACE=1`, `candidate-bench.yml`). The real sets' gold
is what `scripts/realdocs-eval/fetch.py` writes for the documents fetched.
