# Releasing ATLAS

A release is a deliberate act. Nothing is published by merging; a maintainer
dispatches the release workflow against one exact `main` commit, and every gate
in it refuses anything else. This is the sequence that gets a release out
without a long run failing at its last step.

A release takes one QA run and one release run. QA does the testing: every
gate, the Windows build and its smoke tests, and the corpus scored with real
inference. The release does the shipping: it checks that the QA run it names
tested exactly what it is about to build, then builds, signs, smoke-tests,
attests and publishes the installer, with that QA run's evaluation and logs as
its evidence.

## The one rule

**After the QA run, nothing outside `docs/qa/` may change.**

The rendered-fidelity sign-off is bound to `release_inputs_sha256`, which
`scripts/hash-release-inputs.mjs` computes over every tracked file in the
commit's tree except `docs/qa/`. That includes the README, `site/`, the
workflows, the tests, and the release notes, not just application code. Change
any of them after QA and the sign-off no longer describes what would ship, and
the QA run no longer tested it; the release preflight refuses it in seconds,
and so does the release's check of the QA run it names. The same digest is
what lets the release ship on QA's testing instead of repeating it: the commit
QA ran on and the release commit must digest the same. So every pull request
for the release, the version bump, and the release notes merge **before** QA,
and the only
commit between QA and release is the sign-off itself, which touches nothing
but `docs/qa/`.

## One-time setup

For each release, add a matching version entry to
`src/features/updates/releaseNotes.ts` before QA. Write short, plain-language
changes for people using ATLAS, including fixes and improvements they will
notice. Keep older entries so people who skip releases see all their new
changes. The desktop app shows these notes once after updating, and remembers
the version when the person selects Continue or dismisses the summary.

- **Secrets** (Settings → Secrets and variables → Actions). The release
  workflow needs exactly two, and only in its build step:
  - `TAURI_SIGNING_PRIVATE_KEY`: the private half of the updater key whose
    public half is committed as `plugins.updater.pubkey` in
    `src-tauri/tauri.conf.json`. It signs the installer for the in-app updater.
    The build step fails without it rather than publish an update no installed
    copy can verify. Keep a backup: a lost key means installed copies can never
    accept another update, and every user has to reinstall by hand a build
    carrying a new public key.
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: that key's password.

  Nothing else is a secret. Publishing uses the workflow's own `github.token`,
  provenance is keyless (the runner proves its identity through OIDC), and CI
  and QA need no secrets at all, which is what keeps pull requests from forks
  working.
- **Pages**: Settings → Pages → Source: **GitHub Actions**. The Pages workflow
  deploys `site/` from `main`: on every push to `main` that changes `site/`,
  and once more when the release workflow has published, which starts it from
  `main` (a release published with the workflow's own token cannot start it
  any other way). The `github-pages` environment's default rule, `main` only,
  is all it needs.
- **Private vulnerability reporting**: Settings → Code security → Private
  vulnerability reporting, so the link in [`SECURITY.md`](../SECURITY.md) works.

## The sequence

1. **Merge everything the release contains.** All of it, including site and
   documentation changes. See the rule above.
2. **Prepare the release commit** on a branch and merge it once CI is green:
   - Move every release surface to the new version. The files are the ones
     `scripts/release-version-contract.test.ts` reads (package.json and its
     lock, the Cargo workspace version and Cargo.lock, `tauri.conf.json`, the
     worker protocol test, the PowerShell scripts, the notices, the README's
     installer names, the QA checklist, the release and CI workflows, and the
     workflow name `scripts/validate-release-evidence.mjs` accepts). That test
     fails until they agree.
   - Write `docs/releases/v<version>.md`. It is published as the release
     notes. It must start with `# ATLAS v<version>` and must say the
     installer is `not Authenticode signed`; the preflight checks both.
     (Notes before alpha.20, when the app was called Intern, start
     `# Intern v<version>`. The installer, the release assets and the
     install folder keep the name Intern.)
   - Return `docs/qa/rendered-fidelity-signoff.json` to pending
     (`status`, `release_inputs_sha256`, `screenshot_sha256`, `reviewer` and
     `reviewed_at` all `"pending"`) and the QA checklist to pending/blocked.
     QA would set a stale accepted record aside on its own runner anyway, but
     the committed file should say what is true.
