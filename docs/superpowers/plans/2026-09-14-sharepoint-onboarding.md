# Fixed SharePoint Onboarding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Intern automatically connect a nontechnical Contoso user to `InternTestSite/Files`, watch only their fresh uploads in `Inbox`, and file them into `Filed` without exposing folder or Microsoft identifiers.

**Architecture:** A packaged deployment manifest confines the app to one SharePoint site/library and supplies public OAuth/sync identifiers. A backend onboarding service owns durable progress, Microsoft connection, OneDrive enrollment, verified local-path activation, and settings derivation; the existing admission guard is simplified from audit-event proof to strict fresh-item `createdBy`/`lastModifiedBy` metadata and revision proof.

**Tech Stack:** Rust 2024 workspace, Tauri 2, React 19, TypeScript 7, Vitest/Testing Library, Cargo tests, Windows OneDrive `odopen://sync`, Microsoft Graph v1.0.

**Spec:** `docs/superpowers/specs/2026-09-14-sharepoint-onboarding-design.md`

## Global Constraints

- Fixed target: `https://teamcontoso.sharepoint.com/sites/InternTestSite`, library `Files`, intake `Inbox`, destination `Filed`.
- The connected Microsoft account is authoritative; typed names/emails never authorize processing.
- Only new, unchanged uploads are supported; other/unknown/copied/moved/edited items remain held before content extraction.
- No client secret is packaged and credentials remain in the operating-system credential store.
- No arbitrary site/library/folder picker appears in the ordinary UI.
- Existing queue, history, model files, and settings survive upgrade.
- No live compatibility claim is made without a two-account tenant-backed acceptance run.

---

### Task 1: Versioned Deployment Manifest

**Files:**
- Create: `src-tauri/resources/sharepoint-deployment.json`
- Create: `crates/intern-intake/src/deployment.rs`
- Modify: `crates/intern-intake/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`
- Test: `crates/intern-intake/tests/deployment.rs`

**Interfaces:**
- Produces: `SharePointDeployment`, `SharePointDeployment::from_slice(&[u8])`, `SharePointDeployment::validate()`, `SharePointDeployment::contains_web_url(&Url)`, and `SharePointDeployment::odopen_url(&str)`.
- Consumes: `url::Url`, Serde, and packaged Tauri resources.

- [ ] **Step 1: Write failing manifest tests**

Cover exact target fields, HTTPS/site confinement, nonempty public tenant/client/site/web/list/drive/folder identifiers, fixed child names, percent-encoded `odopen` construction, rejection of secrets, lookalike hosts, path traversal, and URLs outside `/sites/InternTestSite`.

```rust
#[test]
fn fixed_manifest_builds_confined_sync_url() {
    let deployment = SharePointDeployment::from_slice(include_bytes!(
        "../../../src-tauri/resources/sharepoint-deployment.json"
    )).unwrap();
    let url = deployment.odopen_url("pat@contoso.com").unwrap();
    assert_eq!(url.scheme(), "odopen");
    assert!(url.as_str().contains("listTitle=Files"));
    assert!(deployment.contains_web_url(
        &Url::parse("https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox/a.pdf").unwrap()
    ));
}
```

- [ ] **Step 2: Run the test and verify RED**

Run: `cargo test -p intern-intake --test deployment`
Expected: FAIL because `deployment` is not exported.

- [ ] **Step 3: Implement parsing and validation**

Create a focused immutable type. Require schema version `1`, exact HTTPS origin/site path, simple `Inbox`/`Filed` components, GUID-shaped public identifiers, and no JSON keys containing `secret`, `password`, or `token`. Construct `odopen://sync` with URL query encoding rather than interpolation.

- [ ] **Step 4: Add the deployment resource to the bundle**

Add `resources/sharepoint-deployment.json` to the existing resource glob and populate administrator-resolved public IDs. If real IDs are not yet available, fail the build through the validation test rather than using placeholders.

- [ ] **Step 5: Run tests and commit**

Run: `cargo test -p intern-intake --test deployment && npm run assets:verify`
Expected: PASS.

Commit: `feat: add fixed SharePoint deployment contract`

### Task 2: Durable Onboarding State

**Files:**
- Create: `src-tauri/src/onboarding.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/lib/bridge.ts`
- Modify: `src/lib/tauriBridge.ts`
- Modify: `src/lib/inMemoryBridge.ts`
- Test: `src-tauri/src/onboarding.rs`
- Test: `src/lib/tauriBridge.test.ts`

**Interfaces:**
- Produces backend commands `onboarding_status`, `onboarding_activate`, `onboarding_complete`; DTO `OnboardingStatus { current_version, completed_version, phase, account, sync, error }`.
- Produces frontend methods `getOnboarding(): Promise<OnboardingStatus>`, `activateOnboarding(): Promise<OnboardingStatus>`, `completeOnboarding(): Promise<void>`.

