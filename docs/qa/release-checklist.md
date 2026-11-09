# Intern v0.1.0-alpha.15 release checklist

**Release status: pending/blocked.** Alpha.14's accepted evidence is historical
and does not qualify these release inputs. No tag or publication is authorized
by this checklist.

| Gate | Status | Required evidence |
|---|---|---|
| Frontend lint, unit tests and build | pending/blocked | `npm run check` on final inputs. |
| Browser interactions, accessibility and layout | pending/blocked | `npm run test:e2e` and fresh 1536x1024 QA capture. |
| Rust format, strict lint and workspace tests | pending/blocked | Rust 1.88 hosted Linux and Windows CI. |
| Corpus replay and real inference | pending/blocked | Recorded-baseline checks and whole-product QA model evaluation. |
| Pinned assets and native fixtures | pending/blocked | Bundled asset verification and native fixture tests. |
| Windows NSIS build and installed-core smoke | pending/blocked | Whole-product QA Windows build, install, runtime, worker and uninstall checks. |
| Update over published alpha.14 | pending/blocked | Deliberately dispatched CI's upgrade smoke with the pinned alpha.14 installer. |
| Rendered fidelity | pending/blocked | Review this QA run's capture under `docs/releasing.md`; record exact digest and screenshot hash. |
| Exact-main release preflight | pending/blocked | All input changes merged before QA; only `docs/qa/` changes after QA. |
| Release workflow | pending/blocked | Deliberate dispatch after preparation; signed updater verification, checksums, SBOMs, evidence acceptance, provenance, tag and publication. |

Follow `docs/releasing.md`. Do not carry forward the old capture's acceptance,
change a digest to bypass review, or treat green CI as installed-app evidence.
