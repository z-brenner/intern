# Alpha.14 preparation and integration state

Prepared source `ed490a87dbb9c28ee88930da51275c2d605933ae` on isolated branch
`qualification/alpha14-preparation` has release-input digest
`01f0d67e72987a0cf196dad7ed1dd1c34b5abe4163b3673d787b7ee9d9208f24`. The alpha.13 qualification branch is preserved.
Main was independently checked at `8299690cd2713fd53bb120f81a9a0adbee58c432`,
with latest release alpha.13. PR 56 is already merged and its head remains an
ancestor. No main update, tag or release was performed.

## Concrete preparation

Every active version surface is aligned to 0.1.0-alpha.14, including package
locks, worker protocol expectations, runtime notices, SBOM paths, artifact names
and the release evidence validator. Release notes state the actual date,
ownership and memory fixes, confidence-policy parity and the performance limits.
No model/runtime pin, third-party dependency, updater identity/key, engine rule,
gold, baseline or tolerance changes during version preparation.

The fidelity record and checklist are pending. The old alpha.13 capture and
its explicit no-person-review history remain historical; no new acceptance is
claimed. [The PR body](alpha14-integration-pr.md) is prepared locally and has not
been submitted. Available history confirms that automatic approval review denied
the earlier draft PR to z-brenner/intern, but does not retain its specific reason;
that action is not bypassed or retried.

## Executed checks

- 765 frontend tests in 78 files, types, version contracts and production build.
- 1,507 portable Rust tests, zero failures, six ignored, in a bounded serial run.
- Formatting, strict portable workspace Clippy and optimized worker/evaluator/
  benchmark builds. The optimized worker answers protocol hello as alpha.14.
- Both historical 77-document model recordings and both 21-fixture pipelines
  pass their unchanged baselines with zero stale inputs and identical semantic
  outputs against the preceding source. Their timing/memory fields are inherited,
  separate from the matched PC experiment.
- Independent invariants check unchanged model/runtime files, engine facts and
  memory rules, gold/baselines, updater configuration and third-party dependencies.
- Release preflight exits 1 with only the intentionally pending fidelity record.

Local full Linux desktop checks remain unavailable because GTK development
packages are absent. The passing preceding-source Windows run
[41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658)
provides application-source evidence, not final alpha.14 version-tree evidence.
The PC worker's matched replay supports 9B's quality gain but the original
roughly fourfold latency cost remains; neither replay nor hosted QA establishes
actual laptop performance. See [the evidence](post-alpha13-report-organization.md).

## Remaining publication boundary

Push and verify this isolated branch, then complete fresh whole-product QA.
Integration still needs a permitted route. [The release procedure](../releasing.md)
requires the final version and notes tree on main before exact-main QA, followed
by the exact capture and truthful person-specific fidelity sign-off. Nothing
outside docs/qa/ changes after that QA. Preflight and the deliberately dispatched
release workflow then recheck the exact main tree and published artifacts.

The prepared tree is reviewable; it is not a release or an approval. Machine-
readable results and artifact hashes are in [the receipt](alpha14-preparation.json).