- [ ] **Step 1: Write failing storage tests**

Test missing state as version zero, atomic round-trip, corrupt JSON as a visible error, preservation of future versions, failure without false completion, and version `1` as the current flow.

```rust
#[test]
fn future_completion_is_never_downgraded() {
    let store = OnboardingStore::new(temp.path().join("ui-state.json"));
    store.write_completed(9).unwrap();
    store.write_completed(CURRENT_ONBOARDING_VERSION).unwrap();
    assert_eq!(store.read().unwrap().completed_onboarding_version, 9);
}
```

- [ ] **Step 2: Run backend test and verify RED**

Run: `cargo test -p intern-app onboarding::tests`
Expected: FAIL because the module does not exist.

- [ ] **Step 3: Implement atomic state and commands**

Use a same-directory temporary file, `sync_all`, atomic replace, and a small `UiState { completed_onboarding_version: u32 }`. Initialize the store under `app_local_data_dir` and expose it through managed Tauri state.

- [ ] **Step 4: Write bridge mapping tests and verify RED**

Assert the exact command names and JSON-safe payloads in `src/lib/tauriBridge.test.ts`.

Run: `npm test -- src/lib/tauriBridge.test.ts`
Expected: FAIL because the bridge methods do not exist.

- [ ] **Step 5: Add shared TypeScript types and bridge implementations**

Define `OnboardingPhase`, `OnboardingAccount`, `OnboardingSyncStatus`, and `OnboardingStatus` in `src/types.ts`; implement production and in-memory bridges without importing Tauri into components.

- [ ] **Step 6: Run tests and commit**

Run: `cargo test -p intern-app onboarding::tests && npm test -- src/lib/tauriBridge.test.ts`
Expected: PASS.

Commit: `feat: persist versioned onboarding state`

### Task 3: User-Consent Microsoft Metadata Connection

**Files:**
- Modify: `crates/intern-intake/src/microsoft/auth.rs`
- Modify: `crates/intern-intake/src/microsoft/transport.rs`
- Modify: `crates/intern-intake/src/microsoft/proof.rs`
- Modify: `src-tauri/src/microsoft_intake.rs`
- Modify: `src/features/intake/microsoft.ts`
- Modify: `src/lib/tauriBridge.ts`
- Test: module tests in the four Rust files
- Test: `src/features/intake/microsoftSettings.test.tsx`

**Interfaces:**
- Consumes `SharePointDeployment` from Task 1.
- Produces `MicrosoftIntake::begin_managed()`, which uses packaged tenant/client configuration and requests only `User.Read`, `Files.Read`, and `offline_access`.
- Produces `FreshUploadProof::verify(account, drive_item, local_facts)` returning authorized/held/retryable decisions.

- [ ] **Step 1: Replace audit-flow expectations with failing metadata-proof tests**

Cover exact account-ID match, verified UPN fallback only when IDs are absent, other uploader, unknown identity, differing last modifier, unequal creation/modification facts, shortcut/remote item, wrong tenant/site/library/folder, changed ETag, checksum mismatch, and transient Graph failure.

```rust
#[test]
fn another_accounts_fresh_upload_is_held() {
    let result = FreshUploadProof::verify(&account("mine"), &item("other", "other"), &facts());
    assert_eq!(result.unwrap_err().code(), "UPLOADER_OTHER");
}
```

- [ ] **Step 2: Run Microsoft unit tests and verify RED**

Run: `cargo test -p intern-intake microsoft`
Expected: FAIL against the audit-event proof contract.

- [ ] **Step 3: Narrow OAuth scopes and endpoint allowlist**

Remove audit-query scopes and calls. Permit only tenant-specific device/token endpoints, `/me`, and exact configured drive-item metadata paths. Continue refusing content-download, arbitrary Graph hosts, query injection, pagination escapes, and client secrets.

- [ ] **Step 4: Implement fresh-upload proof**

Require `createdBy` and `lastModifiedBy` to resolve to the connected account; require fixed path, file facet, size, QuickXorHash, ETag, creation/modification invariants, and a second metadata read after local SHA-256 binding. Return held versus retryable outcomes without a process-anyway route.

- [ ] **Step 5: Remove identifier input from IPC/UI contracts**

Change sign-in start to accept no tenant/client input and no audit acknowledgment. Preserve disconnect/poll cancellation generation guards and Credential Manager storage keyed by the packaged public configuration.

- [ ] **Step 6: Run focused tests and commit**

Run: `cargo test -p intern-intake microsoft && cargo test -p intern-app microsoft_intake && npm test -- src/features/intake/microsoftSettings.test.tsx src/lib/tauriBridge.test.ts`
Expected: PASS.

Commit: `feat: verify fresh uploads with managed Microsoft identity`

