# Task 4 OneDrive enrollment and fixed-path activation progress

## Implementation

- Status: implemented and verified on the final working-tree revision.
- RED: `cargo test -p intern-app sharepoint_setup` failed with missing setup types/traits, and the focused cloud test failed because `paths_overlap` was not exported. The Microsoft binding transaction test then failed because `activate_fixed_binding` and `fixed_binding_active` did not exist. The bridge test failed because `startSharePointSync` did not exist.
- GREEN: the injectable setup service covers 14 enrollment, root-resolution, fixed-child, activation, and rollback behaviors; the cloud overlap regression, Microsoft protected-stage/binding/watermark transaction, and 20 bridge tests pass. A final transaction-ordering regression first demonstrated that the callback could observe an active binding, then passed after binding publication was moved after the local commit.
- Enrollment: the only URL accepted by the opener boundary is constructed by `SharePointDeployment::odopen_url` from the connected account email. Missing connected account, missing OneDrive, missing OneDrive account, missing opener, open failure, and enrollment-pending states have separate stable codes. No OneDrive policy or registry write exists.
- Root authorization: registry roots are discovery candidates only. Activation requires one injected verifier result whose canonical local root and tenant, site, web, list, and drive identifiers all exactly match the packaged deployment. Wrong, ambiguous, and overlapping/nested roots fail closed; no display name or path string authorizes activation.
- Fixed paths: only canonical direct children `Files/Inbox` and `Files/Filed` are accepted. Missing, non-directory, escaped, overlapping, or unwritable paths block activation before settings persistence.
- Recoverable activation: settings are derived from the previous complete value and preserve unrelated fields. Managed values force intake/background/startup on and process-others/local-bypass off. Autostart is applied before the atomic settings write and restored on failure. The Microsoft binding is protected and durably watermarked around the local commit; commit failure restores the previous Microsoft/settings/autostart state, while rollback failure reports an explicit incomplete state and leaves admission fail-closed.
- IPC: `onboarding_start_sharepoint_sync` and `onboarding_activate` take no payload and run blocking root/filesystem/OneDrive work on Tauri's blocking pool. Their JSON DTO exposes only the fixed site/library/folder labels and account display name/email, never local paths or Microsoft/SharePoint identifiers.
- Production limitation: `resources/sharepoint-deployment.json` remains disabled. Production commands therefore return `SHAREPOINT_DEPLOYMENT_UNAVAILABLE` with the packaged deployment error. Even with enabled identifiers, the current production root verifier returns `SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE`; registry display names cannot auto-activate the library.

## Deferred security and tenant evidence

- A separate security scan is deferred; this task did not enable deployment or widen the existing Microsoft transport/content boundary.
- Tenant-backed follow-up remains mandatory: validate the local QuickXor implementation against Microsoft-reported `quickXorHash` for representative empty/small/chunk-boundary/large files in the provisioned library, then run the two-account fresh-upload/held-upload and Inbox-to-Filed acceptance cases. No live SharePoint compatibility claim is made until those tests pass.

## Verification

- `cargo test -p intern-app microsoft_intake::tests::setup_binding_is_published_with_a_watermark_and_rolled_back_if_local_commit_fails`: PASS (1/1).
- `cargo test --workspace --quiet`: PASS (all suites; 1 pre-existing ignored test).
- `npm run check`: PASS (TypeScript compile, 36 files / 237 Vitest tests, production Vite build).
- `cargo check --workspace --all-targets`: PASS.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy -p intern-app --lib --no-deps -- -D warnings`: PASS.
- `cargo clippy -p intern-app --tests --no-deps -- -D warnings`: PASS.
- `cargo clippy -p intern-intake --lib --no-deps -- -D warnings`: PASS.
- `git diff --check`: PASS.
- Full-workspace clippy remains outside this slice because the baseline has unrelated pre-existing warnings in `intern-core/src/snapshot.rs`, `intern-worker/src/sheet.rs`, and the all-targets path in `intern-intake/src/microsoft/auth.rs`; targeted changed-package checks above are clean.
