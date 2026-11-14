# Intern v0.1.0-alpha.18 release checklist

**QA automation: PASS. Capture acceptance: PASS, by Claude Code under the maintainer's standing instruction (no person looked at the capture). Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `3c9ad7052782bdba7e0e21465261382af79c931f`
- Runner: `Windows/X64`
- Run: `43622648750` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 | 86 Vitest files / 933 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 80 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 2061 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 5520 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #80 was built in separate worktrees per work package and per rule track, each reviewed adversarially; rules that a review showed could file a wrong name were narrowed or taken out, and the last verifiers' remaining costs on constructed inputs are written into `docs/accuracy-research-2026-10.md`. Two independent checkers were then set to refute the release notes, the held-out report and the pull request's description against the code and the recorded replays, and what they showed false or overstated was corrected before the merge. It merged after all seven checks passed on its final head (`a3d7671`).
QA artifact `intern-whole-product-qa-43622648750-1` (ID `14126426767`) was downloaded and all six manifest artifact SHA-256 digests were checked.
The original artifact correctly records rendered fidelity as pending; the review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest; its blank Vitest counts are filled from the run's log.

The release-input digest is `6a67a81794faf3ca0da80edef198fc14130d6a82d1be99fedf98ffcfb293ec33`, recomputed on main with `scripts/hash-release-inputs.mjs`.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18 and date-role accuracy 16/16; exact filenames were 9/18, the same counts as alpha.17's QA. QA scores the model's own readiness, not the app's 0.8 token gate.
Median inference was 22.5 seconds on this runner, against 46.1 seconds in alpha.17's QA. The runner's CPU is not recorded and no change in this release speeds the accurate model's inference, so the difference is the runner's, not the release's.
Peak model RSS was 5,520 MB, against 5,489 MB in alpha.17's QA.
These checks do not reproduce a user's own documents or establish improved model accuracy. What this release's engine rules change is measured on recorded replies, in `docs/accuracy-research-2026-10.md` and `bench/reports/heldout-real7-2026-11`; this corpus's counts are the same as in alpha.17's QA.

## Installed upgrade

Manual CI run `43622651508` on the same source commit passed both the per-user installer smoke and the update-over-alpha.14 smoke.

## Capture review

Claude Code opened the fresh capture at full size and accepted it under the maintainer's standing instruction for this release; no person looked at it. The capture differs from the accepted alpha.17 capture in 11 pixels at the upper corners of the Queue and Completed control, each by one colour level, which no one could see. The review-window states this release adds are not in this frame; browser tests cover them. No previous release acceptance was carried forward.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow finishes and publishes alpha.18 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
