# Intern v0.1.0-alpha.16 release checklist

**QA automation: PASS. Human capture acceptance: PASS. Publication and post-publication verification: pending/blocked until the release workflow and independent verification complete.**

- Commit: `b69ffcfa4f9a42fa93856b3193752ede1dc17d17`
- Runner: `Windows/X64`
- Run: `42722700028` attempt `1`

| Status | Command | Exit code | Exact count/result |
|---|---|---:|---|
| pass | `npm run fixtures` | 0 | 21 gold fixtures |
| pass | `npm run check` | 0 | 80 Vitest files / 770 passed, 1 skipped; TypeScript and Vite build passed |
| pass | `npm run test:e2e` | 0 | 33 Playwright tests; 1536x1024 capture written |
| pass | `cargo fmt --all -- --check` | 0 | workspace formatting |
| pass | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 0 | workspace/all targets |
| pass | `cargo test --locked --workspace --all-targets` | 0 | 1797 Rust tests |
| pass | `npm run assets:verify -- --require-bundled` | 0 | 54 runtime files / 27 license files |
| pass | native `generated_fixtures` | 0 | 4 native fixture tests |
| pass | `npm run tauri build -- --bundles nsis -- --locked` | 0 | one NSIS installer |
| pass | `scripts/smoke-installer.ps1` | 0 | install, packaged worker PDF/OCR, uninstall, user-data retention |
| pass | `scripts/run-model-evaluation.ps1` | 0 | 19 documents scored on Qwen3.5-9B-Q4_K_M.gguf; peak model RSS 10407 MB |
| pass | `scripts/validate-model-evaluation.mjs` | 0 | date/type/party/description accuracy and trap-date gates |

This QA workflow has read-only repository permissions and does not tag, push, or publish.

## Source and review evidence

Implementation PR #75 was reviewed by two independent Codex agents and merged after CI passed.
QA artifact `intern-whole-product-qa-42722700028-1` (ID `13724741319`) was downloaded and all 11 manifest artifact sizes and SHA-256 digests were checked independently by two Codex agents.
Its ZIP SHA-256 is `aadb672041fa225253fb96c6f199d1dcf523595a36740d2602676394211ee4e9`.
The original artifact correctly records rendered fidelity as pending; the subsequent human review is recorded in `rendered-fidelity-signoff.json` and `fidelity-ledger.md`.
This checklist is a post-run review record, not a byte-for-byte replacement for the checklist bound by that original artifact manifest.

The release-input digest is `473175c1ed816db6b94c53bec6d1ab35cdca533ed925dbe9510016fd8187b676`.
Only files under `docs/qa/` change after that QA run.

## Model evaluation limits

The generated corpus passed its acceptance gates: 19 documents evaluated, 11 named, zero forbidden dates or parties. Date accuracy was 16/18; date-role accuracy was 15/16 (alpha.15: 16/16); exact filenames were 9/18. The differing date-role fixture remained marked for review and retained the same proposed filename.
Median inference was 34.3 seconds on this runner. Cross-run timing does not establish a performance change.
These checks do not reproduce the user's exact OneDrive/SharePoint environment or establish improved model accuracy.

## Installed upgrade

Manual CI run `42722705079` on the same source commit passed both the per-user installer smoke and the update-over-alpha.14 smoke. The Windows job was completing post-job cleanup when these results were recorded.

## Human capture review

Zachary Brenner viewed the exact fresh capture and replied “I looked at it and accept”. Acceptance was recorded at `2026-11-10T00:57:53Z`. Codex and an independent reviewing agent inspected the capture and verified its hashes; Codex prepared the record. No previous release acceptance was reused.

## Remaining publication checks

- [ ] Release preflight after the QA-only record is committed.
- [ ] Release workflow finishes and publishes alpha.16 as latest.
- [ ] Downloaded assets, checksums, updater signature and provenance verified.
- [ ] Download page deployment verified.
