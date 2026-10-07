# Intern

Intern reads a document and tells you what to call it.

Every pair below is copied from the scored corpus in
[docs/model-bakeoff.md](docs/model-bakeoff.md) — the left column is what the
previous pipeline produced, the right column is what this one does:

```text
2026-04-09 - Statement of Work.pdf  ->  2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide, Inc.pdf
2026-12-29 - notice.pdf             ->  2026-12-29 Notice of Termination - John Smith.pdf
2025-02-14.pdf                      ->  2025-02-14 Employment Agreement between Northstar Lantern Works LLC and Mira Vale.pdf
```

The first line is the point of the whole thing: `2026-04-09` was the date the
statement of work was signed, and `2026-04-01` is the date it takes effect. Only
one of those is what the document *is*.

Alongside the name it produces one sentence saying what the document actually
concerns, the verbatim excerpts behind every fact it used, and a confidence. If
anything is unsupported, the document goes to review instead of being renamed.

No account, no per-document cost, no file limit: the model runs on your own
computer, so a thousand documents cost what one does, and nothing caps how many
it names.

## What leaves your machine

With local inference selected, document text, extracted pages, OCR output and
model prompts stay on the machine. There is no telemetry or automatic cloud
inference fallback. This is the complete list of what Intern sends anywhere:

| Request | When | What it carries |
| --- | --- | --- |
| Model download | Once, when you start it (or never, if you choose existing model files) | A request for the pinned model file. Nothing about the machine or its documents. |
| Update check | When Intern starts, every 6 hours while it runs, and when you press **Check for updates**. The automatic check can be switched off in Settings. | A request for the GitHub release manifest. Nothing about the machine or its documents. |
| Hosted model | Only if you turn it on, under your own API key | The condensed text of each document, to the service you name. |
| Microsoft sign-in and Graph (`login.microsoftonline.com`, `graph.microsoft.com`) | Only in a build an administrator provisioned for the SharePoint deployment | Sign-in and its renewal, then account and file metadata for upload verification, never document content. The build published here has none. |

The first two are all a default install ever sends, and neither carries
document information. The model file is 1.19 GiB, the text model and nothing
else. The update check asks only "is there a newer, signed build"; untick
**Check for updates automatically** in Settings and only the button asks. A
found update is never installed by the check itself: installing is always a
separate, explicit click, and only ever happens if the update is signed by this
project's key — anything else is refused.

There are two optional integrations. Both require an explicit choice. Settings
can point Intern at a **hosted model** — Anthropic's, OpenAI's, or any service
that speaks the OpenAI chat-completions shape, including a local server such as
Ollama or LM Studio — under your own API key. With that on, the condensed text
of every document is sent to that service to be named. Nothing else changes:
the same distillation goes out, the same evidence checks are applied to what
comes back, and the file itself never leaves. The header badge stops saying
*On this device* for as long as it is on, the key lives in the operating
system's credential store rather than in any Intern file, and **Test
connection** sends only Intern's own calibration document. If the promise
above is why you use Intern, leave hosted inference off.

**Microsoft upload verification** is the account connection for the
SharePoint deployment. It exists only in builds an administrator has
provisioned with the deployment's public identifiers. It uses delegated
`User.Read`, `Files.Read`, and `offline_access` permissions to read the
connected account and each Inbox file's metadata, never document text, and it
does not search audit logs. Unknown uploaders stay held. See
[upload verification](docs/microsoft-upload-verification.md) and the
[administrator guide](docs/sharepoint-deployment.md).

Intern reads documents as text: native PDF text first, OCR when a page has none,
and no vision model. The projector for this model is 668,227,264 bytes — 637 MiB,
a second download larger than half the model itself, before anyone names a
single document, for a path almost nothing takes. A page that neither text
extraction nor OCR can read goes to review rather than being guessed at.

## Install and first run

