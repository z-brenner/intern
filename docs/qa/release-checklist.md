# Intern v0.1.0-alpha.21 release checklist

**QA automation: PASS, including the update over alpha.14. Capture acceptance: PASS, by Claude Code under the maintainer's instruction to sign off on its own and release (no person looked at the capture). Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `3ef23e8838ce191d851a316bbcc2b7147adf2a78`
- Runner: `Windows/X64`
- Run: `44229682022` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 | 103 Vitest files / 1304 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 109 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 2217 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/smoke-installer.ps1 -PreviousInstallerPath` | 0 | update over v0.1.0-alpha.14 with /UPDATE, its root llama.cpp build removed, empty folder after uninstall |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 5494 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #88 changes how releases are built and checked, not the app: one shared Windows runtime cache, pull requests that run only the Windows jobs their change needs, the update over alpha.14 moved into QA, and a release that ships on a verified QA run instead of repeating its gates and scoring. It also moves every version surface to alpha.21. It merged after every check on its final head passed (`8c44576`), including that head's own QA run (`44224104216`, 63 minutes), the first to take the update over alpha.14 inside QA.
QA artifact `intern-whole-product-qa-44229682022-1` (ID `14399141695`) was downloaded; its digest matched the one GitHub reports and all twelve manifest file SHA-256 digests were checked.
The original artifact correctly records rendered fidelity as pending; the review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest; its blank Vitest counts are filled from the run's log.

The release-input digest is `ceaa274b32d0a6e07cff6b45deb1c1993c5006dc496d44a93a72452202960f9b`, recomputed on main with `scripts/hash-release-inputs.mjs` and equal to the one the run recorded.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18, date-role accuracy 16/16, parties 15/19 and exact filenames 9/18: every count the same as alpha.20's QA, as it should be with nothing in the engine changed. Five documents' recorded facts still differ from alpha.20's run on the same inputs (party roles in the NDA and the consulting amendment, which party issued the multi-date invoice, a citation in the JPEG, and the ambiguous note's description and review reasons). That is run-to-run variation in the model on a different machine; none of it changed a score, and the invoice and the note wait for review either way. QA scores the model's own readiness, not the app's 0.70 token gate.
The whole evaluation took 690.6 seconds on this runner, against 539.9 seconds in alpha.20's QA, with identical code and model; mean inference per document was 36.1 seconds against 28.1. The build and test steps of this run were each 15 to 35 percent slower than the same steps in the pull request's QA run an hour earlier, so the difference is most likely the runner, whose CPU is not recorded. Downloading, checking and starting the model before the first document took about 9 minutes here and under a minute in that pull request run.
Peak model RSS was 5,494 MB, against 5,517 MB in alpha.20's QA.
These checks do not reproduce a user's own documents or establish improved model accuracy.

## Installed upgrade

QA's own step, "Smoke test an update over v0.1.0-alpha.14", passed: alpha.14's published installer was installed (51 files at the root, 22 of them its root llama.cpp build), this build was installed over it with `/UPDATE` as the in-app updater does, the old root build was gone afterwards (29 files at the root), and the folder was empty after the uninstall. Its log is `logs/update-smoke.log` in the artifact. No separate CI run was needed for it.

## Capture review

Claude Code opened the fresh capture at full size and accepted it under the maintainer's instruction to sign off on its own; no person looked at it. It is byte for byte the accepted alpha.20 capture, which is what a release that changes nothing in the window should produce; that was checked from this run's image, and no previous release acceptance was carried forward.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow, dispatched with `qa_run=44229682022`, verifies that run and publishes alpha.21 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
