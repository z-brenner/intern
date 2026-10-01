# Fixed SharePoint onboarding design

## Goal

Make Intern a setup-and-forget Windows application for nontechnical users. The normal user must not choose folders or enter Microsoft tenant, application, drive, or item identifiers. Intern targets one deployment:

- site: `https://teamcontoso.sharepoint.com/sites/InternTestSite`
- document library: `Files`
- watched folder: `Inbox`
- filed-document folder: `Filed`

Each installation processes only new documents whose SharePoint `createdBy` account matches the Microsoft account connected to that installation. Files belonging to another or an uncertain identity remain untouched.

## Trust and limitations

This version deliberately uses SharePoint drive-item metadata rather than Microsoft audit-log searches. The supported operating procedure requires each document to be newly uploaded directly into `Files/Inbox`. SharePoint `createdBy` is not proof of the actor who later copied or moved an existing item. A same-user copy or move can therefore be indistinguishable from a direct upload when every required identity, creation/modification, location, size, checksum, and revision fact still matches; accepting that case is a known limitation, not a claim that Intern proved the upload operation. Detectably copied or moved items, and edited, shortcut, conflicting, or otherwise ambiguous items, remain held.

The connected Microsoft account is authoritative. A typed name or email never authorizes a document. The UI may display the verified name and email returned by Microsoft, but it does not ask the user to create an identity.

Intern uses a bundled public-client application configuration and delegated, user-consentable Microsoft permissions. It stores refresh credentials in the operating-system credential store. If the tenant blocks user consent, onboarding explains that the organization has blocked the connection; Intern does not fall back to guessing ownership.

## Provisioned deployment configuration

The release contains a versioned public deployment resource with the site URL, library name, intake and destination names, public Microsoft application ID, tenant authority, and the SharePoint/OneDrive synchronization identifiers needed for the supported `odopen://sync` request. These are deployment constants, not settings.

The backend validates the resource at startup and confines Microsoft traffic and returned web URLs to the configured tenant/site/library. A malformed or incomplete deployment resource disables shared intake and presents a support-oriented error. No client secret is packaged.

Changing the target later requires a new deployment-configuration version and a new onboarding version. Existing protected intake roots remain protected; a migration must never silently repoint queued documents.

## Onboarding state and routing

A backend-owned, atomically written `ui-state.json` stores a monotonic `completedOnboardingVersion`. This state is separate from operational settings so a corrupt or invalid setting cannot cause onboarding to repeat or be dismissed accidentally. A future-state version is preserved rather than downgraded.

Onboarding appears when the stored version is below the application's current onboarding version. This covers fresh installs and presents the new workflow once to existing users after update. It is not keyed to every application release. Completion is stored only after the final readiness checks succeed; closing the window or encountering an error leaves onboarding resumable.

The route order is:

1. Welcome and plain-language explanation.
2. Local-model setup when the model is not ready.
3. Microsoft account connection.
4. SharePoint library synchronization and local-root detection.
5. Automatic configuration and readiness checks.
6. Finished summary.

Existing users retain queue/history/model data. Their previous manual folder settings are not overwritten until the fixed deployment is verified and they confirm the final activation step.

## User experience

### Welcome

The first page says what Intern does: watch the team's Inbox, privately read documents on this computer, rename eligible documents, and move them to Filed. It states that Intern processes only documents uploaded by the connected user and leaves all others alone. The primary action is **Set up Intern**.

### Local model

The current resumable, verified model download remains, but it becomes a step within onboarding. The primary path is the local model. Existing model files and hosted-model settings remain available under an advanced/support path rather than competing with the ordinary setup.

### Microsoft connection

The page has one action: **Connect Microsoft account**. It opens the supported device sign-in and polls safely as today. After connection it displays the verified name and email and asks the user to confirm that this is the account they use to upload documents. There are no tenant, client, drive, folder, identifier, or email fields.

### Library sync

Intern looks for an existing local sync root that can be verified as the configured `Files` library. Display names and path strings alone are never proof.

If no verified root exists, Intern launches the supported OneDrive `odopen://sync` enrollment for the fixed library, explains that OneDrive may show a confirmation, and waits while periodically rescanning registered sync roots. The page offers **Try again** and **Open SharePoint** recovery actions. If OneDrive is missing or signed into a different account, the error says exactly what the user must do.

If more than one candidate appears, Intern verifies the remote/local relationship and selects the single valid configured library. It does not ask the user to choose among arbitrary folders.

### Activation

After the sync root is verified, Intern resolves and validates local `Inbox` and `Filed` directories. Missing folders are not silently invented unless the remote configuration explicitly confirms they are expected and writable. Activation saves these derived paths and centrally chosen defaults:

