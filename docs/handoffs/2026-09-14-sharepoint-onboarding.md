# SharePoint onboarding handoff — 2026-09-14

## Repository state

- Safe, buildable feature branch: `feature/sharepoint-onboarding`
- Last safe commit: `4af5f97` (`feat: verify fresh uploads with managed Microsoft identity`)
- Preserved unfinished branch: `wip/sharepoint-exact-byte-snapshot`
- Unfinished snapshot commit: `5c1593a` (`wip: preserve exact-byte snapshot hardening`)
- Approved design: `docs/superpowers/specs/2026-09-14-sharepoint-onboarding-design.md`
- Implementation plan: `docs/superpowers/plans/2026-09-14-sharepoint-onboarding.md`

The WIP commit is intentionally not merged into the safe branch. It does not compile yet. Continue it in a separate worktree or cherry-pick it onto a new branch based on current `main`.

## Product decisions

Intern targets one fixed deployment:

- site: `https://teamcontoso.sharepoint.com/sites/InternTestSite`
- library: `Files`
- intake: `Inbox`
- destination: `Filed`
- one shared Inbox, processing only files attributed by Microsoft to the connected user;
- identity comes from Microsoft sign-in, never a typed name/email;
- users upload new files directly into Inbox;
- missing library sync uses Microsoft's `odopen://sync` fallback;
- onboarding appears on fresh installs and once to existing users after this update.

The user accepted the reduced-assurance metadata approach because tenant audit permissions require administrator consent. SharePoint `createdBy` cannot distinguish every same-user copy/move from a direct upload. Direct upload is a required operating rule, not a cryptographically provable condition.

## Completed work on the safe branch

1. Deployment contract (`4f8bbc7`, `c706e82`): exact site/library/folder confinement, brace-correct structured `odopen` URLs, secret/lookalike rejection, and a deliberately disabled packaged deployment.
2. Durable onboarding state (`80878e0`, `40e7fad`, `a4a4084`, `ac84b47`): backend `ui-state.json`, upgrade versioning, future-version preservation, Windows durable replacement, explicit uncertain-durability retry, bounded reads, cleanup, and bridge contracts.
3. Managed Microsoft metadata connector (`4af5f97`): no identifier inputs, scopes narrowed to `User.Read Files.Read offline_access`, active audit endpoints removed, OS credential storage retained, and initial fixed-site upload proof.

At `4af5f97`: `cargo test -p intern-intake` passed 142 tests; `cargo test -p intern-app microsoft --lib` passed 7 tests with ignored sidecar staging; `npm test` passed 36 files/236 tests; production frontend build, Cargo check, formatting, and diff checks passed. Full workspace/package testing needs the repository's runtime-asset staging because a fresh checkout lacks packaged sidecars.

## Security findings still open

Do not enable the deployment until all are resolved and reviewed:

1. Exact-byte binding: verification returns a hash, then extraction reopens the public path. Create a private owned snapshot from the exact checked bytes, carry ownership through proof/admission, extract only it, fail closed without it, and clean it safely.
2. Freshness watermark: persist binding activation time, reject missing/pre-activation timestamps, and preserve it across restart. Same-user post-activation copy/move remains an explicit procedural limitation.
3. Retryable outages: preserve a typed retryable result through admission. Enqueue retries before fingerprinting; Extract/Analyze requeue cleanly without review/failure; Apply preserves Ready/Review and proposal; avoid hot loops.
4. Actor parsing: distinguish Missing/Valid/Malformed; fallback only for truly missing IDs; valid object ID is authoritative; malformed identity holds unknown.
5. Graph confinement: bind OAuth to the configured tenant; allow only `/me` and exact configured drive/Inbox metadata; validate `sharepointIds.webId`; reject alternate paths/authorities.
6. Enabled integration seam: test an enabled injected deployment through `MicrosoftIntake`, the real queue, and extractor boundary while retaining disabled-production tests.

Detailed evidence is in `.superpowers/sdd/2026-09-14-sharepoint-onboarding/` locally. The WIP branch contains `docs/superpowers/plans/2026-09-14-task-3-security-review.md`.

## WIP snapshot branch

Commit `5c1593a` contains partial work in `crates/intern-core/src/snapshot.rs`, its tests, queue admission/pipeline code and tests, and fresh-upload tests. When halted:

- core private-snapshot/RAII tests existed;
- proof test was RED because `FreshUploadOutcome::Authorized` lacked a snapshot;
- queue tests covered missing-snapshot fail-closed and swap/restore;
- hashing still returned only a digest;
- host/intake adapters still used the old result;
- the branch did not compile end-to-end.

Finish only exact-byte snapshot binding first, make focused suites green, commit, and obtain scoped security re-review before the other five findings.

## Remaining product work

After Task 3 is review-clean:

1. Add `sharepoint_setup` for deployment-generated `odopen`, nonblocking root rescans, fixed Inbox/Filed resolution, and atomic watcher/background/autostart activation.
2. Resolve sync-root trust. Current registry discovery exposes a local path/display name, not authoritative site/list/drive identity. Either derive stable OneDrive registration identity or treat the root as tentative and rely on per-file proof before content reads.
3. Build `OnboardingFlow`: Welcome → model → Microsoft → OneDrive sync → activation → finished; one primary action per page; no editable email/IDs; queue mounts only after completion.
4. Replace manual SharePoint Settings with a read-only connection/health summary and reconnect/support actions.
5. Add upgrade, two-account, Windows package, tray/autostart, updater, and uninstall evidence.

## Missing deployment values

`src-tauri/resources/sharepoint-deployment.json` is disabled. A working release needs real public Microsoft application and SharePoint/OneDrive identifiers. Never enable it with test GUIDs. Contoso tenant policy may still block delegated user consent; report that instead of falling back to typed identity.

## Resume

```powershell
git fetch origin
git switch wip/sharepoint-exact-byte-snapshot
git status --short
cargo test -p intern-core --test owned_snapshot
cargo test -p intern-intake --test microsoft_fresh_upload
cargo test -p intern-queue --test pipeline extraction_reads_the_owned_verified_snapshot_across_a_source_swap_and_restore
```

After the WIP becomes coherent and reviewed, rebase it on current `main` and merge via a separate PR. Do not enable SharePoint in the same commit.
