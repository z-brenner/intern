# Task 3 Security Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the reviewed Microsoft admission gaps while retaining the disabled production deployment and the accepted same-user copy/move limitation.

**Architecture:** The Microsoft proof creates a private verified-byte snapshot and returns it as owned admission evidence; the queue holds that evidence through extraction and gives the worker only the snapshot path. A persisted binding activation watermark rejects older items, typed retryability remains intact through proof, desktop, watcher, and every queue stage, and the concrete HTTP transport is deployment-bound. A test-only injected metadata seam exercises the real manager, admission guard, queue, and extractor without enabling the packaged production resource.

**Tech Stack:** Rust, SQLite-backed `intern-queue`, blocking Microsoft Graph client, Tauri desktop host, Vitest frontend.

**Spec:** `docs/superpowers/specs/2026-09-14-sharepoint-onboarding-design.md`

## Global Constraints

- Strict RED→GREEN TDD for every behavior change.
- No subagents.
- The packaged deployment stays disabled and all production Microsoft entry points fail closed with the exact deployment error.
- Same-user SharePoint copy/move may be indistinguishable from a direct upload and is an accepted procedural limitation; do not invent metadata proof.
- Never read the unverified source during extraction, call Graph content/audit endpoints, or accept webview identifiers.

---

### Task 1: Owned verified extraction snapshot

**Files:**
- Modify: `crates/intern-intake/src/microsoft/hashing.rs`
- Modify: `crates/intern-intake/src/microsoft/proof.rs`
- Modify: `crates/intern-queue/src/admission.rs`
- Modify: `crates/intern-queue/src/pipeline.rs`
- Test: `crates/intern-queue/tests/pipeline.rs`

**Interfaces:**
- Produces: `AdmissionEvidence`, carrying the optional verified hash and an owned private snapshot path whose drop removes it.
- Produces: `FreshUploadOutcome::Authorized { local_sha256, uploader, snapshot }`.

- [ ] Add a pipeline regression whose guard returns a verified snapshot, swaps/restores the public source around extraction, and whose real worker-path read must see only verified bytes.
- [ ] Run it and observe extraction currently reads the mutable source path.
- [ ] Change `AdmissionGuard::authorize` to return owned evidence; hold it through `WorkerBoundary::extract` and pass the snapshot path.
- [ ] Stream the same bytes into a private read-only snapshot while computing QuickXorHash/SHA-256; clean it on every drop/failure.
- [ ] Run the focused queue and proof tests to GREEN.

### Task 2: Persisted activation watermark and complete metadata

**Files:**
- Modify: `src-tauri/src/microsoft_intake.rs`
- Modify: `crates/intern-intake/src/microsoft/proof.rs`
- Test: `crates/intern-intake/tests/microsoft_fresh_upload.rs`
- Test: `src-tauri/src/microsoft_intake.rs`

**Interfaces:**
- Consumes: `verify_fresh_upload(..., activation_watermark, snapshot_directory)`.
- Produces: persisted `activation_watermark` established by a successful fixed-folder binding; missing watermark holds closed.

- [ ] Add a stale pre-activation metadata test and a restart/persistence test.
- [ ] Run them and observe old metadata/missing persisted state can authorize.
- [ ] Parse provider creation time to epoch and require it to be strictly newer than the persisted binding watermark.
- [ ] Save the watermark atomically with the fixed binding; require it during scope verification.
- [ ] Run focused tests to GREEN.

### Task 3: Typed actor parsing

**Files:**
- Modify: `crates/intern-intake/src/microsoft/proof.rs`
- Test: `crates/intern-intake/tests/microsoft_fresh_upload.rs`

**Interfaces:**
- Produces: internal `Missing | Valid(String) | Malformed` actor fields.

- [ ] Add malformed/null/blank/non-string/non-GUID ID cases proving only a truly missing ID permits principal fallback; add a valid matching ID with conflicting principal proving the ID remains authoritative.
- [ ] Run and observe malformed IDs currently fall back and principal conflicts currently override valid IDs.
- [ ] Implement tri-state parsing for ID, tenant, and principal fields and make the valid ID authoritative after tenant validation.
- [ ] Run the proof test target to GREEN.

### Task 4: Deployment-bound transport and web identity

**Files:**
- Modify: `crates/intern-intake/src/deployment.rs`
- Modify: `crates/intern-intake/src/microsoft/transport.rs`
- Modify: `crates/intern-intake/src/microsoft/auth.rs`
- Modify: `crates/intern-intake/src/microsoft/proof.rs`
- Modify: `src-tauri/src/microsoft_intake.rs`
- Test: corresponding Rust unit/integration tests.

**Interfaces:**
- Produces: `deployment_allows_endpoint(&SharePointDeployment, &Url, form, bearer)` and a concrete `MicrosoftTransport` constructed from validated deployment data.
- Produces: `SharePointDeployment::web_id()`.

- [ ] Add tests proving another drive, another by-ID item, nested path, and missing/wrong `sharepointIds.webId` are rejected while fixed Inbox binding/direct children pass.
- [ ] Run and observe current generic transport and proof accept them.
- [ ] Bind the concrete transport to deployment tenant/drive/Inbox identifiers; require `webId` in binding and per-file proof.
- [ ] Verify the exact `$select` contains every field consumed by production proof.
- [ ] Run focused auth/transport/proof/desktop tests to GREEN.

### Task 5: Typed retry through watcher and every queue stage

**Files:**
- Modify: `crates/intern-queue/src/pipeline.rs`
- Modify: `crates/intern-intake/src/scan.rs`
- Modify: `crates/intern-intake/src/watcher.rs`
- Modify: `src-tauri/src/intake.rs`
- Modify: `src-tauri/src/microsoft_intake.rs`
- Test: `crates/intern-queue/tests/pipeline.rs`
- Test: `crates/intern-intake/tests/watcher.rs`

**Interfaces:**
- Produces: `PipelineError::retryable(...)` plus `is_retryable()` and explicit `IntakeAdmission::Retryable`.

- [ ] Add tests for retryable admission at enqueue, extract, analyze, apply, and owned watcher recheck; assert no needs-review verdict, no content/action after the unavailable stage, and queued/ready state is retained for timed retry.
- [ ] Run and observe retryability is currently erased.
- [ ] Map `FreshUploadOutcome::RetryableUnavailable` to typed desktop errors; preserve it through host mapping and queue transitions.
- [ ] Run focused queue/watcher/desktop tests to GREEN.

### Task 6: Enabled injected-client end-to-end seam and documentation

**Files:**
- Modify: `src-tauri/src/microsoft_intake.rs`
- Modify: `docs/superpowers/specs/2026-09-14-sharepoint-onboarding-design.md`
- Modify: `docs/microsoft-upload-verification.md`
- Modify: `.superpowers/sdd/2026-09-14-sharepoint-onboarding/task-3-report.md`
- Test: `src-tauri/src/microsoft_intake.rs`

**Interfaces:**
- Produces: test-only enabled manager construction with injected `FreshUploadMetadata`, fixed binding, activation watermark, and snapshot root.

- [ ] Add an integration test driving injected metadata through the real manager/admission, real queue, and extractor-path boundary, including swap/restore.
- [ ] Run and observe no enabled injected seam exists.
- [ ] Add the narrow seam without changing `MicrosoftIntake::new`; keep production construction tied to the disabled bundled resource.
- [ ] Update docs/spec to require direct upload while explicitly stating same-user copy/move can be indistinguishable; preserve all other holds.
- [ ] Run all focused and full backend/frontend verification, formatting/checks, append the report, and commit.
