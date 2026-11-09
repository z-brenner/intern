# Intern v0.1.0-alpha.15 release checklist

**Preparation complete: ready for the deliberate release workflow.**
Whole-product QA and dispatched main CI both completed successfully.
Nothing here authorizes a tag or publication.

## Exact release inputs

- Main commit: `a70010d9b016b01bd5319bb985d50c2bd9abcac8`.
- Release-input digest: `93a0d6399cd29afac4809ee548c8aa89f6b3f0c1b4f9d3ef91fb2b8fdc7a1c9f`.
- Whole-product QA: run `42665012237`, attempt `1`, Windows/X64.
- Upgrade CI: dispatched run `42665025743`, same main commit, success.
- Fresh capture: `docs/qa/latest-implementation.png`, 1536x1024, SHA-256
  `c37dbd5cffe9951863dfbd7fb24834f96997d7f4032c670b2dcecf15bb402797`.
- QA artifact: `intern-whole-product-qa-42665012237-1`, artifact `13700745905`.

| Gate | Status | Evidence |
|---|---|---|
| Frontend lint, unit tests and build | accepted | QA evidence log: 79 files, 766 passed, 1 skipped; TypeScript/build and main CI passed. |
| Browser interactions, accessibility and layout | accepted | All 30 QA browser tests passed; fresh capture above. |
| Rust format, strict lint and workspace tests | accepted | QA Windows: 1,788 tests passed, 6 ignored; main Linux: 1,789 passed, 6 ignored. Strict lint and formatting passed. |
| Recorded corpus baselines | accepted | Evidence, hosted-digest and InternBench replays, zero gated regressions. |
| Pinned assets, native parser fixtures and OCR | accepted | QA pinned-runtime, generated parser fixture and staged PP-OCR checks passed. |
| Windows NSIS build and installed-core smoke | accepted | Fresh install, app launch, clean shutdown, runtime inventory, installed worker, uninstall and user-data retention all accepted for this QA run. |
| Update over published alpha.14 | accepted | Dispatched CI explicitly passed “Smoke test an update over v0.1.0-alpha.14”. |
| Production model acceptance | accepted | Fresh 19-document run; 11 named, all named dates correct; zero forbidden dates or parties. Independent local model validator exit 0. |
| Immutable artifact binding | accepted | Downloaded manifest, every bound artifact and installer hash verified locally against the exact commit, workflow, run and attempt with `--allow-pending`; rendered fidelity was pending at artifact upload and is now separately accepted. |
| Rendered fidelity | accepted | Zachary Brenner confirmed he looked at and accepts this exact fresh capture; the record names him and separates his acceptance from assistant inspection. |
| Final release preflight | accepted | Preflight passed with the accepted capture, matching release-input digest, synchronized versions and release notes. Only `docs/qa/` changed after QA. |
| Deliberately dispatched release workflow | pending/blocked | Exact-main verification, signed updater, checksums/SBOM/evidence acceptance, provenance, annotated tag and publication remain release-job gates. |

All input changes landed before QA. Only `docs/qa/` may change now without
invalidating the captured release inputs. The original downloaded QA artifact
remains intact; its blocked manifest truthfully records the pending review.
