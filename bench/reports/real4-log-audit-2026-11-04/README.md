# Real4 paired filename audit and routing-score safeguard

Inspection: 2026-11-04, main `a7d302136f0b61b352d9b67a0120fffaf50501f2` (after PR 60).
Source run: [41749074250](https://github.com/z-brenner/intern/actions/runs/41749074250),
recorded at `0295a09fdb72988143481f7e024cc9e6ee240114`, before PR 60.

## What was verified here

The authenticated GitHub decoded job logs contain 58 unique completed filenames per arm,
with exactly the same case IDs. The job IDs are 126309394637 (2B), 126309394195 (4B) and
126309394715 (9B). Their logged filenames were rescored with InternBench's actual
`gold_filenames` and `windows_name_key` functions against the committed real4 gold, reproducing:

| Arm | Accepted filename | Compared with 2B |
|---|---:|---|
| Qwen3.5 2B | 13/55 | Control |
| Qwen3.5 4B | 13/55 | 5 gains, 5 losses, 8 both correct |
| Qwen3.5 9B | 15/55 | 5 gains, 3 losses, 10 both correct |

All 58 filenames are kept in [paired-filenames.json](paired-filenames.json). Three cases have
no scored gold filename; they remain in the case cohort and must still be checked for routing.
No run was restarted, no source document or gold answer was edited, and no model was loaded.

These logs do **not** contain per-document readiness, raw model replies, evidence citations or
extracted source text. The artifact service returned the ZIP references, but both direct
transfer and its supported managed file-transfer path failed before delivering bytes into
this workspace. No full report/recording ZIP was inspected here. Consequently the audit does
not independently recompute the previously reported ready-disagreement counts, identify which
individual mismatches were ready, or certify any mismatch as factual harm.

## Useful paired cases

| Case | 2B | Both larger models | What to inspect next |
|---|---|---|---|
| `real4-letter-13` | Letter to Carl Malamud | Letter to PUBLIC.RESOURCE.ORG | Larger models recover the organization instead of its person; accepted by the benchmark |
| `real4-notice-14` | 2023-02-15 Notice to Oregon Department of Environmental Quality | 2025-10-28 notice naming Erickson Incorporated | Candidate dates/parties improve together; audit the cited own-date and party role |
| `real4-invoice-12` | Invoice from WCCO | Invoice from WCCO-TV Advertiser | Possible role-label contamination in the party span; inspect extracted layout and citations before stripping anything |
| `real4-office-01` | Letter to Duquesne Light Company | Letter to Emily M Farah CNSL | The gold notes put the organization above the person in the address; current recovery searches below the person. Inspect the real extraction and role evidence before changing that rule |
| `real4-notice-06` | Notice of Violation for Tennessee Valley Authority | A different type phrase, same party and date | First check the effect of PR 60's type-kind rules; a different title phrase is not automatically factual harm |

These are investigation leads from logged names and reviewed gold notes, not proposed automatic
rewrites. A direct fetch of the pinned `real4-office-01.docx` source was also refused by the
network proxy (HTTP 403); no source bytes were obtained or claimed verified. Choosing whichever
model the gold says was right would be an oracle, not a deployable routing policy.

## Safe independent change

`GoldFile::parse` now refuses an `expected_readiness` string other than `ready`, `needs_review`
or the deliberately unscored `either`. Missing/null stays valid. Previously a typo such as
`review` parsed but silently disabled `readiness_match`, which matters especially for undated
documents excluded from filename scoring. All five committed benchmark/real-document gold
sets already use supported values, so valid scores and their denominators do not change.

Tests cover rejected spellings, all accepted/optional states and every committed gold set.
The benchmark suite passed 137 tests, with no failures or ignored tests. Formatting and diff
checks passed. The valid 77-document replay passed with zero regressions. A CLI check with
`expected_readiness: "review"` failed with exit 1 and a document-specific diagnostic, before
scoring. No baseline rewrite, model/default, confidence gate, naming or application validation
rule is changed by this fix.

`docs/internbench.md` now defines `unsafe_ready` precisely: it is a ready name that fails the
accepted-filename check. A source audit and the separate forbidden-value/claim scores are
needed to establish factual error; an unlisted harmless name variant can fail that benchmark.

## Next step before another rule or release

Replay **all three original real4 recordings on current main first**. PR 60 merged after the
live run and changes only `compose.rs`, `facts.rs` and `phrases.rs`; it targets several type and
role failures already visible here. Let replay verify the original prompt identities rather
than running inference again or treating pre-PR-60 outputs as the current pipeline's outputs.

Then inspect `report.json` and `recording.json` for each newly ready or benchmark-disagreeing
case, especially the organization/person and label-contamination examples above. Only add a
bounded validation fix when source text and the exact model fields establish its trigger;
retain every loss and rerun the other cohorts. Logs alone cannot establish that trigger.

The original ZIP artifacts are 13285828112 (2B), 13286198234 (4B) and 13290126580 (9B). Full
paired replay is blocked locally on those report/recording bytes. The parent has independently
reviewed them and can provide the decoded case entries or perform that replay in an authorized
environment. No denied route was bypassed and no PR submission or release was attempted here.