Windows 10 or 11, x86-64. Download `Intern_0.1.0-alpha.11_x64-setup.exe` from the
[latest release](https://github.com/z-brenner/intern/releases/latest) and run it;
earlier versions are on the [releases page](https://github.com/z-brenner/intern/releases).
It installs per-user, so it does not ask for administrator rights, and
uninstalling it leaves your documents and Intern's own data alone.

**The installer is not Authenticode signed**, because this project has no
code-signing certificate. Windows SmartScreen shows "Windows protected your PC"
and calls it an unrecognized app; to continue, choose **More info** and then
**Run anyway**. On a PC where **Smart App Control** is on (enforcement mode),
Windows blocks the installer outright and offers no Run anyway, so Intern
cannot be installed there until it is signed. Nothing in this release removes
those warnings, and you should not take an unrecognized-app warning lightly —
so verify the download instead of trusting it. Every published installer
carries a keyless Sigstore build-provenance attestation naming the repository,
workflow, and commit that produced those exact bytes:

```sh
gh attestation verify Intern_0.1.0-alpha.11_x64-setup.exe --repo z-brenner/intern \
  --signer-workflow z-brenner/intern/.github/workflows/release.yml
```

`--signer-workflow` insists the attestation came from this repository's release
workflow, not merely from some workflow in it.

`SHA256SUMS.txt` is published beside the installer and catches a corrupted or
truncated download, but it proves nothing about origin: it sits on the same page
as the installer, so whoever could replace one could replace both. The
attestation is the part that establishes where the file came from.

On first launch Intern shows a setup screen and asks to download the one thing
the installer deliberately leaves out: the pinned model file, 1.19 GiB. That
download is resumable, it transfers no document information, and the file becomes active only after its exact length and SHA-256 match the manifest
built into the installer. If you already have the file, **Choose existing model
files** points Intern at it and skips the download. Nothing else needs
installing — PDF text extraction, OCR, and inference all ship inside the app.

Then drag documents or a folder onto the window — PDFs, Word documents
(`.docx`, `.doc`, `.rtf`, `.odt`), Excel workbooks and exports (`.xlsx`,
`.xls`, `.ods`, `.csv`), PowerPoint decks (`.pptx`, `.ppt`, `.odp`), `.eml` and
Outlook `.msg` emails, plain text, Markdown, and scanned images. Names
Intern can support with verbatim text from the document appear ready to apply;
anything else goes to review with the reason shown. Nothing on disk is renamed
until you approve it, either one item at a time or with **Apply all ready**,
and a rename can be undone. A document whose content Intern has already filed
is flagged as a duplicate of the filed name instead of being renamed into a
second copy, and the **History** view in Completed lists every rename and undo
Intern has ever applied, exportable as CSV. A long queue has a filter box
above it that narrows the rows by original name, proposed name, or a word
from the description.

Every rename carries a date. A name that does not start with the document's
date as `YYYY-MM-DD` is refused, in the inspector and again in the queue, so
nothing undated is ever filed. When the model read a date that Intern could
not find written in the document, review shows the date, says it is
unverified, and accepts it in one click — into the name to edit, or straight
through to the rename. When the model offered none, every date the document
states is one click away, and the file's own last-modified date is the last,
labelled resort. The date a reviewer gives a name is the date the document
is filed under: the year folder and the description record follow it.

Filed documents can be **arranged into subfolders** of the destination —
by year, by year and document type, by type, or by first party — with
**Arrange filed documents** in Settings. Folders are created as documents
arrive and removed again when an undo empties them, and a document missing
the fact a layout keys on goes into `Undated` or `Unsorted` rather than the
root, so what still needs a hand stays visible.

Intern can also **run in the background**: with the setting on, closing the
window keeps Intern in the system tray — watched folders keep working — and
**Start Intern when you sign in** makes it a quiet always-on service. Both are
off by default. The tray icon's tooltip says how many documents need review
or are ready to rename, so a glance answers whether the window is worth
opening.

The installer also puts Intern in Explorer's **Send to** menu: select
documents — attachments saved from Outlook, a folder of scans — right-click,
and choose **Send to > Intern**. They join the queue whether or not Intern is
already open. An add is never refused as a batch: a file Intern cannot read
(a `.zip`, an empty file, an Office lock file, a scan another program still
has open) is left out and named in the window, with the reason, and everything
else is queued. Installing an update restarts Intern without sending those
documents again.

If Intern cannot start, a dialog says why and names its data folder
(`%LOCALAPPDATA%\com.intern.app`), and the same is written to
`logs\startup-error.log` there. A crash leaves one line in `logs\intern.log`:
where and when it happened, never what the document said.

## Watched intake folders, OneDrive, and SharePoint

Instead of dragging documents in, Settings can point Intern at an **intake
folder** to watch: documents that appear in it are analyzed and, once approved
(or automatically, if you enable high-confidence renames), moved to the
destination folder under their new name.

### Your own folder: three clicks

The first time Intern opens without an administrator's deployment, it offers
to watch a folder for you. Nothing asks for a tenant, a site, or an account.

1. **Choose a folder.** Intern lists the OneDrive and SharePoint folders
   already on this computer, such as *Legal - Documents (Contoso SharePoint)*
   or *OneDrive – Contoso*. Use one, or browse to a folder inside it. Folders
   synced with **Sync**, folders added with **Add shortcut to My files**, and
   your own OneDrive all work. If none is listed, open the folder in
   SharePoint, click Sync (or Add shortcut to My files), and check again.
2. **Say where renamed documents go.** Intern creates a *Filed* folder next to
   the one you chose, or you choose another.
3. **Decide about what is already there.** If the folder already holds
   documents, Intern asks once whether to rename them too. The default is to
   rename only new ones.

Pick a folder that **only you add documents to**: choosing it is how Intern
knows the documents in it are yours, so it does not ask Microsoft who uploaded
them. Intern then keeps running in the system tray and starts when you sign in.
Settings shows how the folder is doing — up to date, waiting for OneDrive to
download documents, OneDrive not running, or no longer synced — with the one
button that fixes each, and **Set up a folder step by step…** runs the same
setup again. A document whose contents are still in the cloud is never renamed
or moved until OneDrive has downloaded it.

The verified team Inbox described below is unchanged. It is used only in a
build an administrator provisioned, and is documented in the
[administrator guide](docs/sharepoint-deployment.md).

### The verified team Inbox

Files continue to move through the **Microsoft sync client**. Shared intake
uses strict Microsoft uploader verification rather than assuming that a file
first seen on this machine was uploaded by its user. Unknown uploaders stay out
of the processing queue, including manual imports from the protected folder.
Ordinary manual documents outside intake stay local.

Intern supports one shared library: `Files` on
`teamcontoso.sharepoint.com/sites/InternTestSite`, watching `Files/Inbox` and
filing into `Files/Filed`. In a build provisioned for it, Intern opens with a
short guided setup. It downloads the model if needed, connects your Microsoft
work account (you confirm the name and email Microsoft returns), asks OneDrive
to sync the library, and turns on filing, running in the background and at
sign-in. Nothing asks for a folder, tenant, or identifier. Afterwards Settings
shows a read-only **SharePoint connection** card.

Only documents you upload directly into `Files/Inbox` after setup, unchanged,
are filed. Documents uploaded by anyone else, or that cannot be verified, stay
untouched in the Inbox. SharePoint metadata cannot always tell a copy or move
by the same person from a direct upload, so upload directly. If your
organization blocks the Microsoft connection, setup says so and asks you to
contact your IT administrator.

The build published here has no deployment identifiers, so it shows the
folder setup above instead of the team Inbox setup. The [administrator guide](docs/sharepoint-deployment.md) lists
what a provisioned build needs, and the tenant-backed acceptance run required
before any live SharePoint compatibility is claimed. No live tenant
compatibility is implied by unit tests.

Several machines can coordinate using `.intern/` claim files and the existing
filed index. Sync-based leases are best-effort, not an exactly-once guarantee.
Machine names and origin markers help coordination; they are never evidence
of the Microsoft uploader. Team-worker mode still requires a verified uploader.
The protocol is documented in [shared intake](docs/shared-intake.md).

In a build without the deployment, Settings lists the **synced locations** the
sync client keeps on the machine — each SharePoint library and OneDrive
account, with its local folder — so the folder a library syncs to
(`C:\Users\pat\Contoso\Legal - Documents`, a name nobody chose) can be picked
as a destination without a hunt through a folder dialog. A synced folder is
watched as your own only through folder setup; one typed into Settings by hand
is still held for Microsoft upload verification, and private local mode is
refused for synced folders. A build with the deployment hides these manual
controls and uses the SharePoint connection instead. A folder on a **network
share** — a UNC path or a mapped drive — is recognised and labelled too, and
can never be watched as your own, because anyone with access can add to it.
A subfolder the account cannot read is counted and
skipped rather than stopping the scan.

### The description in a SharePoint column

Alongside the filename, Intern writes one sentence saying what the document
concerns. With **Write a description record for each filed document** turned
on, every rename into the destination folder also writes a small JSON record
under `<destination>\.intern\descriptions\` — the filename, its path in the
library, the sentence, and the date, type, and parties behind the name; never
document text. The sync client uploads the record like any other file, and a
Power Automate flow (the recipe is in
[`docs/sharepoint-descriptions.md`](docs/sharepoint-descriptions.md)) copies
the sentence into a **Description** column of the library. Intern still makes
no network request for this; the flow runs under the SharePoint account of
whoever creates it. Records for documents filed before the setting was turned
on can be written from Settings, and the rename history's CSV export carries
the same sentence in its last column for anyone who would rather paste.

## How a document becomes a filename

```text
document
  -> text/Markdown extraction (native text first; OCR only when there is none)
  -> whole-document distillation (every page read, redundancy removed)
  -> one local inference
  -> evidence validation
  -> filename + description + review decision
```

The distillation stage is the part that matters. Intern does **not** send the
model the first few pages and the last few pages. It reads every block on every
page, scores each one for how much it helps answer "what is this, when does it
take effect, who is it between", and keeps the best blocks **in document order**
under a character budget, marking elisions with `[...]`. A statement of work
whose effective date is on page five reaches the model exactly as well as one
whose date is on page one.

Kept text is verbatim, which is what makes the safety check real: the model has
to quote the document, and Intern checks the quoted facts against the document
before they can rename anything.

Full details, thresholds, and measurements are in
[`docs/architecture.md`](docs/architecture.md).

## The filename

```text
YYYY-MM-DD <what the document is> <the party or parties>.<original extension>
```

The date is the one that *defines* the document, not the first, last, or
easiest one to find: an agreement's effective date, a notice's notice date, an
invoice's invoice date, an amendment's own date rather than the date of the
agreement it amends. Payment due dates, renewal deadlines, and response
deadlines are never used. If no defining date can be established, the document
goes to review rather than getting an invented one — and it is renamed only
once a person has given it one, typed or accepted from the model's unverified
reading. What *kind* of date it is
is read from the document's own wording around it (`Effective Date:`,
`Invoice date`, `Notice is hereby given ... on`) rather than taken from the
model's habit, and a document the model could not type but whose title names
a type (`MUTUAL NON-DISCLOSURE AGREEMENT`, `Board Meeting Minutes`) is named
from the title and sent to review with the reason shown.

Names are sanitised for Windows, keep the original extension, shed the least
identifying detail first when they would be too long to scan, and get a numeric
suffix on collision.

### The same document twice

Identical bytes are caught before analysis and flagged as a duplicate of the
filed name. A second scan of the same page, a PDF exported twice, or a copy
saved again with new metadata is not identical bytes, so Intern also
fingerprints the text it extracted and holds every new document against what
it has filed - on this machine and, through the shared filed index, on
teammates' machines. Two documents that share their words are not always one
document: this month's statement and last month's differ in a date and a few
figures, so the dates must agree too. A near-duplicate waits in review, named
after the filing it repeats, and is never filed on its own; a person can file
it anyway, keep the original, or remove it.
[`docs/prior-art.md`](docs/prior-art.md) says where the idea came from and
what was left on the table.

### Spellings Intern learns from you

The document says "Contoso Worldwide, Inc."; you call it "Contoso". Respell a
party or a document type in review and Intern remembers it. The second time
you make the same change it becomes the spelling Intern uses, in the name, the
layout folder, and the description record, for every later document that
names the same thing. One edit is a decision about one document; two are a
preference. Settings lists everything learned, with "Use now" for a spelling
you want at once and "Forget" for one you do not, and the review panel says
"Uses your spelling" whenever a name differs from the evidence under it. The
evidence itself always keeps the document's own words: a learned spelling
changes what Intern calls a thing, never what it found.

## Development

```sh
npm ci
npm run dev
```

Use the pinned Node 24.15.0 (`.nvmrc`), Rust 1.88.0, and the committed
`Cargo.lock`. Run the deterministic clean-room corpus generator and the frontend
gates with:

```sh
npm run fixtures
npm run check
npx playwright install chromium
npm run test:e2e
```

The browser development and Playwright builds use the in-memory bridge; no
document content or test data is sent to a service. Fixture contents and
reviewed answers are documented in `fixtures/README.md` and
`fixtures/expected.json`.

Run Rust tests with:

```sh
cargo test --locked --workspace --all-targets
```

Accuracy is a number, not a feeling. The corpus is scored on every push from a
committed recording of what the parser read and what the model replied, and
CI fails when a reviewed answer that used to be right is now wrong:

```sh
cargo run --locked -p intern-engine --bin intern-evaluate -- \
  --fixtures fixtures/generated --expected fixtures/expected.json \
  --replay fixtures/corpus-recording.json --baseline fixtures/corpus-baseline.json
```

A prompt change makes the recording stale and needs a live re-record;
[`docs/evaluation.md`](docs/evaluation.md) has the workflow, and
[`docs/model-bakeoff.md`](docs/model-bakeoff.md) the numbers.

### Crates

| Crate | What it owns |
| --- | --- |
| `intern-engine` | Document understanding: distillation, prompt, local model client and server, the optional hosted-model client, evidence validation, house style learned from review, filename composition, model installation, and the corpus evaluator with its record-and-replay mode. |
| `intern-intake` | Shared intake folders: the multi-machine claim protocol, cloud sync-root detection, and the polling watcher. |
| `intern-queue` | The durable queue: ordering, leases, retries, the review/apply workflow, and the spellings it learns from approvals. |
| `intern-core` | Crash-safe queue storage and journalled file operations. |
| `intern-worker` | The out-of-process parser: PDFium, OCR, Office, and Outlook message extraction. |
| `intern-app` | The Tauri desktop shell, including the switch between the local and hosted models and the credential store the hosted key lives in. |
| `intern-release-verifier` | The offline release gate: verifies the installer's updater signature against the public key committed in `tauri.conf.json`, and binds it to `latest.json`, before anything is published. Not shipped in the app. |

The engine is usable without the desktop app:

```sh
intern-analyze --file contract.pdf --worker intern-worker.exe \
  --endpoint http://127.0.0.1:8080/v1/chat/completions --api-key KEY
```

It prints the proposed filename, the description, the evidence, the review
reasons, and local timings as one JSON object. `--distill-only` prints the
digest without running a model. This is the same code path the app uses, so a
watched folder, a script, or a future connector gets identical results.

## Windows runtime assets and installer

From PowerShell on Windows, fetch the exact llama.cpp b10361, PDFium
chromium/7881, Tesseract 5.5.2, tessdata, ONNX Runtime 1.30.0 and OCR model
assets ([OCR](docs/ocr.md) says which models and why):

```powershell
cargo build --locked -p intern-worker --release --features windows-native
Copy-Item target/release/intern-worker.exe src-tauri/binaries/intern-worker-x86_64-pc-windows-msvc.exe
./scripts/fetch-windows-assets.ps1
npm run assets:verify -- --require-bundled
./scripts/stage-windows-runtime.ps1 -Destination "$env:TEMP/intern-runtime-stage"
./scripts/smoke-worker.ps1 -WorkerPath "$env:TEMP/intern-runtime-stage/intern-worker.exe" -RuntimeDirectory "$env:TEMP/intern-runtime-stage"
npm run tauri build -- --bundles nsis -- --locked
```

Every downloaded archive or trained-data file is rejected unless both its exact
byte length and committed SHA-256 digest match. The fetcher checks out the exact
vcpkg baseline, stages only required executables/DLLs/trained data and the full
upstream/vcpkg license closure, and records both source and packaged paths plus a
digest for every file in `src-tauri/resources/runtime-assets.json`.
Tauri produces a per-user NSIS installer. To smoke-test a clean installer:

```powershell
./scripts/smoke-installer.ps1 -InstallerPath target/release/bundle/nsis/Intern_0.1.0-alpha.11_x64-setup.exe
```

The installer includes third-party notices and the generated `licenses/`
inventory but never includes `.gguf` model files. Local model weights are
installed separately under the user's local application data only after explicit
setup.

## CI and release

CI runs fixture generation, TypeScript/Vitest/Vite, Playwright, Rust format,
Clippy with warnings denied, workspace tests, native fixture integration, asset
verification, the Windows Tauri build, and the installer smoke test. A separate
**Rust (Ubuntu)** job runs format, Clippy, and the workspace tests in minutes,
for feedback before the Windows job finishes. Runtime and dependency caches are
keyed by lockfiles and the runtime asset manifest.

`scripts/run-model-evaluation.ps1` scores the whole gold corpus through the
shipping pipeline with the exact pinned model and real inference, and
`scripts/validate-model-evaluation.mjs` gates on date, type, party, and
description accuracy, on never filing a document under a date the corpus marks
as a trap, and on the review rate. Publishing is a deliberate
`workflow_dispatch` against a chosen main commit, never a side effect of
merging; the release job still refuses any commit but the one it was dispatched
for, and a preflight refuses in seconds a commit whose sign-off, release notes,
or versions cannot ship. [`docs/releasing.md`](docs/releasing.md) is the exact
sequence, and [`SECURITY.md`](SECURITY.md) says how to report a vulnerability.

## License

Intern is © 2026 Zachary Brenner, and is **source-available** under the
[Elastic License 2.0](LICENSE) — not an OSI-approved open-source license.

Read, audit, build, run, modify, and redistribute it freely. Intern makes
strong claims about what it does and does not do with your documents; the
source is published so those claims can be checked rather than taken on faith.
What the license reserves to the copyright holder is providing Intern to
third parties as a hosted or managed service, and circumventing its license or
functionality restrictions.

Third-party components keep their own licenses: see
[`THIRD_PARTY_NOTICES.md`](src-tauri/resources/THIRD_PARTY_NOTICES.md) and the
`licenses/` directory in the installed application.
