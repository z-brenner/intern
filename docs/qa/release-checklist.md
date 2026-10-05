# Intern v0.1.0-alpha.11 release checklist

**Release status: blocked pending exact-main validation and the deliberately
dispatched release workflow.** Whole-product QA has run and the rendered
fidelity sign-off is accepted, so those two gates are no longer
**pending/blocked**. Everything the release job owns still is. Nothing in this
checklist authorizes a tag or publication.

## Hosted QA artifacts and accepted alpha.11 evidence

- Workflow: Whole-product QA evidence, run `37365868306`, attempt `5`. Attempts
  1 to 4 were cancelled by GitHub before any step ran, during a GitHub Actions
  incident that left jobs without a runner.
- Commit: `6aa24a522a78475ba7cff02b6b756159b0d67b9b`; runner: `Windows/X64`. Its
  tree is identical to `main` at `6ee8a404b37fe17ac50d14106affba40d0f53806`.
- Execution result: every step passed, including the real-inference corpus
  evaluation, which `validate-model-evaluation.mjs` accepted.
- Release-input digest:
  `7cc3645acd565a349bc58c8cfeab086792c52d282e03704beb0744e17ac1025d`.
- Capture: `docs/qa/latest-implementation.png`, 1536x1024, SHA-256
  `f4bf894357019c77d251b0f0ef9c379e71c2665d67b901c10c8976526bc0aee4`.
- Fidelity reviewer: the maintainer, Zachary Brenner, accepted the capture. The
  record is in `rendered-fidelity-signoff.json`, bound to the digest above, and
  the reasoning is in `fidelity-ledger.md`.

| Gate | Status | Hosted run artifact or post-run evidence |
|---|---|---|
| Frontend unit, lint, and build check | accepted | `npm run check` exit 0; TypeScript, Vitest, and the Vite production build. |
| Browser core interaction, accessibility, and 1024-pixel layout | accepted | `npm run test:e2e` exit 0; 23 Playwright tests, and the 1536x1024 capture. |
| Rendered fidelity review | accepted | The capture above, reviewed and bound to the alpha.11 release-input digest. |
| Rust formatting and workspace lint | accepted | `cargo fmt --all -- --check` and `cargo clippy --locked --workspace --all-targets -- -D warnings`, both exit 0. |
| Rust workspace tests | accepted | `cargo test --locked --workspace --all-targets` exit 0; 1,073 tests. |
| Pinned runtime assets and native fixtures | accepted | `npm run assets:verify -- --require-bundled` exit 0 (51 runtime files, 23 license files); 4 native fixture tests with the pinned assets. |
| Windows Tauri/NSIS build | accepted | `npm run tauri build -- --bundles nsis -- --locked` exit 0; one installer. |
| Installer and installed-core smoke | accepted | `scripts/smoke-installer.ps1` exit 0: install, Send to shortcut, packaged worker PDF/OCR, app launch and clean shutdown, uninstall, user data retained. |
| Corpus evaluation and model acceptance | accepted | 19 documents scored on the pinned model. Every accuracy gate is above its floor, and no document was filed under a forbidden date or party. |
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
