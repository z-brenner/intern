# Microsoft upload verification

## What Intern enforces

For the protected SharePoint intake, unverified ownership is never permission
to process. The admission guard runs before extraction, inference, or any other
document-content read, and runs again at every later pipeline boundary.

This release supports one fixed deployment: the `Files` library in
`teamcontoso.sharepoint.com/sites/InternTestSite`. Users must upload each new
document directly into `Inbox`; copying or moving an existing item there is
not a supported intake procedure. Tenant, application, site, library, drive,
and folder identifiers come only from the versioned deployment resource
bundled with the application. They are not settings and never cross the
webview IPC boundary as sign-in or pairing inputs.

The bundled resource is currently disabled because real public identifiers
have not been supplied. Microsoft connection, pairing, and protected-intake
admission therefore fail closed with:

> SharePoint deployment configuration is unavailable: provisioned identifiers
> are not available in this build.

No client secret is or should be packaged. The identifiers an administrator
must supply, how to find them, what users see during guided onboarding, and the
tenant-backed acceptance checklist are in
[SharePoint deployment: administrator guide](sharepoint-deployment.md).

Intern does not search Microsoft audit logs. Every decision below is made from
drive-item metadata alone.

## Fresh-upload proof

The connected work/school account returned by authenticated Microsoft `/me` is
authoritative. Display names, typed email addresses, machine labels, and local
sync arrival never authorize a file.

For every candidate, Intern performs these checks in order:

1. Map a direct local child of the verified Inbox root to a Graph drive-item
   metadata URL built from the fixed drive and Inbox identifiers.
2. Require the configured tenant, site, library/list, drive, parent Inbox,
   filename, item identity, and contained SharePoint web URL.
3. Require an ordinary file facet with no folder, remote item/shortcut,
   conflict, package, malware, deleted, or pending-operation ambiguity.
4. Require `createdBy.user.id` and `lastModifiedBy.user.id` to match `/me`.
   A principal-name fallback is allowed only for the corresponding absent ID
   and only when it exactly matches the verified `/me` principal. A conflicting
   ID is never overridden.
5. Require valid and identical creation/modification timestamps, a positive
   remote size, QuickXorHash, ETag, and SharePoint item identity. The creation
   time must be strictly later than the moment filing was turned on (the
   activation watermark), so documents already in `Inbox` before activation
   are never processed.
6. Only after those metadata and identity checks, read the local file to match
   its size and QuickXorHash and compute the queue's SHA-256 binding.
7. Fetch metadata again and require the same account, item, identities,
   timestamps, size, checksum, URL, and ETag/revision.

An exact match is authorized. A valid different creator is held for the other
account. Missing, conflicting, detectably copied or moved, edited, shortcut,
conflict, or otherwise ambiguous evidence is held as unknown. Microsoft
unavailability or throttling remains retryable and never becomes a negative
ownership verdict. There is no process-anyway override, including for the legacy
`processOthersUploads` setting.

A held document stays where it is in `Inbox`: it is not read, claimed, renamed,
or moved. Scans count held documents (for another account, or with an
unverified uploader) in Settings > SharePoint connection > Support details, and
they are checked again on later scans. A document already in the queue that
fails a later check goes to review with `UPLOADER_UNVERIFIED`.

SharePoint `createdBy` does not establish who later copied or moved an existing
item. A same-user copy or move can be indistinguishable from a direct upload
when all required identity, timestamp, location, size, checksum, and revision
facts match. In that case metadata-only verification may authorize the item;
this is an accepted limitation and does not prove that the user performed a
direct upload. The operating requirement remains direct, new, unchanged
uploads. Every other hold remains in force, including detectable copy/move,
editing, shortcut, conflict, identity ambiguity, and revision mismatch.

## OAuth and network boundary

Managed device sign-in accepts no tenant/client IDs and no audit
acknowledgment. It signs in against the configured tenant's authority and
requests only delegated `User.Read`, `Files.Read`, and `offline_access`, which
users can consent to themselves where tenant policy allows. When the
organization blocks that consent, sign-in fails with the
`MICROSOFT_CONSENT_BLOCKED` explanation and nothing weaker is attempted. Refresh credentials remain in the operating-system credential
store; access tokens and device codes never enter settings, attribution files,
shared folders, or IPC responses.

`Files.Read` can authorize file content at the Microsoft permission layer, but
Intern's HTTP allowlist exposes metadata only. It rejects Graph `/content`,
preview, upload, permissions, versions, and audit-log endpoints. The compiled
transport has no audit POST capability. Requests remain bounded by timeout,
response-size, retry, and throttle controls, and returned SharePoint URLs must
remain inside the configured site and Inbox boundary.

Device-code polling intervals, expiry, slow-down handling, explicit
cancellation, refresh-time `/me` revalidation, and tenant/account change checks
remain enforced. Disconnect invalidates authorization immediately even if OS
credential deletion reports an error.

## Release limitation and acceptance

Synthetic tests cover fixed-boundary mapping, exact IDs, principal fallback,
conflicting IDs, other/unknown outcomes, changed modifiers and revisions,
size/checksum mismatch, shortcut/conflict facets, transient Graph failures,
OAuth cancellation and credential behavior, scope emission, endpoint
confinement, no-payload IPC, and disabled deployment behavior.

They do not establish live SharePoint compatibility. Before the deployment is
enabled, a tenant-backed Windows acceptance run must prove that account A can
process only A's fresh unchanged upload and holds account B, detectably copied
or moved, edited, shortcut, conflict, unknown, offline, and changed-revision
cases before content extraction. The acceptance record must also call out the
same-user copy/move indistinguishability limitation rather than claiming those
operations can always be detected.

## Primary references

- [DriveItem resource](https://learn.microsoft.com/en-us/graph/api/resources/driveitem?view=graph-rest-1.0)
- [Device authorization](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-device-code)
- [QuickXorHash algorithm](https://learn.microsoft.com/en-us/onedrive/developer/code-snippets/quickxorhash?view=odsp-graph-online)
