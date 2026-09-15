# SharePoint onboarding handoff — 2026-09-15

## Repository and branch state

- Review-clean merge boundary: `d69e0aad1fc67c85e9faa92b4d3fff56027918e7`.
- Preserved setup branch: `wip/sharepoint-exact-byte-snapshot`.
- Current WIP head: `93bb06051a185a571bdba42237c18c631c89e0af`.
- Task 4 initial setup commit: `c7087aa88a666a096311811e0b3002997af80c01`.
- The WIP head is buildable, but it is not approved for `main`; two Important review findings remain open.

The review-clean commits through `d69e0aa` may be merged into `main`. Keep `c7087aa` and `93bb060` on the WIP branch until the remaining activation findings below are fixed and independently re-reviewed.

## Product contract

Intern targets one fixed SharePoint deployment:

- site: `https://teamcontoso.sharepoint.com/sites/InternTestSite`
- library: `Files`
- intake: `Inbox`
- destination: `Filed`
- identity comes from Microsoft sign-in; users never type an authorizing email or deployment ID;
- only the connected user's post-activation, unchanged direct uploads are eligible;
- other, missing, malformed, stale, edited, shortcut, conflicting, or unverifiable items remain held before content extraction;
- missing local sync launches the deployment-generated OneDrive `odopen://sync` handoff;
- onboarding is intended to be simple and automatic for nontechnical users.

The user accepted that SharePoint metadata cannot reliably distinguish a same-user copy/move from a direct upload. Direct upload to `Files/Inbox` remains an operating requirement and must be stated in onboarding and documentation.

## Review-clean work through `d69e0aa`

1. Exact-byte binding: Microsoft verification streams the checked bytes into an app-private owned snapshot; queue extraction receives only that snapshot and fails closed without it. The snapshot retains an allowlisted extension for the real worker and is protected from replacement while owned.
2. Actor parsing: Microsoft object IDs use `Missing | Valid | Malformed`; only truly missing IDs permit exact-principal fallback. A valid matching object ID is authoritative.
3. Activation watermark: the successful fixed binding persists a watermark; legacy/missing/nonpositive watermarks fail closed, and only provider creation times strictly newer than activation can authorize.
4. Typed retry: transient Microsoft failures remain retryable through watcher and every queue stage without Needs Review noise, destructive state, or hot loops.
5. Graph confinement: production OAuth/Graph traffic is bound to the configured tenant, exact `/me` query, exact drive/Inbox binding item, one direct Inbox child, and matching SharePoint `webId`.
6. Enabled test seam: a test-only enabled Microsoft manager drives the real admission guard, SQLite queue, and extractor boundary through a source swap/restore and proves only snapshot bytes are read.

Verification at `d69e0aa`:

- `cargo test --workspace` passed;
- `cargo check --workspace --all-targets` passed;
- `cargo fmt --all -- --check` passed;
- `npm run check` passed (36 files / 236 tests plus production build);
- every security slice passed an independent scoped review.

A whole-range Codex Security diff scan reviewed all 19 changed production source files and produced zero reportable findings. It retained one deferred acceptance item: Graph exposes QuickXorHash rather than a provider-authenticated strong digest, so tenant-backed Windows testing must establish whether a lower-privilege or different-user attacker can ever keep authorized Graph metadata stable while substituting equal-length colliding local bytes. Do not claim this scenario is proven safe without that test.

## Task 4 WIP at `93bb060`

The branch contains an injectable `SharePointSetup` service, fixed-root resolution, deployment-generated `odopen`, no-payload Tauri/TypeScript bridge methods, managed settings derivation, and a staged Microsoft/settings/autostart activation transaction.

The first Task 4 review found three Important issues and one Minor issue:

1. Addressed in `93bb060`: remove the fallible post-publication binding confirmation. A successful atomic binding save/watermark is definitive, so a later status read cannot create an impossible rollback obligation.
2. Addressed in `93bb060`: fixed-binding activation now snapshots connection generation and exact tenant/object ID, detects reconnect races, and restores the staged Microsoft configuration when safe.
3. Still open: finish and prove the production live-runtime transaction. `activate_sharepoint_settings` now routes through the normal settings application path and replays previous settings on failure, but the implementation stopped before independent review. Verify watcher, tray, hosted runtime, intake events, persisted settings, and autostart are all restored together, including rollback failure reporting. The current unit test uses a synthetic closure and does not by itself prove real `AppState` behavior.
4. Still open: production OneDrive availability/opener taxonomy. Do not report every opener failure as `ONEDRIVE_MISSING`; distinguish missing account/client, unavailable protocol opener, and generic launch failure with production-boundary tests.

Current WIP verification performed after halt:

- `cargo fmt --all -- --check` passed after formatting;
- `cargo check -p intern-app --lib` passed without warnings;
- `cargo test -p intern-app sharepoint_setup --lib --quiet` passed 14/14;
- `git diff --check` passed before the WIP commit.

These focused results do not replace the full Task 4 suite and independent re-review after the two open findings are completed.

## Production blockers

- `src-tauri/resources/sharepoint-deployment.json` remains intentionally disabled because the real public tenant/client/site/web/list/drive/Inbox/Filed IDs are unavailable. Never enable it with invented GUIDs or a client secret.
- Current OneDrive registry discovery does not authoritatively prove that a local root maps to the configured SharePoint library. Production activation therefore returns `SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE` instead of trusting a display name or path. A supported Windows local-to-remote identity source or verified adapter is still required.
- Tenant policy may block delegated user consent. Surface that honestly; do not fall back to a typed email.
- Live two-account, OneDrive `odopen`, tray/autostart, updater, package, and uninstall acceptance runs remain outstanding.

## Remaining implementation order

1. Finish the two open Task 4 review findings on the WIP branch, rerun full Rust/frontend verification, and obtain a scoped re-review.
2. Implement `OnboardingFlow`: inert boot gate, Welcome, model setup, Microsoft sign-in/account confirmation, OneDrive sync, activation, and finished summary. Do not mount queue hooks/subscriptions until onboarding is complete.
3. Replace manual SharePoint settings with a read-only connection/health card and managed reconnect/rescan/support actions. Backend settings saves must continue enforcing the managed paths and flags; hiding controls is insufficient.
4. Add upgrade/browser/package regression coverage and run the tenant-backed acceptance checklist once real deployment identifiers and accounts exist.
5. Perform final whole-branch review and open a separate PR for Task 4/UI work.

## Resume on another computer

```powershell
git clone https://github.com/zgbrenner/intern.git
cd intern
git fetch origin
git switch wip/sharepoint-exact-byte-snapshot
git status --short
cargo test -p intern-app sharepoint_setup --lib --quiet
cargo check -p intern-app --lib
```

Read this handoff, then inspect the exact WIP delta with:

```powershell
git diff d69e0aa..HEAD
```

Do not enable the packaged deployment or merge the WIP setup commits until the remaining findings are fixed and reviewed.
