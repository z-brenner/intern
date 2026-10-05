# Releasing Intern

A release is a deliberate act. Nothing is published by merging; a maintainer
dispatches the release workflow against one exact `main` commit, and every gate
in it refuses anything else. This is the sequence that gets a release out
without a two-hour run failing at its last step.

## The one rule

**After the QA run, nothing outside `docs/qa/` may change.**

The rendered-fidelity sign-off is bound to `release_inputs_sha256`, which
`scripts/hash-release-inputs.mjs` computes over every tracked file in the
commit's tree except `docs/qa/`. That includes the README, `site/`, the
workflows, the tests, and the release notes, not just application code. Change
any of them after QA and the sign-off no longer describes what would ship; the
release preflight refuses it in seconds, and the final evidence validation
would refuse it after ninety minutes. So every pull request for the release,
the version bump, and the release notes merge **before** QA, and the only
commit between QA and release is the sign-off itself, which touches nothing
but `docs/qa/`.

## One-time setup

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
     notes. It must start with `# Intern v<version>` and must say the
     installer is `not Authenticode signed`; the preflight checks both.
   - Return `docs/qa/rendered-fidelity-signoff.json` to pending
     (`status`, `release_inputs_sha256`, `screenshot_sha256`, `reviewer` and
     `reviewed_at` all `"pending"`) and the QA checklist to pending/blocked.
     QA would set a stale accepted record aside on its own runner anyway, but
     the committed file should say what is true.
3. **Dispatch "Whole-product QA evidence"** (`qa.yml`) on `main` at that commit.
   It takes about two hours: every gate CI runs, the Windows build, the
   installer smoke test, and the whole corpus scored with real inference. Its
   first step prints the release-input digest this commit's sign-off has to
   carry, and if the committed sign-off was accepted for other inputs it is
   treated as pending for the run, with a note in the job summary. Download the
   `intern-whole-product-qa-<run>-<attempt>` artifact.
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
6. **Dispatch "Release v&lt;version&gt;"** (`release.yml`) on `main`. Its first
   job refuses any ref but `main`, a tag that already names another commit, and
   anything the preflight refuses, all in under a minute. The release job then
   takes about ninety minutes: the gates again, the signed installer, the
   installer smoke test, real inference on the corpus, the updater signature
   verified offline, SBOMs and checksums, the evidence manifest validated, the
   provenance attestation, the annotated tag, and the release published as
   **latest** (a prerelease would be invisible to `/releases/latest`, which is
   where installed copies look for updates). A last, short job starts the
   Pages workflow from `main`, so the download page is deployed again.
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
  apart by icon as well as colour, attributed evidence, and no clipping,
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
  failed job, or dispatch again on the same commit: the tag already names it, so
  it is accepted, and the publish step creates the release, or repairs and
  publishes a draft one left behind. A release that is already published is
  left as it is.
- **The release job failed before tagging.** Fix the cause. If the fix is
  outside `docs/qa/`, it changes the release inputs, so QA runs again first.
- **QA's corpus scoring hit its 60-minute limit.** Usually a wedged
  llama-server. `docs/qa/logs/model-evaluation.log` in the artifact shows where
  it stopped; run QA again.
- **Pages did not deploy.** Usually Pages is not enabled; the run summary
  then says so, with the fix (Settings → Pages → Source: GitHub Actions). A
  run the `github-pages` environment refused stops before any step, so it
  writes no summary; its error names the ref, and running the Pages workflow
  from `main` deploys. If the release's **Redeploy the download page** job
  warns that it could not start the Pages workflow, the release itself is
  published: run the Pages workflow from `main` by hand.
