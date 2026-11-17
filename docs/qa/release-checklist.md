# Intern v0.1.0-alpha.20 release checklist

**QA automation: PASS. Capture acceptance: PASS, by Claude Code under the maintainer's instruction to sign off on its own and release (no person looked at the capture). Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `d54288db4d8f9a5091f47c0e685f8cbf7a7244ca`
- Runner: `Windows/X64`
- Run: `44191876137` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 |  101 Vitest files / 1284 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 109 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 2217 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 5518 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #85 was built in separate worktrees, one per work package (naming, model context, confidence and the review gate, review panel and viewer, main window, Settings, the rename to ATLAS), each by its own agent, then merged and checked as a whole: the browser journeys no single package ran found seven failures, all fixed before the merge. It merged after every check on its final head passed (`744f55a`). The first QA run on it (`44187628650`) failed one unit test on Windows, a real race between the launch selection and a view switched to by hand; PR #86 fixed it, with a test that waits for the toggle and checks it again a moment later.
QA artifact `intern-whole-product-qa-44191876137-1` (ID `14381776778`) was downloaded; its digest matched the one GitHub reports and all eleven manifest file SHA-256 digests were checked.
The original artifact correctly records rendered fidelity as pending; the review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest; its blank Vitest counts are filled from the run's log.

The release-input digest is `0ecbed124343f9379790361427f3630bb96f94e84e7f5a7a668a38862cdeabf9`, recomputed on main with `scripts/hash-release-inputs.mjs` and equal to the one the run recorded.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18 and date-role accuracy 16/16; exact filenames were 9/18, the same counts as alpha.19's QA. QA scores the model's own readiness, not the app's 0.70 token gate.
The whole evaluation took 539.9 seconds on this runner, against 532.6 seconds in alpha.19's QA, though this release widens the evidence the model reads; the corpus's documents are mostly short enough to have gone whole before, so the widening barely touches it. The runner's CPU is not recorded.
Peak model RSS was 5,517 MB, against 5,493 MB in alpha.19's QA.
These checks do not reproduce a user's own documents or establish improved model accuracy. What the wider evidence changes is measured on InternBench, recorded again with the accurate model for this release (filename 72 of 77 against 71, nothing wrong filed without a check), in `docs/releases/v0.1.0-alpha.20.md`.

## Installed upgrade

Manual CI run `44191882378` on the same source commit passed both the per-user installer smoke and the update-over-alpha.14 smoke, the installer smoke now checking that Send to > ATLAS replaces Send to > Intern.

## Capture review

Claude Code opened the fresh capture at full size and accepted it under the maintainer's instruction to sign off on its own; no person looked at it. It differs from the accepted alpha.19 capture in what this release meant to change: the ATLAS brand, Copy table in the More menu, the counts as words, the panel opening with the document before What to check, and the confidence inside Analysis details. The organization-led names, the reason chips, the viewer, Clear queue and the double-click are not in this frame; browser tests cover them. No previous release acceptance was carried forward.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow finishes and publishes alpha.20 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