3. **Dispatch "Whole-product QA evidence"** (`qa.yml`) on `main` at that commit.
   It takes about 45 minutes (69 for alpha.19, before its runtime came from
   CI's cache): every gate CI runs, the Windows browser tests, the Windows
   build, the installer smoke test, an update over an old release, and the
   whole corpus scored with real inference. Its first step prints the
   release-input digest this commit's sign-off has to carry, and if the
   committed sign-off was accepted for other inputs it is treated as pending
   for the run, with a note in the job summary. Download the
   `intern-whole-product-qa-<run>-<attempt>` artifact, and note the run's ID
   (the number in its URL): the release names it.

   QA reads the pinned Windows runtime (Tesseract and its closure, built by
   vcpkg) from the cache CI's Windows packaging job saves on `main`. If this
   release changes the runtime pins (`src-tauri/resources/runtime-assets.json`
   or `scripts/fetch-windows-assets.ps1`), wait for CI on `main` to finish
   before dispatching QA, or QA builds the runtime itself, about 16 minutes
   more. A version bump alone does not change the cache.

   QA also takes the step "Smoke test an update over v0.1.0-alpha.14", which
   installs that release's published installer and then this build's over it
   with `/UPDATE`, as the in-app updater does, and requires the old root
   llama.cpp build gone and an empty folder after the uninstall
   (`src-tauri/windows/hooks.nsh`). Its log is part of QA's evidence, and the
   release refuses a QA run without it. No separate CI run is needed for it
   any more.
4. **Review the capture and record the sign-off**, following the reviewer rule
   below. In one commit that changes nothing outside `docs/qa/`:
   - `docs/qa/latest-implementation.png`: the capture from the artifact,
     byte for byte.
   - `docs/qa/rendered-fidelity-signoff.json`: `status` `"accepted"`,
     `release_inputs_sha256` the digest the QA run printed (also in the
     artifact's `model-evaluation.json`, or run
     `node scripts/hash-release-inputs.mjs` on the commit), `screenshot_sha256`
     the capture's SHA-256, the reviewer, the time, and notes saying what was
     checked.
   - `docs/qa/fidelity-ledger.md` and `docs/qa/release-checklist.md`, updated
     from the run.

   The release names the QA run whose capture this is and checks that the
   committed capture is byte for byte the one in that run's evidence. CI on
   the sign-off's pull request runs only its Ubuntu jobs, a few minutes, since
   nothing outside `docs/qa/` changed.
5. **Run the preflight** on `main` after the sign-off merges:

   ```sh
   node scripts/release-preflight.mjs --root=.
   ```

   It checks the same things the release workflow's first job checks: the
   sign-off is accepted for exactly this commit's release inputs, the committed
   capture is the one it accepted, the notes exist and say what they must, and
   package.json, Cargo.toml, `tauri.conf.json`, the workflow name and the
   workflow name the final evidence validation accepts all state one version.
   It names every problem at once.
6. **Dispatch "Release v&lt;version&gt;"** (`release.yml`) on `main`, with
   `qa_run` set to the ID of the QA run whose capture was accepted:

   ```sh
   gh workflow run release.yml --ref main -f qa_run=<QA run ID>
   ```

   Its first job refuses any ref but `main`, a tag that already names another
   commit, and anything the preflight refuses. It then checks the QA run
   (`scripts/verify-qa-run.mjs`): a successful QA run dispatched on `main` in
   this repository, on this commit or an ancestor with this commit's release
   inputs, whose evidence manifest validates (an accepted model evaluation
   scored on these release inputs, an accepted installer smoke, the update over
   alpha.14), and whose capture is the committed one. All of it takes about a
   minute and names every problem at once. The release job then takes about
   half an hour (94 minutes for alpha.19, which repeated QA's work): it checks
   the same QA run again where it uses its files, builds the worker and stages
   the pinned runtime, builds and signs the installer, smoke-tests it
   installed, verifies the updater signature offline, writes SBOMs and
   checksums, validates the evidence manifest (its own installer smoke, and the
   QA run's model evaluation and logs, bound to that run), records the
   provenance attestation, creates the annotated tag, and publishes the
   release as **latest** (a prerelease would be invisible to
   `/releases/latest`, which is where installed copies look for updates). It
   does not run the gates or score the corpus again: the QA run did, on
   byte-identical release inputs. A last, short job starts the Pages workflow
   from `main`, so the download page is deployed again.
7. **Check what shipped.** Download the installer from the release page and
   verify it:

   ```sh
   gh attestation verify Intern_<version>_x64-setup.exe --repo z-brenner/intern \
     --signer-workflow z-brenner/intern/.github/workflows/release.yml
   ```

   Confirm `https://github.com/z-brenner/intern/releases/latest/download/latest.json`
   names the new version, and that the Pages run the release's last job
   (**Redeploy the download page**) started from `main` deployed the download
   page.

## The reviewer rule

The sign-off is a statement that a named person looked at this exact capture
and accepts it. It is only worth anything if that is true.

- Open the capture from this run's artifact, at full size, and check it against
  the concept the ledger records: the three planes, the header's privacy
  posture, date-first filenames with right-aligned confidence, statuses told
  apart by icon as well as color, attributed evidence, and no clipping,
  collisions, illegible text, or ambiguous focus.
- `reviewer` names the person who accepted it. An assistant may inspect the
  capture first and draft the record, but the notes then say who did what, as
  the alpha.10 record does.
- Never carry a digest forward from an earlier record, and never edit a digest
  or a hash to make a gate pass. A mismatch means the reviewed thing and the
  shipping thing differ; that is the gate working.
- If something is wrong, leave the record pending, or record it as rejected
  with the reason, and fix the cause. QA can be run again.

## When something fails

- **The published notes need a correction.** Fix `docs/releases/v<version>.md`
  in a pull request and merge it, then run **Sync release notes**
  (`release-notes.yml`) from `main` with the tag. It replaces the notes on the
  release page with the corrected file and changes nothing else; the assets,
  the tag and the latest flag stay as the release workflow left them.
- **The preflight says the sign-off is stale.** Something outside `docs/qa/`
  changed after the review. Run QA again on the current `main` and review its
  capture, or revert the change if it was not meant for this release.
- **The preflight says the screenshot does not match.** The committed capture
  is not the one the sign-off accepted. Commit the reviewed capture from the QA
  artifact, or review the committed one.
- **The preflight says the notes are missing or the versions disagree.** Fix it
  in a commit, which changes a release input: QA has to run again.
- **"v&lt;version&gt; already points at &lt;commit&gt;, not &lt;commit&gt;".**
  An earlier run created the tag. If no release was published from it (the
  earlier run failed after tagging), delete the tag
  (`git push --delete origin v<version>`) and dispatch again on the current
  commit. If a release was published from it, bump the version instead; never
  move a published tag.
- **The release job failed after tagging, on the same commit.** Re-run the
  failed job, or dispatch again on the same commit with the same `qa_run`: the
  tag already names it, so it is accepted, and the publish step creates the
  release, or repairs and publishes a draft one left behind. A release that is
  already published is left as it is.
- **The release job failed before tagging.** Fix the cause. If the fix is
  outside `docs/qa/`, it changes the release inputs, so QA runs again first.
- **The release refuses the QA run it was given.** Its first job names every
  reason. "Whose release inputs are ... this commit's are ..." means something
  outside `docs/qa/` changed after that QA run: run QA on the current `main`
  and review its capture. "The committed sign-off accepted ..." means the
  capture in that run is not the reviewed one: name the QA run whose capture
  was reviewed. "Not completed/success", "not a ... run", or "dispatched on
  main" mean the ID is not a passing QA run dispatched on `main`. "Run again in
  between" means the QA run was re-run while the release was starting:
  dispatch the release again. "Did not smoke an update over v0.1.0-alpha.14"
  means a QA run from before QA did that: run QA again. A QA artifact expires
  after 90 days; an older QA run cannot be shipped on.
- **QA's corpus scoring hit its 150-minute limit.** Usually a wedged
  llama-server: upstream llama.cpp from the staged runtime's `llama-upstream`
  folder, running the accurate model, which is the one the corpus is scored
  on.
  `docs/qa/logs/model-evaluation.log` in the artifact shows where it stopped;
  run QA again.
- **The installer smoke says an installed llama-server exited with
  -1073741515.** That is `STATUS_DLL_NOT_FOUND`: a DLL the server imports is
  not beside it. The server, which both models run on, loads its own DLLs and
  LLVM's `libomp.dll` from `llama-upstream\`. Compare what
  `scripts/fetch-windows-assets.ps1` staged in `src-tauri/resources/native/`
  with the server's imports in the pinned archive.
- **Pages did not deploy.** Usually Pages is not enabled; the run summary
  then says so, with the fix (Settings → Pages → Source: GitHub Actions). A
  run the `github-pages` environment refused stops before any step, so it
  writes no summary; its error names the ref, and running the Pages workflow
  from `main` deploys. If the release's **Redeploy the download page** job
  warns that it could not start the Pages workflow, the release itself is
  published: run the Pages workflow from `main` by hand.
