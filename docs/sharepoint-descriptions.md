# Descriptions in a SharePoint column

Intern produces two things about every document: a filename and a
one-sentence description. The filename travels with the file. The sentence
used to stay in the local queue, where nobody else could see it.

A SharePoint document library has a natural home for that sentence — a
column — and the sync client that carries the file can carry a small record
beside it. This document describes the record Intern writes, the
ready-made Power Automate flow that copies it into a column, how to build the
same flow by hand, and what the design deliberately does not do.

## In short

1. In Settings → *Descriptions for SharePoint*, **Write a description record
   for each filed document** is on. (Setup turns it on when the output folder
   is in a OneDrive or SharePoint library.)
2. In the library, add a column named **Description** (*Multiple lines of
   text*).
3. Import the flow: download
   [`intern-description-flow.zip`](https://z-brenner.github.io/intern/downloads/intern-description-flow.zip),
   then in Power Automate choose *My flows → Import → Import Package
   (Legacy)*, pick your SharePoint connection, and choose your site and
   library in the flow.
4. Back in Settings, **Write records for documents already filed** fills in
   what was filed before.

Intern itself makes no network request for any of this. The flow runs in
your own Microsoft 365 tenant, under your own SharePoint connection.

## What Intern does

With **Write a description record for each filed document** turned on in
Settings (under *Descriptions for SharePoint*), every rename Intern completes
into the output folder also writes one JSON file under

```text
<output folder>\.intern\descriptions\<key>.json
```

The record carries the document's new filename, where it is, the
description, and the facts behind the name. Nothing in a record is document
text: not a page, not an excerpt, not the evidence quotations. If the output
folder is in a SharePoint library synced with the OneDrive client, the sync
client uploads the record the same way it uploads the renamed document, and a
flow in the library can read it.

The setting needs an output folder, because that is where the records live;
Settings refuses to save it without one. Undoing a rename removes the record
for the path the document vacated. Re-filing a document at the same path
overwrites its record, so a document has at most one record at a time.

Intern itself makes no network request for any of this. The two requests
described in the README — the one-off model download and the update check —
remain the only ones, unless a hosted model has been chosen in Settings; that
is a separate decision, and these records play no part in it.

## Filling the column: import the ready-made flow

Intern ships a Power Automate flow that does this, packaged so it can be
imported rather than built. It runs in your Microsoft 365 tenant, under the
SharePoint connection of whoever imports it, so that person needs edit rights
on the library. Intern is not involved once the record is on disk.

> **Status: not yet verified in a tenant.** The package was written from
> Microsoft's connector reference and from the layout of real exported
> packages, and its structure is tested locally, but it has not yet been
> imported into a real Microsoft 365 tenant. If the import or the first run
> fails, [build the flow by hand](#building-the-flow-by-hand) instead — those
> steps do exactly what the package does — and please report what Power
> Automate said.

1. **Add the column.** In the library, choose *Add column → Multiple lines
   of text*, name it `Description`, and save. (A single line of text holds at
   most 255 characters, which a long description can exceed.)
2. **Get the flow.** Download
   [`intern-description-flow.zip`](https://z-brenner.github.io/intern/downloads/intern-description-flow.zip).
   Settings → *Descriptions for SharePoint* → **Open the flow download**
   opens the guide section that links to it. Your browser downloads it;
   Intern does not.
3. **Import it.** In [Power Automate](https://make.powerautomate.com), choose
   *My flows → Import → Import Package (Legacy)*, and upload the zip. Under
   *Review package content*, the flow is set to **Create as new**; under
   *Related resources*, select the **SharePoint connection** row and choose
   your connection (or create one). Select **Import**.
4. **Point it at your library.** Open the imported flow, *Intern - fill the
   Description column*, and choose:
   - in the trigger, *When an Intern record is created or changed*: your
     **Site Address** and the **Library Name** Intern files into. Leave
     *Folder* blank.
   - in *Get the record*, and in *Find the document* (inside *Wait for the
     document to sync*): the same **Site Address**.
   - in *Write the Description column* (inside *If the document was found*):
     the same **Site Address** and **Library Name**. Check that the
     **Description** box holds the record's description. If the library's
     column has the internal name `Description0` (see
     [building the flow by hand](#building-the-flow-by-hand)), that box is
     empty: put `body('Read_the_record')?['description']` in it as an
     expression.

   Save. If the flow is off, turn it on.
5. **Fill in what was filed before.** Settings → *Descriptions for
   SharePoint* → **Write records for documents already filed** writes a
   record for every document filed before records were turned on. Each new
   or rewritten record starts one flow run.
6. **Check it.** File a document with Intern and watch the flow's run history.
   The column fills within a few minutes of the sync client uploading the
   record; the trigger checks the library every few minutes.

### What the flow does

- **Trigger: *When a file is created or modified (properties only)*** on the
  whole library, with two trigger conditions (in the trigger's *Settings*)
  that admit only `.json` files in a `.intern/descriptions` folder. Nothing
  else in the library starts a run, so the records folder never has to be
  found in a folder picker, and one flow serves every output folder in the
  library. *Modified* matters: a backfill rewrites records in place, and that
  has to reach the column too.
- **Get the record** (*Get file content*) and **Read the record** (*Parse
  JSON*, with the schema shown under
  [building the flow by hand](#building-the-flow-by-hand)).
- **Document path** works out where the document is: the record's own folder
  with `.intern/descriptions` taken off the end — which is the output folder,
  inside the library — followed by the record's `path`. Nothing has to be
  typed in, and it does not depend on `libraryPath`, so it also works for a
  record written without one.
- **Wait for the document to sync** (*Do until*, 60 tries a minute apart,
  so about an hour): *Get file metadata using path* on that path, and a
  one-minute wait whenever the document is not there yet. The sync client
  uploads a document and its record independently, and the record can arrive
  first.
- **Write the Description column** (*Update file properties*) with the
  record's `description`. If the document never appeared — the rename was
  undone, or the file was moved before the flow ran — the run ends as
  *Cancelled*, with nothing written.

Two things it does not do. Undoing a rename removes the record, but the flow
does not clear a description it already wrote; an undone document normally
leaves the library anyway. And writing the column counts as a modification of
the document; the trigger conditions keep that from starting another run.

### About the package

The package is a standard Power Automate legacy export: `manifest.json`, and
`Microsoft.Flow/flows/<id>/` holding `definition.json`, `apisMap.json`, and
`connectionsMap.json`. Its only dependency is the SharePoint connector, a
standard connector. The legacy import was chosen over a solution because it
needs no Dataverse, no solution publisher, and no environment variables: the
import asks for one connection, and the site and library are chosen in the
flow itself. Microsoft recommends solutions for moving flows between
environments, and in some environments the legacy import is not offered;
there, build the flow by hand.

The source is committed as readable JSON under
[`integrations/power-automate/description-flow/package/`](../integrations/power-automate/description-flow/package/),
and `npm run flow:build` packs it into
`site/downloads/intern-description-flow.zip`, which the Pages deployment
serves beside the guide. `scripts/build-description-flow.test.ts` reads the
zip back and fails if the committed zip is not what the committed source
builds, so change the JSON, run `npm run flow:build`, and commit both.

## Building the flow by hand

The same flow, step by step, for a tenant where the import is unavailable or
fails. Adjust the names to match yours.

1. **Add the column.** *Multiple lines of text*, named `Description`. Note
   the column's internal name if you later need it in an expression: for a
   column created with this name it is `Description`, but a library that
   already had a hidden field with that name gets `Description0`. The
   internal name appears in the column's settings URL after `Field=`.
2. **Create the flow.** In Power Automate, create an automated cloud flow
   with the SharePoint trigger **When a file is created or modified
   (properties only)**. Site Address: your site. Library Name: the library.
   Leave Folder blank. In the trigger's *Settings*, add two trigger
   conditions:

   ```text
   @endsWith(toLower(triggerOutputs()?['body/{FilenameWithExtension}']), '.json')
   @contains(toLower(triggerOutputs()?['body/{Path}']), '.intern/descriptions')
   ```

   The first keeps the folder's `README.txt` (and Intern's temporary files)
   from starting a run; the second limits the flow to records.
3. **Read the record.** Add **Get file content** (SharePoint), with
   Identifier set to the trigger's *Identifier*. Then add **Parse JSON** with
   Content set to the expression `json(string(body('Get_file_content')))` and
   this schema:

   ```json
   {
     "type": "object",
     "properties": {
       "version": { "type": "integer" },
       "filename": { "type": "string" },
       "path": { "type": "string" },
       "libraryPath": { "type": "string" },
       "description": { "type": "string" },
       "documentDate": { "type": ["string", "null"] },
       "documentType": { "type": ["string", "null"] },
       "parties": { "type": "array", "items": { "type": "string" } }
     },
     "required": ["version", "filename", "path", "description"]
   }
   ```

4. **Work out the document's path.** Add a **Compose** with the expression
   below. The trigger's *Folder path* is the records folder, such as
   `Shared Documents/Contracts/.intern/descriptions/`; taking
   `.intern/descriptions` off it leaves the output folder, and the record's
   `path` is relative to that:

   ```text
   replace(concat('/', first(split(triggerOutputs()?['body/{Path}'], '.intern/descriptions')), body('Parse_JSON')?['path']), '//', '/')
   ```

5. **Find the document, waiting for it to sync.** At the top of the flow, add
   **Initialize variable** (integer, value `0`). After the Compose, add a
   **Do until** that runs until the variable is greater than `0` (Count 60,
   Timeout `PT2H`). Inside it: **Get file metadata using path** with File
   Path set to the Compose output; **Set variable** to its *ItemId*; and a
   **Delay** of one minute, configured (*Settings → Run after*) to run only
   when *Get file metadata using path* has failed or timed out. The sync
   client uploads the document and its record independently, and the record
   can arrive first.
6. **Write the column.** After the loop (*Run after*: succeeded, failed, or
   timed out), add a **Condition** that the variable is greater than `0`. In
   *If yes*, add **Update file properties**: Library Name, the library; Id,
   the variable; Description, the `description` from Parse JSON. A record for
   a document the flow never finds is a document that was undone or moved
   before the flow ran; the flow can simply end.
7. **Test it.** File one document with Intern and watch the flow run.

## If the flow is not an option

The rename history's CSV export (Completed → History → Export CSV) has a
`description` column beside the original and new paths. Opened in Excel, the
sentences can be copied into the library in **Edit in grid view**, one column
at a time. It is manual, but it needs no flow and no admin. The export holds
every rename and undo, not only the ones the dialog lists; it starts with a
byte-order mark so Excel keeps accented names intact; and a description that
begins like a formula (`=`, `+`, `-`, `@`) is written with a leading
apostrophe, so it opens as text rather than running.

## The record

```json
{
  "version": 1,
  "key": "5d3f0c0e8b7a4f1e9c2d6a8b1f3e7d9c",
  "filename": "2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf",
  "path": "Contracts/2026/2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf",
  "libraryPath": "Contracts/2026/2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf",
  "library": "Contoso Worldwide",
  "provider": "sharepoint",
  "description": "Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc. for the 2026 member-map engagement, at a fixed fee of $248,000.",
  "documentDate": "2026-04-01",
  "documentType": "Statement of Work",
  "parties": ["Ridgeline Cartography LLC", "Contoso Worldwide, Inc."],
  "confidence": 0.93,
  "originalFilename": "scan0012.pdf",
  "filedAt": 1775001600,
  "machineId": "0123456789abcdef0123456789abcdef",
  "machineName": "Front desk",
  "userName": "pat"
}
```

| Field | Meaning |
| --- | --- |
| `version` | Record format version. This document describes version 1. |
| `key` | The record's own name (`<key>.json`): the first 32 hex characters of the SHA-256 of `path`, lowercased and `/`-separated, so machines that spell a path differently agree on the record. |
| `filename` | The document's filename after filing. |
| `path` | The document's path relative to the output folder, `/`-separated. |
| `libraryPath` | The document's path relative to the SharePoint library (or OneDrive) the output folder is synced from. Present only when the output folder lies inside a sync root the OneDrive client has registered on the machine that filed it. |
| `library` | The sync root's display name: the tenant for a SharePoint library, the account for a OneDrive. Present with `libraryPath`. |
| `provider` | `sharepoint`, `onedrive_business`, `onedrive_personal`, or `network_share`. Present with `libraryPath`. |
| `description` | The sentence that was applied — Intern's, or the reviewer's edit of it. |
| `documentDate`, `documentType`, `parties`, `confidence` | The validated facts behind the filename, as the review inspector showed them. Any of the first three may be `null` or empty when the document went through review without one. |
| `originalFilename` | The name the document arrived with. |
| `filedAt` | Unix seconds when the rename completed. |
| `machineId`, `machineName`, `userName` | Which machine and account filed it, as in the shared-intake claim files. |

Absent optional fields are omitted, not written as `null`. The folder also
contains a `README.txt` explaining itself to whoever finds it.

## Network shares and other destinations

An output folder on a network share — a UNC path like `\\fileserver\legal`,
or a mapped drive letter — gets the same records under
`<output folder>\.intern\descriptions`, with `libraryPath` absent. Settings
labels such a folder *On a network share*, and a watched input folder on a
share coordinates several machines through the same `.intern` claim files it
uses under OneDrive. Any script or indexer that can read the share can read
the records.

## What this design does not do, and why

**It does not call Microsoft Graph or the SharePoint REST API.** Doing so
would give Intern a sign-in, a token to store, an app registration for an
administrator to consent to, and a third network connection to audit — for a
product whose central promise is that it makes exactly two unless a person
chooses otherwise. The record approach keeps that promise intact: the only
thing moving anything to the cloud is the sync client already installed and
already trusted, and the only thing writing the column is a flow the
library's own people imported and can read, running under their own
connection.

If a direct writer is ever wanted, the shape is known: an Entra app
registration with delegated `Sites.ReadWrite.All` (or `Files.ReadWrite.All`
for OneDrive), a device-code sign-in, a `PATCH` to
`/drives/{drive}/items/{item}/listItem/fields` per filed document, and a way
to resolve a local path to a drive item — either the library URL entered in
Settings plus `/shares/{encoded url}/driveItem`, or the sync client's own
`libraryScope`/`libraryFolder` lines in its settings files. Each of those is
a trust decision for the people who run the tenant, not a default for a tool
that reads private documents. It would be built as an explicit opt-in, and
documented as the third request.

**It does not modify the documents.** Office files can carry custom
properties that SharePoint promotes into columns on upload, and PDFs can
carry XMP metadata; writing the description into the file itself would put
the sentence somewhere every viewer shows it. But it would also change the
file's bytes after Intern hashed and verified them, which is exactly what the
rename's verification exists to notice, and PDFs — most of what Intern files
— get no column from it anyway. Records beside the file keep the file
untouched.