### Task 4: OneDrive Enrollment and Fixed-Path Activation

**Files:**
- Create: `src-tauri/src/sharepoint_setup.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/microsoft_intake.rs`
- Modify: `crates/intern-intake/src/cloud.rs`
- Test: `src-tauri/src/sharepoint_setup.rs`
- Test: `crates/intern-intake/tests/cloud.rs`

**Interfaces:**
- Consumes `SharePointDeployment`, connected `Account`, `detect_cloud_roots()`, `SettingsStore`, and Microsoft metadata verification.
- Produces `SharePointSetup::status()`, `SharePointSetup::start_sync()`, and `SharePointSetup::activate()`.
- Produces commands `onboarding_start_sharepoint_sync` and `onboarding_activate`.

- [ ] **Step 1: Write failing enrollment/status tests**

Use injected opener, root detector, filesystem, metadata verifier, settings writer, and autostart boundary. Cover existing verified root, no root, multiple display-name matches, wrong remote site, missing OneDrive, encoded user email, missing/unwritable `Inbox` or `Filed`, and settings unchanged on partial failure.

```rust
#[test]
fn activation_is_atomic_after_both_fixed_folders_verify() {
    let result = rig().with_root(files_root()).with_folders(["Inbox", "Filed"]).activate();
    assert!(result.is_ok());
    assert_eq!(rig.saved().intake_folder, files_root().join("Inbox"));
}
```

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test -p intern-app sharepoint_setup`
Expected: FAIL because the module does not exist.

- [ ] **Step 3: Implement supported `odopen` handoff**

Open only the URL produced by `SharePointDeployment::odopen_url(account.email)`. Never write OneDrive policy registry keys. Report opener absence, missing OneDrive account, and enrollment pending distinctly.

- [ ] **Step 4: Implement verified root resolution**

Rescan registered roots, canonicalize component-wise, verify the candidate against configured remote library identity, then resolve sibling `Inbox` and `Filed`. Do not select using display name alone and do not expose a general folder picker.

- [ ] **Step 5: Implement atomic activation**

Derive settings from the previously saved settings while forcing intake enabled, other-upload processing off, local bypass off, background on, start-at-login on, and start minimized on. Validate both paths and autostart before saving; restore previous autostart state if persistence fails.

- [ ] **Step 6: Run focused tests and commit**

Run: `cargo test -p intern-intake --test cloud && cargo test -p intern-app sharepoint_setup commands::settings_tests`
Expected: PASS.

Commit: `feat: enroll and activate the fixed SharePoint library`

### Task 5: Guided Onboarding UI

**Files:**
- Create: `src/components/OnboardingFlow.tsx`
- Create: `src/features/onboarding/onboarding.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/SetupScreen.tsx`
- Modify: `src/styles.css`
- Modify: `src/types.ts`
- Modify: `src/lib/inMemoryBridge.ts`

**Interfaces:**
- Consumes Task 2/4 bridge methods and the existing setup download API.
- Produces an accessible route-level `OnboardingFlow` that owns welcome, model, Microsoft, sync, activation, and completed screens.

- [ ] **Step 1: Write failing UI tests**

Test legacy version zero showing once, current version skipping, welcome copy, model setup/resume, device code, connected identity confirmation, existing sync, `odopen` pending/rescan, wrong account, blocked consent, OneDrive unavailable, activation failure, completion persistence failure, successful summary, keyboard focus, and no identifier/folder fields.

```tsx
it('never asks a person for deployment identifiers', async () => {
  render(<App bridge={createInMemoryBridge({ onboardingVersion: 0 })} />);
  expect(screen.queryByLabelText(/tenant|client|drive|folder id|email/i)).not.toBeInTheDocument();
});
```

- [ ] **Step 2: Run UI tests and verify RED**

Run: `npm test -- src/features/onboarding/onboarding.test.tsx`
Expected: FAIL because the flow does not exist.

- [ ] **Step 3: Implement the state machine and route order**

Load settings, model setup, and onboarding concurrently, render an inert loading state until all gates settle, then route welcome → model → Microsoft → sync → activation → done. Persist completion only from the final successful action; closing/relaunching resumes.

- [ ] **Step 4: Implement plain-language recovery and accessibility**

Use one primary action per step, status live regions for polling, alert roles for failures, visible focus, deterministic focus movement on phase changes, minimum target sizes, and copy from the approved spec. Never freeze the window while polling.

- [ ] **Step 5: Integrate model setup without changing its backend guarantees**

Reuse existing start/cancel/resume progress behavior and keep hosted/existing-file alternatives in a clearly labeled advanced support disclosure.

- [ ] **Step 6: Run UI tests and commit**

Run: `npm test -- src/features/onboarding/onboarding.test.tsx src/features/setup/setup.test.tsx src/App.test.tsx && npm run lint`
Expected: PASS.

Commit: `feat: guide users through automatic SharePoint setup`

### Task 6: Simplify Settings and Preserve Support Diagnostics

**Files:**
- Modify: `src/components/SettingsDialog.tsx`
- Modify: `src/features/intake/MicrosoftIntakeSettings.tsx`
- Modify: `src/features/intake/microsoftSettings.test.tsx`
- Create: `src/features/settings/sharepointConnection.test.tsx`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes managed Microsoft/intake status and reconnect APIs.
- Produces read-only SharePoint connection summary and reconnect/rescan actions.

- [ ] **Step 1: Write failing settings tests**

Require site `InternTestSite`, library `Files`, `Inbox`, `Filed`, account, sync/watcher/background health, reconnect, and support detail. Assert absence of manual intake/destination selectors, tenant/client/drive/folder IDs, process-others toggle, machine label, and local-only bypass.

- [ ] **Step 2: Run tests and verify RED**

Run: `npm test -- src/features/settings/sharepointConnection.test.tsx src/features/intake/microsoftSettings.test.tsx`
Expected: FAIL against the current settings form.

- [ ] **Step 3: Implement the summary and support actions**

Replace ordinary shared-intake configuration with a status card. Keep precise account IDs, paths, last scan, held counts, hydration, conflicts, and stable error codes inside a collapsed **Support details** section. Reconnection cannot silently change to a non-Contoso tenant.

- [ ] **Step 4: Preserve unrelated settings**

Keep model, learned spelling, layout/renaming, description policy, updates, and help behavior. Ensure saving unrelated settings cannot disable the managed watcher/background policy.

- [ ] **Step 5: Run tests and commit**

Run: `npm test -- src/features/settings src/features/intake/microsoftSettings.test.tsx src/components && npm run lint`
Expected: PASS.

Commit: `feat: replace manual SharePoint settings with connection status`

### Task 7: End-to-End Admission and Upgrade Regression Coverage

**Files:**
- Modify: `crates/intern-queue/tests/pipeline.rs`
- Modify: `src-tauri/src/microsoft_intake.rs`
- Modify: `src/App.test.tsx`
- Modify: `tests/app.spec.ts`
- Modify: `README.md`
- Modify: `docs/microsoft-upload-verification.md`
- Modify: `docs/shared-intake.md`

**Interfaces:**
- Consumes all prior tasks.
- Produces regression evidence that no queue path reads an unauthorized file and that alpha.9 users onboard once without data loss.

- [ ] **Step 1: Add failing pipeline bypass tests**

For watcher admission, protected-folder manual add, retry, extraction, inference, apply, disconnect, account change, and revision change, assert the extractor/model/file action counters remain zero until the same connected account has a complete fresh-upload proof.

- [ ] **Step 2: Run pipeline tests and verify RED**

Run: `cargo test -p intern-queue --test pipeline microsoft -- --nocapture`
Expected: at least one new bypass assertion fails before integration fixes.

- [ ] **Step 3: Close integration gaps minimally**

Route every existing `AdmissionStage` through the metadata verifier, preserve retryable/held distinctions, and ensure protected roots survive onboarding retries, folder changes, disconnects, and update migration.

- [ ] **Step 4: Add upgrade and browser journey tests**

Seed alpha.9-shaped settings/queue state with no UI state; assert onboarding appears once, activation preserves queue/history/model and unrelated settings, completion survives remount, and current users go directly to the app. Exercise the complete browser fake journey without exposing technical fields.

- [ ] **Step 5: Rewrite user/admin documentation**

Document the fixed library, direct-upload-only limitation, Created By semantics, user-consent caveat, OneDrive prompt, held-file behavior, and the public deployment identifiers needed before producing an installer. Remove claims that audit-event verification ships in this mode.

- [ ] **Step 6: Run all automated verification**

Run:

```powershell
cargo fmt --all -- --check
cargo test --workspace
npm run check
npm run test:e2e
npm run assets:verify
git diff --check origin/main...HEAD
```

Expected: every command exits zero.

- [ ] **Step 7: Perform packaged Windows smoke testing**

Build the NSIS application using the repository's existing release/smoke scripts. Verify launch responsiveness, resumable model setup, sign-in cancellation, fake `odopen` enrollment, background/tray/start-at-login, updater handoff, and clean uninstall without deleting documents. Record any tenant-backed cases that remain unexecuted rather than claiming them.

- [ ] **Step 8: Commit and prepare PR**

Commit: `test: verify managed SharePoint onboarding end to end`

Push `feature/sharepoint-onboarding` and open a PR against `main` containing the design, plan, automated results, packaged-smoke evidence, security limitations, missing real public deployment IDs if any, and the explicit tenant-backed acceptance checklist.
