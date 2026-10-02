# SharePoint deployment: administrator guide

This guide is for the person who prepares an Intern build for an organization
SharePoint deployment. For the admission rules Intern applies to each document,
see [Microsoft upload verification](microsoft-upload-verification.md). For how
Intern proves which local folder is the synced library, see
[SharePoint root verification](sharepoint-root-verification.md).

## What the deployment is

Each Intern build supports exactly one SharePoint library, fixed at build time:

| Item | Value |
| --- | --- |
| Site | `https://teamcontoso.sharepoint.com/sites/InternTestSite` |
| Document library | `Files` |
| Watched folder | `Files/Inbox` |
| Destination folder | `Files/Filed` |

These values live in `src-tauri/resources/sharepoint-deployment.json`. They are
not settings. Users cannot change them, and Intern offers no other site,
library, or folder. The backend rejects any other site URL, library name, or
folder name when it loads the resource.

## Onboarding appears only in enabled builds

The resource in this repository has `"enabled": false`, and every identifier is
`null`, because no real identifiers have been supplied. In that build:

- There is no team Inbox onboarding. A first run that watches no folder is
  offered the simpler **folder setup** instead (see "Your own folder: three
  clicks" in the README): the person picks a OneDrive or SharePoint folder
  only they add documents to, and its documents are filed without a Microsoft
  uploader check (`intakeMyFolder`). Network shares are refused.
- Microsoft sign-in, pairing, and SharePoint setup all fail with
  `SHAREPOINT_DEPLOYMENT_UNAVAILABLE`.
- Settings will not save a OneDrive, SharePoint, or network folder typed in by
  hand as a watched intake; only folder setup marks a synced folder as the
  person's own.

In an enabled build, folder setup's "my folder" mode is refused
(`INTAKE_MY_FOLDER_UNAVAILABLE`) and activation clears it: only verified
uploads to the team Inbox are filed.

Guided onboarding is required only when both of these are true:

- the packaged deployment is enabled and valid; and
- the `completedOnboardingVersion` stored in `ui-state.json` (in Intern's
  local app-data folder) is lower than the current onboarding version, which
  is `1`.

So in an enabled build, both a fresh install and an update from 0.1.0-alpha.9
see onboarding once. After it finishes, Intern opens straight into the app.
Closing Intern partway leaves onboarding to resume on the next launch.

Never enable the resource with invented identifiers, and never add a client
secret. The loader refuses any field whose name contains `secret`,
`password`, or `token`.

## Identifiers an administrator must supply

To enable a build, set `"enabled": true` and fill in every identifier below.
All of them are public identifiers, not credentials.

| Field | What it is | Shape |
| --- | --- | --- |
| `tenant_id` | Your organization's Microsoft Entra tenant (directory) ID | GUID |
| `client_id` | Application (client) ID of the public client app registration | GUID |
| `site_id` | SharePoint site collection ID | GUID |
| `web_id` | SharePoint web (subsite) ID | GUID |
| `list_id` | List ID of the `Files` document library | GUID |
| `drive_id` | Microsoft Graph drive ID of the `Files` library | `b!` followed by 64 base64url characters |
| `intake_folder_id` | Graph driveItem ID of `Files/Inbox` | `01` followed by 32 upper-case base32 characters |
| `destination_folder_id` | Graph driveItem ID of `Files/Filed` | `01` followed by 32 upper-case base32 characters |

Accepting the `b!` and `01…` shapes depends on the backend change that
validates Graph drive and driveItem IDs by their real format. Earlier loaders
expected every identifier to be a GUID.

### App registration

Create the app registration in your organization's tenant, in Microsoft Entra admin
center > **App registrations** > **New registration**:

1. For **Supported account types**, choose this organizational directory only.
   Intern signs in against `https://login.microsoftonline.com/<tenant_id>`, not
   a multi-tenant endpoint.
2. Leave out any redirect URI. Intern uses the device-code flow.
3. Under **Authentication**, set **Allow public client flows** to **Yes**.
4. Under **API permissions**, add these delegated Microsoft Graph permissions:
   `User.Read`, `Files.Read`, and `offline_access`. Add nothing else.
5. Do not create a client secret or certificate.
6. Copy the **Application (client) ID** into `client_id`. Copy the
   **Directory (tenant) ID** into `tenant_id`.

### Graph Explorer lookups

Sign in to [Graph Explorer](https://developer.microsoft.com/graph/graph-explorer)
with an organization account that can open the site. Then run these `GET` requests.

1. **Tenant:** `https://graph.microsoft.com/v1.0/organization?$select=id`.
   The `id` is `tenant_id`.
2. **Site:** `https://graph.microsoft.com/v1.0/sites/teamcontoso.sharepoint.com:/sites/InternTestSite?$select=id,webUrl`.
   The `id` has the form `teamcontoso.sharepoint.com,<site GUID>,<web GUID>`.
   The first GUID is `site_id` and the second is `web_id`.
3. **Library drive:** `https://graph.microsoft.com/v1.0/sites/<full site id from step 2>/drives?$select=id,name,webUrl`.
   Find the drive whose `webUrl` ends in `/sites/InternTestSite/Files`. Its `id`
   (starting with `b!`) is `drive_id`. Intern checks returned web URLs against
   the path `/sites/InternTestSite/Files/Inbox`, so the library's URL segment
   must be `Files`, not only its display name.
4. **Library list:** `https://graph.microsoft.com/v1.0/drives/<drive_id>?$select=id,sharePointIds`.
   `sharePointIds.listId` is `list_id`. Also check that `sharePointIds.siteId`,
   `webId`, and `tenantId` match steps 1 and 2.
5. **Folders:** `https://graph.microsoft.com/v1.0/drives/<drive_id>/root:/Inbox?$select=id,name,folder`
   and `https://graph.microsoft.com/v1.0/drives/<drive_id>/root:/Filed?$select=id,name,folder`.
   The `id` values are `intake_folder_id` and `destination_folder_id`. Both
   folders must already exist. Intern never creates them.

## What users see

### Consent

Intern requests only delegated, user-consentable permissions. A user can
approve them at sign-in if tenant policy allows user consent. If the tenant
blocks user consent, or requires admin consent, sign-in fails and Intern does
not fall back to anything weaker. Onboarding then says:

> Ask your IT administrator to allow Intern to connect to Microsoft. Your
> organization blocked the connection, so Intern cannot safely watch the shared
> Inbox.

The support code shown is `MICROSOFT_CONSENT_BLOCKED`. The backend reports it
by adding the token `(MICROSOFT_CONSENT_BLOCKED)` to a sign-in error that
Microsoft Entra ID identifies as a consent block. Older backends reported only
"declined, expired, or blocked by organization policy". Intern shows that as a
general "connect again" message that also mentions the organization. To
prevent this, an administrator can grant tenant-wide admin consent for the
three permissions on the app registration.

### Sign-in and account confirmation

Users connect with a Microsoft device code. Intern then shows the verified name
and email from Microsoft and asks the user to confirm that this is the account
they upload with. No step asks for a tenant, client, site, drive, folder, path,
or email. An account from another tenant is reported, and the user is asked to
sign in with their work account.

Refresh credentials are stored in the Windows credential store, never in
settings files.

### The OneDrive prompt

If the `Files` library is not already synced on the computer, Intern asks
OneDrive to sync it through an `odopen://sync` link. Intern builds the link from
the deployment's site, web, and list IDs and the connected account's email.
OneDrive may ask the user to confirm; they should choose **Sync**. While
OneDrive works, Intern checks again every few seconds, and the user can close
Intern and finish later.

- If OneDrive is not installed, cannot receive sync links, or has no work
  account signed in, the step says what to do. It offers **Get OneDrive**
  (Microsoft's download page) where relevant, and does not offer another sync
  request, which would fail the same way.
- If OneDrive is signed in with a different work account than the Microsoft
  account connected to Intern, setup stops with `ONEDRIVE_ACCOUNT_MISMATCH`.
  The step asks the user to sign in to OneDrive with the same work account
  and then choose **Try again**. It also offers **Use a different account**,
  as a secondary choice, for when the account connected to Intern is the
  wrong one.
- If the connected Microsoft account belongs to another organization
  (`MICROSOFT_ACCOUNT_WRONG_TENANT`), the step offers **Use a different
  account** instead of a sync request.
- Other SharePoint or Teams libraries already synced on the computer do not
  block setup, and neither does another person's OneDrive account on the same
  computer syncing the same library. Until the `Files` library itself is
  synced, setup reports that enrollment is pending, and **Sync Files with
  OneDrive** stays available. It is also offered after a sync request that
  never reached OneDrive (`ONEDRIVE_OPEN_FAILED`, `SYNC_OPENER_UNAVAILABLE`).
  A check that lands while OneDrive is still adding the library keeps waiting
  rather than stopping.
- **Open SharePoint** opens the site, so the user can choose **Sync** on the
  library themselves.

### Local sync-record verification

A folder name or path is never taken as proof. Intern reads the OneDrive sync
client's own records for the candidate folder:

- the `libraryScope` lines in
  `%LOCALAPPDATA%\Microsoft\OneDrive\settings\Business<N>\<cid>.ini`;
- the matching `ScopeIdToMountPointPathCache` registry entry.

It requires exactly one record for that folder, with tenant, site, web, and
list IDs equal to the deployment's, held by the OneDrive account signed in as
the connected Microsoft account. Malformed, conflicting, or unreadable records
keep the library from verifying: `SHAREPOINT_ROOT_RECORD_MALFORMED`,
`SHAREPOINT_ROOT_RECORD_CONFLICT`, or `SHAREPOINT_ROOT_RECORD_UNAVAILABLE`.
While setup waits for the library, the onboarding sync step and the Settings
SharePoint card show that problem in plain language with its support code,
so the wait is never silent. **Turn on filing** reports the same code.

These record formats come from public sources and synthetic fixtures. No
team-site record has been observed yet. **A tenant-backed Windows run must
confirm them before the deployment is enabled.** The steps are in
[SharePoint root verification](sharepoint-root-verification.md#release-acceptance-required-before-enabling-the-deployment).

### Activation

Once the library is verified, `Files/Inbox` and `Files/Filed` must exist as
direct folders of the synced library and be writable. When the user chooses
**Turn on filing**, Intern:

- sets these values, and enforces them again on every later settings save:
  - watched intake on `Inbox`, destination `Filed`;
  - other people's uploads off;
  - private-local bypass off;
  - run in the background, start at sign-in, and start minimized all on;
- keeps every other setting unchanged, including destination layout,
  automatic renaming, description records, and the model choice;
- keeps the existing queue, history, and downloaded model.

If any step fails, Intern restores the previous settings and startup state.
The Settings dialog then shows a read-only **SharePoint connection**
card instead of the manual shared-intake controls. Manual folder pairing is
refused in an enabled build (`MICROSOFT_MANUAL_PAIRING_DISABLED`), and a
settings save that arrives while filing is being turned on is refused with
`SHAREPOINT_ACTIVATION_IN_PROGRESS` so it can be tried again a moment later
instead of being silently undone. Onboarding is recorded as finished only
while filing is on; otherwise completion is refused with
`ONBOARDING_SETUP_INCOMPLETE` and onboarding returns to **Turn on filing**.

### Reconnecting or switching accounts

Filing is turned on for one Microsoft account. If **Reconnect Microsoft** in
Settings signs in a different account from the same organization, Intern does
not carry the earlier activation over: setup returns to "ready to turn on",
and the SharePoint connection card offers **Turn on filing** for the new
account. Documents are filed for the new account only after that, and only
ones it uploads after that moment. Reconnecting the same account keeps filing
on. An account from another organization is disconnected and reported, as in
onboarding.

The card also offers **Sync Files with OneDrive** if the library is no longer
synced on the computer, so setup can be finished again without reinstalling
or re-running onboarding. It shows a OneDrive record problem that keeps the
library from verifying, with its code under **Support details**. After a sync
request fails in a way another request cannot fix, such as OneDrive missing,
the card offers **Check again** (and **Get OneDrive** where relevant) instead.

## Which documents are filed

Intern processes a document only when Microsoft's metadata shows that the
connected account created it, directly in `Files/Inbox`, after filing was
turned on, and that it has not changed since. The full checks are in
[Microsoft upload verification](microsoft-upload-verification.md).

- **Created By.** The account in SharePoint's **Created By** column is the
  account Microsoft Graph reports as `createdBy`, which Intern checks, and Intern also requires
  `lastModifiedBy` to be the same account. `createdBy` records who created the
  item, not who later copied or moved it.
- **Direct upload only.** Users must upload each new document directly into
  `Files/Inbox`. A copy or move of an existing item by the same user can look
  exactly like a direct upload in the metadata. Intern may then accept it.
  This is a known limitation of metadata-only checks, not proof of an upload.
  Detectable copies, moves, edits, shortcuts, and conflicts stay held.
- **Held documents** stay where they are in `Inbox`. Intern does not read,
  rename, move, or claim them. This covers documents from another account,
  from an unknown account, created before activation, or otherwise unverified.
  Held documents are counted under Settings > SharePoint connection > Support
  details, and Intern checks them again on later scans. If Microsoft cannot be
  reached, the document waits and is retried; that is not treated as a
  verdict.

Intern does not use Microsoft audit-log searches. Earlier documentation that
described audit-event verification does not apply to this release.

## Tenant-backed acceptance checklist

The automated tests use fake OneDrive and Graph boundaries. Before any release
claims live SharePoint compatibility, run these on a clean Windows machine with
two test accounts (A and B):

- [ ] Onboarding works when the library is already synced, and when it is not
      synced yet and OneDrive must enroll it, including on a computer that
      already syncs another SharePoint or Teams library.
- [ ] OneDrive signed in as account B while Intern is connected as account A
      reports `ONEDRIVE_ACCOUNT_MISMATCH`.
- [ ] With OneDrive signed in to both A and B on the same Windows profile, and
      both syncing the library, A's setup verifies A's folder and reports no
      `SHAREPOINT_ROOT_RECORD_CONFLICT`.
- [ ] After activation as A, reconnecting Microsoft as B shows **Turn on
      filing**, and B's uploads made before B turns filing on stay held.
- [ ] Only account A processes a fresh upload created by account A.
- [ ] Account A holds account B's upload before any content extraction.
- [ ] Detectably copied or moved, edited, conflict, shortcut, unknown, offline,
      and changed-revision items stay held.
- [ ] A same-user copy or move whose available metadata matches a direct upload
      is recorded as an accepted limitation of metadata-only verification.
- [ ] Moving documents from `Inbox` to `Filed` syncs successfully.
- [ ] Restart, tray operation, start at sign-in, update from 0.1.0-alpha.9,
      reconnect, and uninstall all leave user documents intact.
- [ ] Signed updater installation still succeeds.
- [ ] Every step in the root-verification release acceptance passes.
- [ ] Local QuickXorHash results match Microsoft-reported `quickXorHash` for
      empty, small, chunk-boundary, and large files in the library.
- [ ] Blocked user consent shows the organization-blocked explanation.
