# Release evidence manifest

`scripts/create-release-evidence.mjs` writes schema version 1 of `release-evidence-manifest.json`. The manifest records the exact 40-character commit, GitHub workflow name, run ID, and attempt that produced it, then records the byte length and SHA-256 of:

- the production model evaluation;
- the 1536×1024 implementation capture;
- the run checklist;
- the rendered-fidelity sign-off;
- the installed-core smoke report;
- the exact NSIS installer; and
- every named test log.

`scripts/validate-release-evidence.mjs` re-hashes every artifact and derives all three sign-off states from source evidence. A publishable manifest requires:

1. a completed, accepted model report produced by the QA run the manifest names under `qa` (its commit and run ID), scored on the release inputs the release workflow states with `--release-inputs`;
2. an accepted human fidelity record bound to the screenshot SHA-256 and the model report's release-input SHA-256; and
3. an accepted installer report for the manifest's own commit and workflow run, with successful native launch, clean shutdown, signed runtime inventory, installed worker path, uninstall, and user-data-retention checks.

A release manifest (one with `distribution`) always carries `qa`: the commit, workflow (`Whole-product QA evidence`), run ID and attempt of the QA run the release ships on, which must equal the `--qa-*` values the release workflow passes from its `qa_run` input. Its logs must include `cargo-test.log`, `model-evaluation.log` and `update-smoke.log` (that QA run's) and `installer-smoke.log` (the release run's own). A QA manifest never carries `qa`; its model report is its own run's.

The fidelity record uses the release-input digest rather than embedding the commit that contains the sign-off. This avoids a self-referential Git commit while still preventing a sign-off from being reused after product, prompt, fixture, runtime, packaging, or test inputs change. The final manifest separately names the exact release commit.

The read-only QA workflow may create a blocked manifest with `rendered_fidelity: pending`; `--allow-pending` exists only so the complete QA artifact can be uploaded for inspection, and so the release can validate that artifact before shipping on it. The tagged release workflow never uses that flag for its own manifest. It no longer reruns the model: it names one QA run (`qa_run`), and `scripts/verify-qa-run.mjs` refuses that run unless it is a successful QA run dispatched on main, on a commit whose release-input digest (computed from git) is the release commit's, whose own manifest validates, whose evidence includes the update over alpha.14, and whose capture is the committed, accepted one. The release then ships the reviewed capture and that QA run's model report and logs, builds, signs and installs the exact installer, creates a new run-bound manifest naming the QA run, validates it without exceptions, and only then reaches the publishing step.

Until alpha.20 the release workflow ran every gate and scored the corpus again itself, on release inputs its preflight had already required to be identical to QA's. That repeated about forty minutes of work and could only add a chance of a flaky refusal; the binding above is what makes QA's result the release's.

An earlier revision of this paragraph said the release run "reruns Q4 and Q8" and "recaptures the UI". Neither is true: `src-tauri/resources/model-manifest.json` pins one model and `docs/model-bakeoff.md` records Q8 as rejected, and the release workflow deliberately ships the capture a reviewer inspected rather than generating a new one — see the freshness note in `docs/qa/release-checklist.md`.
