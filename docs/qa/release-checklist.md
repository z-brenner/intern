# Intern v0.1.0-alpha.19 release checklist

**QA automation: PASS. Capture acceptance: PASS, by Claude Code under the maintainer's instruction to merge and release (no person looked at the capture). Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `ac7f859a51f4bffcb9b5412c74052d489561d3c3`
- Runner: `Windows/X64`
- Run: `43685838406` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 | 90 Vitest files / 1032 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 95 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 2085 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 5493 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #82 was built in separate worktrees per work package, each with one implementer and one adversarial verifier whose verdict was ship or roll back. All four packages were ruled ship. The verifiers of the review window's two changes found three narrow ways a second click could act on something nobody had looked at; each was closed with the verifier's own remedy and a unit test that fails without it, and the probes that found them were run again on the committed code. The pull request's description lists what each change costs and what nobody ran. It merged after all six checks passed on its final head (`0dea769`).
QA artifact `intern-whole-product-qa-43685838406-1` (ID `14151744476`) was downloaded; its digest matched the one GitHub reports and all eleven manifest file SHA-256 digests were checked.
The original artifact correctly records rendered fidelity as pending; the review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest; its blank Vitest counts are filled from the run's log.

The release-input digest is `c29aeeffd45717e4ef4069f5d6c3879645285858ab19c421798f382036d0f23f`, recomputed on main with `scripts/hash-release-inputs.mjs` and equal to the one the run recorded.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18 and date-role accuracy 16/16; exact filenames were 9/18, the same counts as alpha.18's QA. QA scores the model's own readiness, not the app's 0.8 token gate.
Median inference was 25.7 seconds on this runner, against 22.5 seconds in alpha.18's QA. The runner's CPU is not recorded and no change in this release touches the accurate model's inference, so the difference is the runner's, not the release's.
Peak model RSS was 5,493 MB, against 5,520 MB in alpha.18's QA.
These checks do not reproduce a user's own documents or establish improved model accuracy. This release has one engine rule (a spreadsheet date worked out by TODAY or NOW waits for a check); this corpus has no spreadsheet, and its counts are the same as in alpha.18's QA. What the rule changes is measured in `docs/accuracy-research-2026-10.md`, and how the engine reads documents nobody had seen is in `bench/reports/heldout-real8-2026-11`.

## Installed upgrade

Manual CI run `43685841025` on the same source commit passed both the per-user installer smoke and the update-over-alpha.14 smoke.

## Capture review

Claude Code opened the fresh capture at full size and accepted it under the maintainer's instruction to merge and release; no person looked at it. The capture is byte for byte the one accepted for alpha.18: nothing this release changes is drawn in this frame. The two changes to the review window are covered by browser tests in Chromium on Linux and Windows, and nobody has run either in the Windows app's own window. No previous release acceptance was carried forward.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow finishes and publishes alpha.19 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
