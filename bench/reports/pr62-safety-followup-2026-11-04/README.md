# PR 62 safety follow-up, 2026-11-04

These changes follow PR 62 at `b94198418dc7aad57329b91bea2cca4e764dc60c`.
Its merge into main, `e4f1d706083c2f2ee77fefa35d1d01000cf38481`, has the same
tree. This is an isolated follow-up; it does not modify the parallel branch.

## What changed

- A signature gives way to a unique, explicit document-date field that both
  evidence views support. A lease's **Date of this Lease** wins over signing,
  commencement and rent commencement. A policy's **Policy Period / From**
  gives the coverage start, not its countersignature. Prior policy periods,
  cancellation notices and a lease date mentioned in an amendment are excluded.
  Conflicting fields or an unsettled numeric convention require review.
- Effective-date alternatives are classified at each date's position. The
  end of a term in the same sentence no longer competes with its effective
  date. A bare date in a verified signature region is treated as a signature.
- The QA/release evaluation requests token probabilities and applies the
  shipping confidence gate. Its report exports the actual threshold and the
  accurate tier's threshold directly from the app. The validator rejects a
  missing or different policy, missing probability collection, and replay
  offered as live release qualification. Accuracy and review-rate gates remain.
- Every installed tier must fit measured available memory, including Fastest
  and a sole installed model. If neither fits, no model is selected. The
  existing `MODEL_NOT_READY` queue behavior preserves the document and retry
  budget. Initial setup explains that memory must be freed before trying
  again; model files and verification are retained.

## Recorded-response comparison

No model was downloaded or run for these measurements. Original extraction
and model replies were replayed through the modified engine, without
`--allow-stale`. Both 77-document arms completed with zero stale prompts.

| InternBench, 77 documents | PR 62 9B, gate 0.7 | Follow-up 9B, gate 0.7 | 2B control, gate 0.5 |
| --- | ---: | ---: | ---: |
| Exact filenames | 70 | **73** | 71 |
| Dates correct | 74 | **77** | 76 |
| Parties correct | 77 | 77 | 75 |
| Forbidden dates | 3 | **0** | 0 |
| Wrong filenames ready | 1 | **0** | 1 |
| Ready | 37 | 37 | 65 |

The three corrections are `sow-under-msa-5p`,
`industrial-lease-dated-in-schedule-25p`, and
`property-policy-declarations-mid-60p`. The lease's correct April 14 date is
its explicit document date; its commencement is July 1. The SOW and policy
remain in review under their original low token probabilities. No additional
document goes to review and no correct filename is lost in either model arm.

The original 9B baseline reports two `needless_review` regressions: a name
that was wrong and reviewed is now correct and reviewed, so that derived
metric becomes true. The new baseline records these two changes alongside
the three date corrections. Only those three documents' scores and their
aggregate metrics change; every original timing value is preserved. No gate,
tolerance, confidence threshold, gold answer or recording is relaxed. The
production `Baseline::compare` passes against that baseline. The 2B replay
passes its original baseline with zero regressions and three reported
type-presence/relation improvements. Both fixture pipelines pass their existing baselines with
zero regressions and zero improvements.

## Validation and limits

- Portable Rust workspace, all targets excluding `intern-app`: **1,494 passed,
  6 ignored, zero failures**. Includes date counterexamples, ambiguity controls,
  low-memory model selection and the existing fuzz suites.
- Rust 1.88 strict Clippy on the same scope and workspace format checks pass.
- `npm run check`: **77 files, 758 tests**, type checking and production build
  pass. Release-policy tests first failed against the old implementation.
- A real evaluator replay exports the shipping threshold, preserves the 0.7
  gate, and is correctly refused as live release evidence.
- Desktop integration tests cover insufficient memory with both tiers and a
  sole accurate tier, no launch, retained verification, and successful retry
  after memory recovers. Existing residency tests now model free memory before
  and after loading separately. These desktop tests are **not run locally**:
  the native workspace build is blocked by missing GTK `glib-2.0` development
  dependencies. No native PowerShell execution is claimed.

This is not release qualification. The new rules still need real-document and
CUAD replay, fresh Windows inference, and cold/warm CPU measurements on a
16 GB laptop with normal applications open. The original speed/RSS evidence
does not change. Replaying recorded replies does not measure current model
latency or memory. The fixture party regressions of the larger model remain.
The local workflow API GET returned `Forbidden`; no Windows workflow dispatch,
PR submission, approval, merge or release was performed by this follow-up.

[`evidence.json`](evidence.json) binds inputs, modified source files, reports,
baseline comparison and test counts by hash. Replay report HEAD metadata names
the base commit because the runs occurred in the isolated working tree; the
source hashes identify the modified engine. Full reports and original-control
recordings are retained under `/workspace/artifacts/intern-pr62-fixes` in the
execution environment.

## Reproduce

```sh
npm ci
npm run fixtures
npm run bench:generate
cargo run --locked -p intern-bench -- run --corpus bench/generated \
  --gold bench/gold.json --manifest bench/manifest.json \
  --replay bench/recording.json --min-token-confidence 0.7 \
  --baseline bench/baseline.json --output report.json --markdown report.md
cargo run --locked -p intern-engine --bin intern-evaluate -- \
  --fixtures fixtures/generated --expected fixtures/expected.json \
  --replay fixtures/corpus-recording.json --min-token-confidence 0.7 \
  --baseline fixtures/corpus-baseline.json --output fixture-report.json
```
