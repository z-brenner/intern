# CPU cohorts and release interpretation

This records an observational association, not proof of CPU causation or
release qualification. The current action-noun fix is still under Windows QA
[41964133224](https://github.com/z-brenner/intern/actions/runs/41964133224),
source `b702a134fc12826bdbf654a75c3273d476d2c28d`, release-input digest
`e8a4ddb7775398f8cf902df329a973dc90ce11b8b7f68efe845ed29fcd871aeb`.

## Evidence sources

The parent independent audit reports matching hashes for all 21 fixture
files/extractions, 19 prompts, the model and server executable. It identifies
AMD Family 25 Model 1 in passing run
[41961091444](https://github.com/z-brenner/intern/actions/runs/41961091444) and
Intel Family 6 Model 207 in failed diagnostic run
[41955071603](https://github.com/z-brenner/intern/actions/runs/41955071603).
It also reports that the AMD confidence values exactly match the earlier
passing run. These archive comparisons and processor identifiers have not
been independently inspected from archive bytes on this executor.

This executor independently read both completed job logs and compared every
completed record's confidence summary, readiness and reasons:

| Case | AMD minimum / readiness | Intel minimum / readiness |
|---|---|---|
| Journal | 0.7725790739059448 / ready | 0.6038329005241394 / review |
| Termination notice | 0.7505289316177368 / ready | 0.6733022928237915 / review |
| Statement of work | 0.9344902634620667 / review | 0.9439870715141296 / review |

All 19 confidence objects differ. Only the journal and termination readiness
decisions differ, as the unchanged 0.7 gate requires. The SOW has only
`description_unsupported` in both: the initial grounded-acronym fix did not
correct its captured noun wording. The ambiguous note's confidence-summary
token count also differs (AMD 16, Intel 12), but it stays in review in both.
These are critical-value confidence-summary counts, not total generated
token counts.

## Consequences for the revised fix

The CPU/runtime association is stronger evidence than unexplained random
variation. It does not isolate the cause; matched inputs and executable
hashes alone do not disentangle all runtime effects. Neither cohort is
discarded, and the AMD pass is not credited to the later action-noun fix.

The [revised fix](post-alpha13-subject-actions.md) does not alter probabilities
or use the processor identifier to change routing. Its current QA must finish
live acceptance and replay the same fresh recording through pinned before/after
validators. The comparison must preserve emitted facts and probabilities, with
no stale prompts or readiness loss. Both existing cohort recordings remain
useful for additional paired validation when available.

A hosted pass alone does not establish held-out document correctness, actual
16 GB CPU laptop cold/warm behavior, or the required fidelity review. The
branch remains unqualified for release. No PR, approval, merge or release was
performed, and the uncertain PC run is untouched.

All 19 compared diagnostic rows and their provenance are retained in
[`post-alpha13-hardware-cohorts.json`](post-alpha13-hardware-cohorts.json).
