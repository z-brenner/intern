# SharePoint onboarding handoff — 2026-09-16

Supersedes `2026-09-15-sharepoint-onboarding.md`.

## State

Branch `feature/sharepoint-onboarding-ui` completes plan Tasks 4–7 in code:

- Task 4: both open review findings closed. Activation and rollback run through a
  `SettingsRuntime` seam and are tested against the real settings, Microsoft, and
  setup adapters. Opener failures are classified as `ONEDRIVE_MISSING`,
  `ONEDRIVE_ACCOUNT_MISSING`, `ONEDRIVE_ACCOUNT_MISMATCH`,
  `SYNC_PROTOCOL_UNAVAILABLE`, `SYNC_OPENER_UNAVAILABLE`, or `ONEDRIVE_OPEN_FAILED`.
- Production root verification is no longer unavailable. It reads OneDrive's own
  per-account records and matches them to the connected account; see
  `docs/sharepoint-root-verification.md`.
- Task 5: guided onboarding (`src/components/OnboardingFlow.tsx`). It appears only
  when the packaged deployment is enabled.
- Task 6: read-only SharePoint connection card in Settings, which can also finish
  sync or activation. Managed paths and flags are enforced on every backend save.
- Task 7: pipeline bypass coverage, the alpha.9 upgrade journey, the Playwright fake
  journey (`?sharePoint=fake`, dev server only), and user and administrator docs
  (`docs/sharepoint-deployment.md`).
- Hardening from two independent reviews:
  - The fixed binding records the activating account.
  - Manual pairing is refused.
  - Completion requires setup to be Active.
  - Saves that race an activation are refused.
  - Other synced libraries no longer block enrollment.
  - Real Graph `b!` drive IDs and `01…` item IDs are required and compared exactly.

Automated gates at merge: `cargo fmt`, workspace clippy on 1.88.0 (restored to green;
it had failed on main since #38), `cargo test --workspace`, `npm run check`,
Playwright, and `assets:verify`.

## Still blocked on people

- `src-tauri/resources/sharepoint-deployment.json` stays disabled. It needs the
  identifiers listed in `docs/sharepoint-deployment.md` from the Contoso tenant.
  Enabling it requires those real values; the packaged-manifest test fails CI on
  invalid ones.
- Main already removed alpha.9's manual tenant/client Microsoft connection (#38).
  Until a tenant-configured build ships, verified shared intake is unavailable.
- Tenant-backed acceptance is still outstanding:
  - the OneDrive `libraryScope`/`UserEmail` record format on a real team-site sync;
  - a UPN rename;
  - two OneDrive accounts syncing the same library;
  - the QuickXorHash check;
  - the two-account held/admitted cases;
  - the `odopen` prompt;
  - the tray, autostart, update, and uninstall checklist in the spec.
