# Intern v0.1.0-alpha.17 release checklist

**QA automation: PASS. Capture acceptance: PASS, by Claude Code under the maintainer's standing instruction (no person looked at the capture). Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `82f0f8da25f23ba301f57c4d7b45e3914d33e9bf`
- Runner: `Windows/X64`
- Run: `43100350279` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 | 81 Vitest files / 820 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 34 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 1894 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 5489 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #77 was reviewed by independent agents per work package, then by a final adversarial review of everything added after those reviews (19 confirmed findings, all fixed), and by the Codex review bot (no findings). It merged after all five CI jobs passed on its final head.
QA artifact `intern-whole-product-qa-43100350279-1` (ID `13896976315`) was downloaded and all six manifest artifact SHA-256 digests were checked.
The original artifact correctly records rendered fidelity as pending; the review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest; its blank Vitest counts are filled from the run's log.

The release-input digest is `05fceed5313fef26b52ac3bf2d6061bca27ed3717ad7de80f524716f18f12a8d`, recomputed on main with `scripts/hash-release-inputs.mjs`.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18 and date-role accuracy 16/16; exact filenames were 9/18. QA scores the model's own readiness, not the app's 0.8 token gate.
Median inference was 46.1 seconds on this runner, against 34.3 seconds in alpha.16's QA: decode was slower and prefill faster on a runner whose CPU is not recorded. A same-runner comparison on an Intel Xeon Platinum 8370C (`runtime-speed.yml` run 43113377135, 9 requests twice each) put the accurate model with `--no-repack` about 4% faster per request than without it (prefill 12% faster, decode 15% slower), so cross-run QA timing does not show a slowdown from this release.
Peak model RSS was 5,489 MB, against 10,407 MB in alpha.16's QA.
These checks do not reproduce a user's own documents or establish improved model accuracy.

## Installed upgrade

Manual CI run `43100354343` on the same source commit passed both the per-user installer smoke and the update-over-alpha.14 smoke.

## Capture review

Claude Code opened the fresh capture at full size and accepted it under the maintainer's standing instruction for this release; no person looked at it. The capture is byte-identical to the accepted alpha.16 capture. No previous release acceptance was carried forward.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow finishes and publishes alpha.17 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
