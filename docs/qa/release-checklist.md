# Intern v0.1.0-alpha.13 release checklist

**Release status: blocked pending exact-main validation and the deliberately
dispatched release workflow.** Whole-product QA has run and the rendered
fidelity sign-off is accepted, so those two gates are no longer
**pending/blocked**. Everything the release job owns still is. Nothing in this
checklist authorizes a tag or publication.

## Hosted QA artifacts and accepted alpha.13 evidence

- Workflow: Whole-product QA evidence, run `41777820194`, attempt `1`.
- Commit: `d900c11604efefaa56616230f0b0818c26df3911`; runner: `Windows/X64`.
- Execution result: every step passed, including the real-inference corpus
  evaluation on the Qwen3.5 9B model, which `validate-model-evaluation.mjs`
  accepted.
- Release-input digest:
  `40df3381aac1a22c7cbfcd50944dc4f5c4dd629ba710e03eaef9f6d0a3864c32`.
- Capture: `docs/qa/latest-implementation.png`, 1536x1024, SHA-256
  `a95052639f75b6e2bdad0438da20645b59490271748ca2a9dab901fdb31cbd25`.
- Fidelity reviewer: Claude Code accepted the capture under the maintainer's
  standing instruction for this release; no person has looked at it. The
  record is in `rendered-fidelity-signoff.json`, bound to the digest above, and
  the reasoning, and who did what, is in `fidelity-ledger.md`.

| Gate | Status | Hosted run artifact or post-run evidence |
|---|---|---|
| Frontend unit, lint, and build check | accepted | `npm run check` exit 0; TypeScript, 77 Vitest files (753 tests passed, 1 skipped), and the Vite production build. |
| Browser core interaction, accessibility, and 1024-pixel layout | accepted | `npm run test:e2e` exit 0; 30 Playwright tests, and the 1536x1024 capture. |
| Rendered fidelity review | accepted | The capture above, reviewed and bound to the alpha.13 release-input digest. |
| Rust formatting and workspace lint | accepted | `cargo fmt --all -- --check` and `cargo clippy --locked --workspace --all-targets -- -D warnings`, both exit 0. |
| Rust workspace tests | accepted | `cargo test --locked --workspace --all-targets` exit 0; 1,753 tests. |
| Pinned runtime assets and native fixtures | accepted | `npm run assets:verify -- --require-bundled` exit 0 (76 runtime files, 27 license files); 4 native fixture tests with the pinned assets. |
| Windows Tauri/NSIS build | accepted | `npm run tauri build -- --bundles nsis -- --locked` exit 0; one installer. |
| Installer and installed-core smoke | accepted | `scripts/smoke-installer.ps1` exit 0: install, both installed llama-server builds started, packaged worker PDF/OCR, uninstall, user data retained. |
| Corpus evaluation and model acceptance | accepted | 19 documents scored on the pinned Qwen3.5 9B model, peak model RSS 8,132 MiB. Every accuracy gate is above its floor, and no document was filed under a forbidden date or party. |
| Exact-main validation | pending/blocked | The release workflow must verify its dispatch target is the exact current `main` commit. |
| Deliberately dispatched release workflow | pending/blocked | Rebuild, updater signature verification, checksums/SBOM/evidence acceptance, provenance, annotated tag creation, and publication remain release-job gates. |

## Release boundary

The QA workflow has read-only repository permissions and cannot tag, push, or
publish. The release workflow independently checks the exact main commit and
recreates its release evidence. It fails closed unless the model evaluation,
the fidelity sign-off, the installed-core smoke, updater verification,
checksums, SPDX SBOMs, and the evidence manifest are accepted before
provenance, annotated tag creation, and GitHub release publication.

The release ships the reviewed capture rather than generating a new one after
review. Freshness is supplied by the committed non-QA `release_inputs_sha256`;
changing a relevant release input requires a new capture and sign-off.
