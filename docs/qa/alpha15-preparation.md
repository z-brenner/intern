# Alpha.15 preparation evidence

Candidate: `fabd118f771c4fad1a697a44d86ad6f5e12bfba2`, PR #74.
Merged main: `a70010d9b016b01bd5319bb985d50c2bd9abcac8`; release-input digest unchanged.
Release inputs: `93a0d6399cd29afac4809ee548c8aa89f6b3f0c1b4f9d3ef91fb2b8fdc7a1c9f`.

Local Linux verification on 2026-11-09:

- `npm run check`: exit 0, 79 files, 766 tests passed, 1 skipped; TypeScript and production build passed.
- `npm run test:e2e`: exit 0, all 30 Chromium tests passed.
- Rust 1.88 `cargo test --locked -p intern-engine -p intern-bench --all-targets`: exit 0, 666 passed, 4 ignored across 14 suites.
- `cargo clippy --locked -p intern-engine -p intern-bench --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0.
- Evidence corpus, hosted-digest corpus, and InternBench recorded baseline replays: exit 0, zero gated regressions in each. InternBench: 71/77 correct names; one pre-existing unsafe-ready name remains.
- All three new loose signing-date regressions failed before the fix and passed after it.
- Full-suite verification first exposed a pre-existing download-test cleanup hang (mock waiting in accept after cancellation won before the request). The new unused-listener test failed before the helper fix; all 16 download tests passed after deterministic request synchronization and cleanup. The full suite then passed.
- Independent assistant review of the date fix, version preparation, and cancellation-test fix found no remaining important issues. This is code review, not a rendered-fidelity sign-off.

PR CI run `42653053470` and format run `42653053583` passed all six checks,
including Windows native fixtures, NSIS build and per-user installer smoke.
The hosted Linux workspace run passed 1,789 tests, with six ignored.

Exact-main dispatched CI run `42665025743` passed, including the explicit
alpha.14 upgrade smoke. Whole-product QA run `42665012237` completed successfully:
frontend, browser/capture, strict lint, full workspace tests, native fixtures/OCR,
NSIS build, installed-core smoke and real-inference corpus evaluation all passed.
The model report scored 19 documents; 11 were named, all named dates were correct,
and no forbidden dates or parties were observed. The duplicate push-triggered CI
run `42664972583` was canceled in favor of the dispatched run on the same commit.

The downloaded QA artifact validates against the exact commit and run with model
and installed-core acceptance. Zachary Brenner explicitly confirmed he looked at
and accepts the exact fresh capture. The accepted fidelity record names him and
distinguishes his acceptance from assistant inspection. Local browser tests alone
do not establish Windows installer or model-inference readiness; the hosted
installer and real-inference evidence supplies those separate checks.

Final verification: all 40 release-evidence/preflight/version-contract tests passed
against the refreshed QA records. Release preflight passed after the capture
acceptance was recorded. No tag or publication was created. The deliberate
release workflow still owns rebuilding, signing, distribution evidence,
provenance, tagging and publication.
