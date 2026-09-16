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
  is strict: a `libraryScope` line it cannot fully parse makes the whole
  source fail closed (`SHAREPOINT_ROOT_RECORD_MALFORMED`). It never guesses.

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
- The sync records have no Graph drive ID. The verifier therefore takes
  `drive_id` from the packaged deployment, and only after tenant, site, web,
  and list are proven locally. That is explicit in
  `src-tauri/src/sharepoint_root_verifier.rs`.

## Verification rules (implemented)

`intern_intake::onedrive_identity::verify_library_root`:

1. Read only `Business1` to `Business9` accounts. Take the single `cid` from
   `global.ini`; it must be `[A-Za-z0-9-]{1,64}`, so it cannot name another
   file. If `global.ini` or `<cid>.ini` is missing, or `cid` is empty, the
   account has nothing enrolled and the result is `Ok(None)`.
2. Parse every `libraryScope` line strictly: UTF-16LE with an optional BOM,
   quoted fields, at least 13 fields, a numeric index, a 32-hex scope ID, a
   dashed or braced GUID for the tenant, and 32 hex digits for site, web, and
   list (the scope ID may carry a `+<digits>` suffix, which is ignored). Any
   failure returns `SHAREPOINT_ROOT_RECORD_MALFORMED`.
3. Ignore records with an empty mount (subfolder-only syncs). Compare mounts
   with the candidate component by component, case-insensitively, reusing
   `cloud::path_components`.
4. No record at the candidate returns `Ok(None)`. More than one record at
   the candidate, across all accounts, returns
   `SHAREPOINT_ROOT_RECORD_CONFLICT`.
5. The record's tenant, site, web, and list must all equal the deployment's.
   GUIDs are compared case-, dash-, and brace-insensitively; otherwise the
   result is `Ok(None)`. Titles and URLs are never consulted.
6. If the provisioned library is mounted at more than one folder, the result
   is `SHAREPOINT_ROOT_RECORD_CONFLICT`.
7. The account's `ScopeIdToMountPointPathCache` must map this scope ID to the
   same folder. If the scope is absent, the result is `Ok(None)` (sync not
   settled). If the scope maps to another folder, or another scope maps to
   this folder, the result is `SHAREPOINT_ROOT_RECORD_CONFLICT`.
8. A source that exists but cannot be read returns
   `SHAREPOINT_ROOT_RECORD_UNAVAILABLE`. A settings file over 4 MiB is
   malformed.

The setup service then applies its own checks: the canonical `local_root`
must equal the candidate and every identifier must match; more than one
verified root is ambiguous; and the Inbox and Filed folders are resolved
under the verified root.

Every test fixture is synthetic. Each follows the layout above, and the
fixtures are labelled as synthetic in the code. The parser was also run once,
ad hoc, against this machine's real Personal `libraryScope` line. It parsed
the line and produced the site and list IDs that `ClientPolicy.ini` records.

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