- watched intake on;
- process other users' uploads off;
- private-local bypass off;
- run in background on;
- start at login on;
- start minimized on;
- destination layout and automatic-renaming policy retain their existing values;
- description-record policy retains its existing value until centrally decided.

The final page says: **Watching Files/Inbox. Filing your documents into Files/Filed.** It names the connected Microsoft account and explains that Intern starts automatically and remains available in the system tray.

## Per-document admission

Before extraction or any other content read, the admission guard:

1. maps the local path under the verified `Inbox` binding to the configured remote drive item;
2. fetches metadata through the connected account;
3. requires the exact configured tenant, site, library, folder boundary, filename, and item identity;
4. requires a file rather than a folder, shortcut, or conflict artifact;
5. requires `createdBy.user.id` to equal the connected account ID, with verified principal-name comparison only as a narrowly defined compatibility fallback;
6. requires `lastModifiedBy` to identify the same account and creation/modification facts to indicate a new, unchanged upload;
7. verifies remote size/checksum, local checksum, ETag/revision, and then rechecks metadata so a file cannot change between authorization and processing.

The same guard runs for watcher admission, manual import from the protected Inbox, retries, extraction, inference, and file application. A mismatch produces a held status, not a process-anyway button. Microsoft being unreachable is retryable and does not become a negative ownership verdict. Passing these checks does not distinguish a direct upload from every same-user copy or move, so users must upload documents directly into `Inbox`.

## Settings after onboarding

The normal Settings dialog shows a read-only **SharePoint connection** summary with site, library, Inbox, Filed, connected account, sync health, background/startup status, and a **Reconnect Microsoft** support action.

The current manual shared-intake controls, local folder pickers, machine label, process-others toggle, tenant/client fields, drive/folder fields, and local-only bypass are removed from the ordinary UI. Diagnostic details may remain behind an explicitly labelled support disclosure. Intern does not offer alternative SharePoint destinations in this deployment.

Advanced model settings, learned spellings, update controls, and help remain available.

## Failure handling

- Microsoft sign-in canceled or expired: remain on the connection step and retry.
- User consent blocked: explain that the Microsoft organization refused the connection and that Intern cannot safely watch the shared Inbox.
- Wrong Microsoft account or tenant: disconnect it and request the work account.
- OneDrive absent: link to install/open OneDrive; never claim sync is active.
- Sync enrollment delayed: continue rescanning without freezing the window; allow safe exit and resume on next launch.
- Inbox or Filed missing/unwritable: stop activation and identify the missing location.
- Metadata missing, identity mismatch, detectable copy/move/edit, conflict, or hydration pending: hold the document without reading it.
- Microsoft unavailable or throttled: keep the document pending and use bounded retry/backoff.
- Onboarding-state write failure: show retry and do not enter the main application under a false completed state.
- Start-at-login or background activation failure: report it on the readiness page and do not describe setup as complete.

All messages lead with a plain-language action. Stable error codes remain available for support and tests.

## Testing and release evidence

Backend tests cover deployment-resource parsing and URL confinement, onboarding-state migration/atomicity, `odopen` URL construction, sync-root verification, exact remote/local path mapping, account-ID matching, held states, metadata changes between checks, and every pipeline bypass boundary.

Frontend tests cover fresh install, legacy update, resume, model download, Microsoft device sign-in, wrong account, blocked consent, existing sync, enrollment and delayed detection, activation, completion persistence, read-only settings, accessibility, and plain-language errors.

Windows integration tests use a fake OneDrive/Graph boundary for deterministic CI. Before release, a clean Windows machine and two test accounts must prove:

- existing and missing-library-sync onboarding paths;
- only account A processes a fresh upload created by account A;
- account A holds account B's upload before content extraction;
- detectably copied or moved, edited, conflict, shortcut, unknown, offline, and changed-revision cases remain held;
- a same-user copy or move whose available metadata is identical to a direct upload is recorded as an accepted limitation of metadata-only verification;
- `Inbox` to `Filed` movement syncs successfully;
- restart, tray operation, start-at-login, update-from-alpha.9, reconnect, and uninstall preserve user documents;
- signed updater installation still succeeds.

No release claims live SharePoint compatibility until this tenant-backed acceptance run passes.

## Out of scope

- arbitrary SharePoint sites, libraries, or folder selection;
- processing documents uploaded by other people;
- tenant audit-log verification;
- copied, moved, or edited SharePoint items;
- silently bypassing Microsoft or OneDrive consent;
- direct cloud document processing without a local OneDrive sync;
- automatically creating SharePoint libraries, folders, columns, or Power Automate flows.
