# SharePoint root verification

Activation needs proof that one local folder is the provisioned SharePoint
`Files` library. `detect_cloud_roots` only finds candidate folders, using the
`Tenants` registry values and display names, and those never authorize
activation. This document records what the OneDrive sync client stores on
Windows, how trustworthy each record is, and which records the production
verifier (`OneDriveRecordVerifier`) relies on.

Confidence labels:

- **Documented**: Microsoft publishes it.
- **Corroborated**: several independent public sources agree, and it matches
  what we saw on this machine.
- **Observed**: seen on this development machine only. The machine has a
  Personal OneDrive account and no work or school (Business) data.
- **Inferred**: our reasoning, not seen directly.

No team-site record has been observed on this machine. **Before release, a
tenant-backed Windows run must confirm the record format** (see "Release
acceptance" below).

## Sources considered

### 1. `%LOCALAPPDATA%\Microsoft\OneDrive\settings\Business<N>\<cid>.ini`, `libraryScope` lines (chosen)

- Each signed-in account has a settings folder: `Personal` or `Business1` to
  `Business9`. Its `global.ini` holds `cid = <id>`. The file `<cid>.ini` holds
  one `libraryScope = …` line for each synced library, plus `libraryFolder`
  lines for synced subfolders and `AddedScope` lines for shortcuts.
  **Corroborated** by [guwidoe's GetLocalPath gist][gist], by the maintained
  parser in [cristianbuse/VBA-FileTools `LibFileTools.bas`][vbaft], by
  [VBA-FileTools issue #1][issue1], and by [a folder-rename guide][schreiner].
  **Observed** here: `settings\Personal\global.ini` and
  `settings\Personal\<cid>.ini`.
- The files are UTF-16LE. The observed files have no byte-order mark.
  **Corroborated** by the gist ("All of the `.ini` files … use UTF-16
  encoding") and **observed**.
- Fields are separated by spaces. A quoted field runs to the next `"`. Field
  positions after `libraryScope =`, counting from 0:

  | # | Meaning | Evidence |
  |---|---------|----------|
  | 0 | library number (`0` = the account's own OneDrive) | Corroborated (`parts(2)` in both parsers) |
  | 1 | scope ID, 32 hex digits, optionally followed by `+<number>` | Corroborated; observed equal to the `ScopeIdToMountPointPathCache` value name and the SyncRootManager key suffix |
  | 3, 4 | site title, library title | Corroborated (issue #1 example); **never used for authorization** |
  | 6 | web URL | Corroborated; **never used for authorization** |
  | 7 | tenant ID, dashed GUID | Observed (Personal record holds the consumer tenant; the team-site example in issue #1 holds a dashed GUID in the same position). Matches `tenantID` in `SyncEngineDatabase.db` (observed). Inferred for team sites; must be confirmed on a tenant. |
  | 8, 9, 10 | site ID, web ID, list ID, 32 hex digits without dashes | Corroborated (`siteID = parts(10): webID = parts(11): libID = parts(12)`). Observed equal to `SiteID`, `WebID`, and `IrmLibraryId` in `ClientPolicy.ini` and to `siteID`, `webID`, and `listID` in `od_ScopeInfo_Records` |
  | 12 | local mount point, quoted; empty when only subfolders of the library are synced | Corroborated (`tempMount = parts(14)`; issue #1 shows an empty mount alongside a `libraryFolder` line). Observed. |

  The team-site example from [issue #1][issue1], which has invented IDs and
  an empty mount because only a subfolder was synced:

  ```text
  libraryScope = 1 5404014da42949c3af2bf558970233a6+1 5 "TestLib3rdLvlMount" "Dokumente" 4 "https://companyname.sharepoint.com/sites/TestLib" "dffdfdd4-77f5-445a-9dc0-1f5c6d259395" 9d85bcc70f964867ab531b8a918d89b5 2e351aec44184b348acb2a8fef7da70f 223717ee7bb3432ca991b96d3da29d7f 0 "" 1 00000000-0000-0000-0000-000000000000  - 0 0 00000000-0000-0000-0000-000000000000
  ```

  The scope-ID field in that Business example has a `+1` suffix. The observed
  Personal record uses a bare 32-hex scope ID, and VBA-FileTools strips the
  `+N` part before looking IDs up (`Split(tempID, "+")(0)`). The verifier
  accepts `<32 hex>` or `<32 hex>+<digits>` and compares only the 32-hex part.
  The suffix is not an identifier used for authorization; it only links this
  record to the registry scope cache. **Inferred**; see "Release acceptance".
- The layout has changed over time. Parsers handle both
  `ClientPolicy_<list><site>.ini` and `ClientPolicy_<list>_<site>.ini`, and
  version 23.184 replaced `<cid>.dat` with `SyncEngineDatabase.db`. Our parser
  is strict about each line and never guesses at a field, but it is lenient
  about lines it cannot parse: such a line is set aside, and it blocks
  verification (`SHAREPOINT_ROOT_RECORD_MALFORMED`) only when it could be the
  candidate's own record. See "Verification rules" below.

### 2. `HKCU\Software\Microsoft\OneDrive\Accounts\Business<N>\ScopeIdToMountPointPathCache` (chosen, as corroboration)

- Each REG_SZ value is named by a scope ID and holds that scope's local path.
  **Corroborated**: the [Metasploit `enum_onedrive` module][msf] enumerates
  these values and joins them to `SyncEngines` provider keys, and a
  [Microsoft Q&A thread][qa-dup] names the key during duplicate-library
  cleanup. **Observed** here for Personal:
  `8e7d…4fc5 = C:\Users\<user>\OneDrive`. The same scope ID appears in
  `<cid>.ini` and in the SyncRootManager key name.
- The registry key and the `.ini` file are separate writes by the sync
  engine. Requiring both to agree defends against a stale `libraryScope` line
  left after the user stops syncing. This is **inferred**; we have not
  observed how or when either record is removed.

### 3. `HKCU\Software\SyncEngines\Providers\OneDrive\<key>` (not used)

- Values are `MountPoint`, `UrlNamespace`, `LibraryType` (`personal`,
  `mysite`, or `teamsite`), `LastModifiedTime`, and sometimes `CID`.
  **Corroborated** by [Metasploit][msf], [CloudPilot][cloudpilot], and the
  [xlwings issue][xlwings]. **Observed** for Personal.
- There is no tenant, site, web, or list GUID; only a URL namespace.
  Authorizing on it would be URL-substring authorization, which is
  forbidden. The gist also documents cases where these keys are
  [wrong][gist] (for example, a synced folder named `Personal`).

### 4. `ClientPolicy_*.ini` (not used)

- Holds `SiteID = {GUID}`, `WebID = {GUID}`, `IrmLibraryId = {GUID}` (the list
  ID), `DavUrlNamespace`, and `LibraryTitle`. **Corroborated** and
  **observed**.
- It holds no local path, so it cannot bind a folder. Everything it could add
  (the same IDs) is already on the `libraryScope` line. Adding a third
  dependency on a file name whose format has changed would only add ways to
  fail.

### 5. `SyncEngineDatabase.db`, table `od_ScopeInfo_Records` (not used; possible future upgrade)

- Columns include `scopeID`, `siteID`, `webID`, `listID`, `tenantID`,
  `webURL`, `libraryType`, and `lastKnownFolderPath`. **Corroborated** by
  [OneDriveExplorer's parser][ode] and a [forensic research note][issen].
  **Observed** on a copy of this machine's Personal database: the values
  match `<cid>.ini` exactly.
- This is the richest structured source. Reading it needs a SQLite
  dependency and a live WAL database that OneDrive keeps open. Its schema is
  also undocumented and has changed: OneDriveExplorer queries two column sets.
  The `.ini` record carries the same binding facts, so we chose the
  dependency-free text record. If the `.ini` lines disappear in a future
  client, switch to this table behind the same `OneDriveRecords` boundary.

### 6. `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SyncRootManager\OneDrive!<SID>!Business<N>|<scopeId>` (not used)

- A Cloud Files sync-root registration. The ID has the form
  `[Storage Provider ID]![Windows SID]![Account ID]`
  (**Documented**: [StorageProviderSyncRootInfo.Id][syncrootid]).
  `UserSyncRoots\<SID>` holds the local path, and the key name ends in the
  scope ID. **Observed** for Personal.
- There are no SharePoint IDs. The key lives under HKLM, but the user's own
  sync engine registers it through the Cloud Files API ([Build a cloud sync
  engine][cfapi]). It is not written by an administrator, so it is no more
  trustworthy against same-user code than the HKCU records. It could only
  confirm the scope ID → path pair that source 2 already provides.

## Trust model

- **Who can write the sources.** The chosen sources are under
  `%LOCALAPPDATA%` and `HKCU`. By default their ACLs grant access to the
  user, SYSTEM, and Administrators. Low-integrity or AppContainer processes
  cannot write `%LOCALAPPDATA%\Microsoft\OneDrive` or `HKCU\Software` outside
  the `*Low` locations. A different standard user cannot write another user's
  profile. (**Inferred** from default Windows profile ACLs and mandatory
  integrity policy.) One caveat was **observed** here: developer tooling had
  added extra Modify ACEs to `%LOCALAPPDATA%`. Machines under managed
  deployment should keep the default profile ACL.
- **Same-user code is already trusted.** It can rewrite Intern's settings,
  its OS credential store entries, and its per-user install. Protecting the
  root mapping from it would add nothing. **Given the stated threat model
  (a lower-privilege or different-user attacker), this is adequate.**
- **Damage from a spoofed mapping is limited.** Every document is still
  admitted only through Microsoft Graph proof (see
  [`microsoft-upload-verification.md`](microsoft-upload-verification.md)).
  That proof checks the fixed drive and Inbox IDs, tenant, site, web, and
  list, `/me` as creator and modifier, and QuickXorHash and size equality
  between the local bytes and the remote item. A wrong mapping can make
  Intern watch the wrong local folder. Its files will not match Graph items
  in the fixed Inbox, so they are held, not processed.

## Graph cross-check decision

We did not widen Graph confinement.

- Graph has no local paths, so it cannot strengthen the local-folder →
  library link. That link is exactly what the local records provide.
- The one fact Graph could add is the drive-to-list binding. For example,
  `GET /v1.0/drives/{drive}?$select=id,sharePointIds` would return the list,
  site, and web behind the packaged drive. That binding is already enforced
  on every admitted item: `parentReference.driveId` must equal the packaged
  drive, and `sharepointIds.{tenantId,siteId,webId,listId}` must equal the
  packaged IDs in the same response. A setup-time call would only repeat
  that check, and it would need a new allowlisted endpoint.
- The sync records have no Graph drive ID, so the verifier returns none: the
  local identity (`RemoteLibraryIdentity`) is tenant, site, web, and list
  only. The packaged drive is proven per item by Graph at admission
  (`parentReference.driveId` together with `sharepointIds`). That is explicit
  in `src-tauri/src/sharepoint_root_verifier.rs`.

## Verification rules (implemented)

### Record checks

`intern_intake::onedrive_identity::verify_library_root` decides for one
candidate folder. The candidate has already been canonicalized by the setup
service. The function takes the connected Microsoft account's mail and user
principal name, plus the setup service's canonicalizer.

1. **Deployment IDs.** The deployment's tenant, site, web, and list are
   normalized to 32 lowercase hex digits. Dashed, braced, and bare GUIDs are
   accepted. A deployment ID that cannot be normalized returns
   `SHAREPOINT_ROOT_RECORD_MALFORMED`.
2. **Accounts.** Only `Business1` to `Business9` are read.
   - The settings folder not existing means no accounts.
   - A settings folder that cannot be listed returns
     `SHAREPOINT_ROOT_RECORD_UNAVAILABLE`.
3. **Each account's records.**
   - **Nothing enrolled yet.** The account is skipped with no registry read
     when `global.ini` is absent, it has no `cid = ` line, the `cid` is empty,
     or `<cid>.ini` is absent.
   - **Unreadable file.** A file that exists but cannot be read returns
     `SHAREPOINT_ROOT_RECORD_UNAVAILABLE` for the whole check.
   - **The whole account is malformed** when any of these hold. Nothing in it
     is trusted, and the malformed-record rule below decides whether that
     matters.
     - a file is not valid UTF-16LE (an optional byte-order mark is allowed);
     - a file is over 4 MiB;
     - there are two `cid` lines;
     - the `cid` is not `[A-Za-z0-9-]{1,64}`, so it cannot name another file.
   - **Line parsing.** Every `libraryScope = ` line is parsed strictly:
     - space- or tab-separated fields, where a quoted field runs to the next
       `"`, and no stray or unterminated quotes;
     - at least 13 fields, with a numeric index;
     - a scope ID of 32 hex digits, optionally with a `+<digits>` suffix that
       is dropped;
     - a tenant written as a dashed or braced GUID;
     - site, web, and list of exactly 32 hex digits.

     A line that fails is set aside as malformed (kept lowercased) rather
     than failing the account. It may belong to any library the person
     syncs.
4. **Registry corroboration.** The account's `ScopeIdToMountPointPathCache`
   is read.
   - Each value name is normalized as a scope ID. One that cannot be
     normalized is kept as "unknown scope".
   - Each path is canonicalized through the setup filesystem, or compared as
     written if it does not canonicalize.
   - **Folder comparison.** All folder comparisons use `cloud::path_components`
     on both sides: case-insensitive, component by component, with `\\?\`
     and `\\.\` prefixes removed. This is why a junctioned or redirected
     profile folder recorded as `C:\…` matches a candidate that canonicalized
     to `\\?\D:\…`.
5. **Malformed records fail closed only when they could be the candidate's.**
   If the account has malformed input, and its cache maps any scope to the
   candidate folder, the result is `SHAREPOINT_ROOT_RECORD_MALFORMED` in
   these cases:
   - the scope ID is unknown;
   - the whole account is malformed;
   - a malformed line contains that scope ID;
   - no well-formed record has that scope ID.

   Otherwise the malformed input belongs to some other library and is ignored.
6. **Which records take part.** A well-formed record takes part only when
   both of these hold:
   - its mount is not empty. An empty mount is a subfolder-only sync on
     separate `libraryFolder` lines, not the library root.
   - the same account's cache maps the same scope ID to the same folder.

   An uncorroborated record, such as one left behind by an unlinked account,
   neither verifies nor conflicts.
7. **Records at the candidate.** No corroborated record at the candidate
   returns `Ok(None)`. More than one, across all accounts and whatever their
   IDs, returns `SHAREPOINT_ROOT_RECORD_CONFLICT` ("folder records").
8. **Identifiers.** The record's tenant, site, web, and list must all equal
   the deployment's after normalization. Otherwise the result is `Ok(None)`.
   Titles and URLs are never consulted.
9. **Account slot.** The OneDrive account holding the record must be signed in
   as the connected account. Its `Accounts\Business<N>\UserEmail`, trimmed
   and non-empty, must equal the connected mail or UPN, ignoring case.
   Otherwise the result is `Ok(None)`: a library synced by someone else's
   OneDrive account on this computer is not this person's enrollment.
10. **One sync of the library.** Corroborated records of the provisioned
    library are counted only in slots signed in as the connected account. If
    there are more than one, the library is synced at two folders and the
    result is `SHAREPOINT_ROOT_RECORD_CONFLICT` ("library records"). Another
    person's OneDrive account syncing the same library does not count.
11. **Scope cache consistency.** In the holding account's cache, no other
    scope may map to the candidate, and this scope may map nowhere else.
    Either case returns `SHAREPOINT_ROOT_RECORD_CONFLICT` ("scope cache
    path").
12. Otherwise the tenant, site, web, and list are returned as dashed GUIDs.

`OneDriveRecordVerifier` (`src-tauri/src/sharepoint_root_verifier.rs`) is the
production adapter.
- It passes the connected account's mail and UPN, and canonicalizes recorded
  folders through the same `SetupFileSystem` that canonicalized the
  candidate.
- It maps a `RecordError` to its stable `{ code, message }`.
- It returns the candidate itself as `local_root`, with no drive.

### Setup service

`src-tauri/src/sharepoint_setup.rs` then applies its own checks.

- **Before any record is read**, these are required, in order:
  - OneDrive is installed (a real `OneDrive.exe`), else `ONEDRIVE_MISSING`.
  - Some `Business<N>` slot has a `UserEmail`, else `ONEDRIVE_ACCOUNT_MISSING`.
  - A Microsoft account is connected, else `MICROSOFT_ACCOUNT_MISSING`.
  - That account has a GUID object ID in the deployment tenant, else
    `MICROSOFT_ACCOUNT_WRONG_TENANT`.
  - Some `Business<N>` slot's `UserEmail` equals its mail or UPN, compared the
    same way as rule 9, else `ONEDRIVE_ACCOUNT_MISMATCH`.
- **Candidates** are the SharePoint roots `detect_cloud_roots` found. Each is
  canonicalized; ones that do not canonicalize are dropped, and duplicates
  are removed.
- **Per-candidate errors.** A verifier error for one candidate fails closed
  for that candidate only. The scan continues, and the first error is kept.
  Another synced SharePoint or Teams library that does not verify never
  blocks setup.
- **Verified candidates.** A verified identity must name the candidate and
  match every deployment ID again.
  - Overlapping verified roots return `SHAREPOINT_ROOT_NESTED`.
  - More than one verified root returns `SHAREPOINT_ROOT_AMBIGUOUS`.
  - With exactly one, `Files/Inbox` and `Files/Filed` are resolved as direct,
    separate children of it.
  - `start_sync` and `activate` also probe writability; `status` never writes.
- **No verified root.**
  - `status` and `start_sync` report `enrollment_pending`, and `start_sync`
    still asks OneDrive to sync through `odopen://`.
  - When a kept verifier error is the reason, that status carries it as
    `problem: { code, message }`; otherwise `problem` is `null`. Onboarding's
    sync step and the Settings SharePoint card show that problem in plain
    language with its support code while they keep waiting and rescanning.
    Persistent record trouble is therefore never a silent wait.
  - `activate` returns the kept error, or `SHAREPOINT_SYNC_PENDING` when there
    is none.

Every test fixture is synthetic. Each follows the layout above, and the
fixtures are labelled as synthetic in the code. The parser was also run once,
ad hoc, against this machine's real Personal `libraryScope` line. It parsed
the line and produced the site and list IDs that `ClientPolicy.ini` records.

### Not yet confirmed on a real tenant

- **Record format.** The Business `libraryScope` layout, field positions, and
  tenant spelling (see the table above).
- **Scope ID spelling.** The `+<digits>` scope-ID suffix, and whether the
  registry value name carries it.
- **Removal.** Whether unsyncing removes both the `.ini` line and the registry
  cache entry, and in what order.
- **`UserEmail` values.** Whether `UserEmail` holds the mail or the UPN for
  accounts where they differ.
- **Renamed accounts (review finding N3).** A UPN change can leave OneDrive's
  cached `UserEmail` on the old name. The string comparison then reports
  `ONEDRIVE_ACCOUNT_MISMATCH` from `status` even after activation, and
  onboarding completion is refused.
  - This fails closed. Admission is unaffected, because it compares Graph
    object IDs.
  - No change is made until pilot telemetry shows it happens.
- **Two OneDrive accounts.** Two OneDrive work accounts on one Windows profile
  both syncing the library. The rules above let the connected person's sync
  verify; this has not been exercised against the real client.

## Release acceptance (required before enabling the deployment)

On a Windows machine signed in to the Contoso tenant, after syncing the
`Files` library through the `odopen://` flow:

1. Confirm the settings folder contains `global.ini` and `<cid>.ini`, and that
   `<cid>.ini` has one `libraryScope` line with the library mount in field 12
   and the tenant, site, web, and list IDs in fields 7–10, matching the
   deployment.
2. Check the scope-ID field (1) on that line and the matching registry
   value name. The verifier accepts `<32 hex>` with an optional `+<digits>`
   suffix on either side. Any other spelling fails every activation with
   `SHAREPOINT_ROOT_RECORD_MALFORMED`, or leaves it unverified. If that
   happens, update the parser and fixtures before release.
3. Confirm `HKCU\Software\Microsoft\OneDrive\Accounts\Business1\ScopeIdToMountPointPathCache`
   maps that scope ID to the same folder.
4. Confirm activation reaches `ReadyToActivate`. Then unsync the library and
   confirm the records are removed and activation returns to pending, not
   verified.
5. Record the OneDrive client version.

[gist]: https://gist.github.com/guwidoe/038398b6be1b16c458365716a921814d
[vbaft]: https://github.com/cristianbuse/VBA-FileTools/blob/master/src/LibFileTools.bas
[issue1]: https://github.com/cristianbuse/VBA-FileTools/issues/1
[schreiner]: https://blog.andreas-schreiner.de/2023/11/02/onedrive-for-business-anpassen-von-ordnerpfad-und-anzeigename/
[msf]: https://github.com/rapid7/metasploit-framework/blob/master/modules/post/windows/gather/enum_onedrive.rb
[qa-dup]: https://learn.microsoft.com/en-au/answers/questions/5929719/persistent-onedrive-duplicate-tenant-library-entri
[cloudpilot]: https://www.cloudpilot.no/blog/List-all-your-syncronized-libraries-in-OneDrive-for-Business-using-PowerShell/
[xlwings]: https://github.com/xlwings/xlwings/issues/1829
[ode]: https://github.com/Beercow/OneDriveExplorer/blob/master/OneDriveExplorer/ode/parsers/sqlite_db.py
[issen]: https://github.com/SecurityRonin/issen/blob/main/research/onedrive-forensic-research.md
[syncrootid]: https://learn.microsoft.com/en-us/uwp/api/windows.storage.provider.storageprovidersyncrootinfo.id
[cfapi]: https://learn.microsoft.com/en-us/windows/win32/cfapi/build-a-cloud-file-sync-engine
