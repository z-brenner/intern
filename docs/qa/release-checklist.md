# Intern v0.1.0-alpha.10 release checklist

**Release status: blocked pending exact-main validation and the deliberately
dispatched release workflow.** Whole-product QA has run and the rendered
fidelity sign-off is accepted, so those two gates are no longer
**pending/blocked**. Everything the release job owns still is. Nothing in this
checklist authorizes a tag or publication.

## Hosted QA artifacts and accepted alpha.10 evidence

- Workflow: Whole-product QA evidence, run `35084991785`, attempt `1`.
- Commit: `f7fd55c56ade21b2fac8091b76993aa51f104245`; runner: `Windows/X64`.
- Execution result: every step passed, including the real-inference corpus
  evaluation, which `validate-model-evaluation.mjs` accepted.
- Release-input digest:
  `5e0cc31c1184aaa033a7066e65c336394787aa65889c72792a819e660ac73885`.
- Capture: `docs/qa/latest-implementation.png`, 1536x1024, SHA-256
  `ee46abcfe76433b6ed25b6f7a53c80056bf36918b8b44009ae8f5511c54167a4`.
- Fidelity reviewer: the maintainer, Zachary Brenner, accepted the capture. The
  record is in `rendered-fidelity-signoff.json`, bound to the digest above, and
  the reasoning is in `fidelity-ledger.md`.

| Gate | Status | Hosted run artifact or post-run evidence |
|---|---|---|
| Frontend unit, lint, and build check | accepted | `npm run check` exit 0; TypeScript, Vitest, and the Vite production build. |
| Browser core interaction, accessibility, and 1024-pixel layout | accepted | `npm run test:e2e` exit 0. |
| Rendered fidelity review | accepted | The capture above, reviewed and bound to the alpha.10 release-input digest. |
| Rust formatting and workspace lint | accepted | `cargo fmt --all -- --check` and `cargo clippy --locked --workspace --all-targets -- -D warnings`, both exit 0. |
| Rust workspace tests | accepted | `cargo test --locked --workspace --all-targets` exit 0. |
| Pinned runtime assets and native fixtures | accepted | `npm run assets:verify -- --require-bundled` exit 0; native fixture tests with the pinned assets. |
| Windows Tauri/NSIS build | accepted | `npm run tauri build -- --bundles nsis -- --locked` exit 0; one installer. |
| Installer and installed-core smoke | accepted | `scripts/smoke-installer.ps1` exit 0: install, packaged worker PDF/OCR, uninstall, user data retained. |
| Corpus evaluation and model acceptance | accepted | 19 documents scored on the pinned model. Every accuracy gate is above its floor, and no document was filed under a forbidden date. |
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
