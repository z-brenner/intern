# Alpha.15 Release Preparation Plan

**Goal:** Prepare the next Windows alpha with honest, commit-bound QA evidence.
**Spec:** `docs/releasing.md`.
**Approach:** Execute in this session. Review the green accuracy PR before inclusion; keep the older experimental PRs outside this release. Prepare all release inputs before QA; never publish or reuse an earlier screenshot sign-off.

- [x] Review PR #73 and resolve any outstanding correctness findings before inclusion.
- [x] Synchronize current version surfaces to 0.1.0-alpha.15, preserving historical records and the alpha.14 upgrade source and checksum.
- [x] Write release notes covering runtime removal, memory planning, included accuracy work, and measured limitations. Reset QA evidence to pending.
- [ ] Run frontend checks and browser tests; verify Rust changes with Rust 1.88 and hosted Linux/Windows CI.
- [ ] Review and merge the release preparation once checks pass.
- [ ] Dispatch whole-product QA and upgrade CI on the exact main commit; diagnose failures without weakening gates.
- [ ] Review the new capture and record truthful sign-off under docs/releasing.md; run release preflight. Do not publish a release as part of preparation.

Local validation: frontend check (766 passed, 1 skipped), all 30 browser tests, three signing-date regressions (red then green), and all three committed baseline replays passed. The full Rust run exposed an inherited cancellation-test cleanup hang; the mock now wakes an unused listener, cancellation waits for an actual request, and its latency excludes TLS startup. Its new unused-listener regression failed before the fix; all 16 download tests then passed. Hosted checks and fresh whole-product QA remain gates.
