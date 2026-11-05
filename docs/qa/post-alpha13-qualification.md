# Post-alpha.13 model qualification

This is experimental follow-up evidence, not a release sign-off. The branch
`qualification/post-alpha13` starts from shipped alpha.13 commit
`8299690cd2713fd53bb120f81a9a0adbee58c432` and preserves its release history.
The earlier `fix/pr62-qualification` branch is unchanged.

## Published confidence-policy audit

The alpha.13 `model-evaluation.json`, `release-evidence-manifest.json` and
`latest.json` were downloaded from the public release and checked against the
SHA-256 digests returned by GitHub's release API. Their hashes are retained in
[`post-alpha13-qualification.json`](post-alpha13-qualification.json).

The evaluation is a live, 19-document Qwen3.5 9B run from the shipped commit.
Its median inference time is 20,413 ms and its peak model RSS is 8,534,339,584
bytes. It names 11 documents and matches 9 of 18 scored filenames. **All 19
completed records lack token probabilities.** The tagged desktop code requests
probabilities and configures the accurate model's 0.7 gate, but the tagged
release evaluator supplies neither CLI option. The published results therefore
do not measure the desktop app's confidence-gated readiness behavior. They are
not a 16 GB laptop measurement.

## Reconciled changes

- The three earlier fixes apply cleanly: explicit document dates take
  precedence over inappropriate signatures; every installed tier must fit
  measured memory; QA requests confidence and binds its configured threshold
  to the accurate tier's app policy.
- Release validation now also requires returned probabilities for completed
  records and verifies that below-threshold results are actually held for
  review. Request metadata alone is insufficient. Missing probabilities,
  invalid values, inconsistent readiness and exact-threshold behavior have
  behavioral regression tests.
- The existing ASCII normalization optimization is carried separately. It
  retains the Unicode fallback and tests equivalence for every ASCII byte and
  pair, mixed runs and representative Unicode. Its performance experiment
  measures recorded-response replay CPU work, not model inference.
- `qualification/**` pushes opt into the existing non-publishing Windows QA
  workflow. Repository permissions remain read-only. Changes solely under
  `docs/qa/` do not repeat the run; they also do not change the release-input
  digest, as the repository's existing evidence design requires.

## Checks and pending evidence

- Reconciled portable Rust workspace: **1,496 passed, 6 ignored**, excluding
  `intern-app`; strict Rust 1.88 Clippy and formatting pass.
- Frontend: **761 tests in 77 files**, type checking and production build pass.
- The reconciled validator rejects the original hash-verified published model
  report for absent probability collection.
- Local desktop compilation is blocked by missing GTK development libraries.
  The configured Debian snapshot mirror is denied by the executor's enforced
  network policy. The dependency bootstrap installed nothing and was stopped.
- The frozen-binary replay comparison uses ABBA order, two runs per arm, with
  no cache flush. Median wall time falls from **118.35 s to 65.06 s (45.0%)**.
  All 77 filenames, descriptions, proposals, scores, probabilities and readiness
  decisions agree across all four runs; all baseline gates pass with zero stale
  prompts. Peak replay RSS stays about 54 MiB. This includes benchmark scoring;
  it is not a measurement of live app latency or model inference. Recorded
  model latency and server memory remain inherited evidence.
- The reconciled 9B replay scores **73/77 filenames, 77/77 dates, 77/77 parties,
  zero forbidden dates and zero wrong-ready filenames**, with 37 ready. The 2B
  control retains 71/77 filenames, 76/77 dates, 75/77 parties and 65 ready, with
  zero baseline regressions. Both fixture pipelines pass their original
  baselines with no regressions. Existing fixture party losses and the digest
  recording's truncated reply remain; they are not erased by the replay pass.
- Matching Windows QA has passed the frontend checks, browser QA, formatting,
  strict full-workspace Clippy and the **full Windows Rust workspace tests**,
  including the desktop app tests. This is the workflow step result, not an
  independently downloaded test-count receipt. At 2026-11-05 00:39 UTC the run
  is building the parser and evaluator. Installer checks and fresh
  confidence-aware model results are still pending. Real-document and CUAD
  checks after the date rules, and actual 16 GB laptop cold/warm behavior,
  also remain outstanding. Partial CI success does not establish release quality.

Matching Windows QA is running at
[41940189938](https://github.com/z-brenner/intern/actions/runs/41940189938),
source head `875bb6e49b10d3feb38ad23c21377941d667090a`. Later commits update only
these QA evidence files; the non-QA release-input digest remains
`e1e96bd395259779eec3557b9b146d066bd426933d95f881ddf20308fa011530`.
The entire run and release qualification remain pending; the completed steps
above are the only Windows success claims at this checkpoint.

The experimental status, source hashes and run receipts are in the companion JSON.
Existing alpha.13 release artifacts and its fidelity record are retained as
historical records. These code changes invalidate that release's input digest;
any subsequent release needs fresh run-bound evidence and the fidelity review
required by `docs/releasing.md`. No PR, approval, merge or release is performed
by this qualification branch.
