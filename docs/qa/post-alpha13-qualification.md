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
  no cache flush. Results are pending. Recorded model latency and server memory
  are inherited evidence and must not be attributed to the replay process.
- Matching Windows QA, fresh confidence-aware model results, real-document and
  CUAD checks after the date rules, and actual 16 GB laptop cold/warm behavior
  remain outstanding. CI success alone does not establish release quality.

The experimental status and run receipts are updated in the companion JSON.
Existing alpha.13 release artifacts and its fidelity record are retained as
historical records. These code changes invalidate that release's input digest;
any subsequent release needs fresh run-bound evidence and the fidelity review
required by `docs/releasing.md`. No PR, approval, merge or release is performed
by this qualification branch.
