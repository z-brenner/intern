# Follow-up filename audit

The later [review-rate investigation](post-alpha13-review-variance.md) records
the failed matching Windows run and supersedes the pending status below.

This is experimental evidence for `qualification/post-alpha13`, not a release
sign-off. Source commit `4b1c1af28336a8ae993e80cf87bbb7c1c904865f` adds a narrow
document-kind cleanup after the earlier qualification fixes. Its release-input
digest is `8f3091501b8251a047790c96846ab723dcd5c3a0afee26f492592d05af76e659`.

## What was independently inspected

The GitHub metadata for Windows run
[41940189938](https://github.com/z-brenner/intern/actions/runs/41940189938)
shows its product checks, installer smoke, live evaluation, evidence binding
and upload steps passed at source `875bb6e49b10d3feb38ad23c21377941d667090a`.
The full run has completed successfully. The parent workstream independently
inspected its artifact and reported four strict filename mismatches among ready
outputs. After the job completed, this executor independently read the decoded
job log: all four expected/actual filename pairs match those findings. The log
confirms **19 evaluated, 9 named, 100% correct dates when named**, and acceptance
passed. It also confirms the release-evidence manifest's status remains
**blocked**. Readiness and individual probability values in the report remain
parent-reported. The Windows log totals **1,764 passing workspace Rust tests,
0 failed, 6 ignored**, with 4 additional native fixture and 6 PP-OCR integration
tests passing later. The low-memory desktop regression passed by name.

This executor resolved artifact `13379942626`, whose GitHub digest is
`sha256:c1de5b127e372f2cbe69189375ff46ca3ae501cd6676846257ae31ac31e9b376`.
The supported materialization helper failed to download it; a Library read
also could not resolve it as a visible Library file. No alternate byte-transfer
route was attempted. The new ZIP/report bytes and individual probabilities
remain unverified on this executor; reading the supported job-log endpoint
does not fill that gap.

Independently available evidence is the original, SHA-verified alpha.13
[`model-evaluation.json`](https://github.com/z-brenner/intern/releases/download/v0.1.0-alpha.13/model-evaluation.json),
the fixture sources, unchanged gold and existing recordings. The original
report contains the same four mismatch patterns. It did not collect token
probabilities, so it is not substituted for the new confidence-gated run.

## Each mismatch

| Fixture | Evidence and interpretation | Action |
|---|---|---|
| Scanned lease | The type copied `LEASE AGREEMENT EFFECTIVE SEPTEMBER 1 2024`. The date and both parties match the source; the type redundantly includes a date already carried by the filename. This is a naming-quality defect, not evidence of a wrong effective date. | Strip a whole terminal `effective`/`dated` date field immediately after the document kind. Keep the separately validated date, date role and evidence. |
| 100-page journal | The source's repeated heading is `MOONLIT ARCHIVE PROJECT JOURNAL - PAGE n`; the filename keeps `Moonlit Archive Project Journal`, while strict gold expects `Project Journal`. The date is correct. Existing title-completion tests explicitly preserve this more specific title. | Preserve the supported title. Record the strict naming-style mismatch; do not remove identifying words or force review to satisfy exact gold. |
| Termination notice | The source says `To: John Smith`; the output is `Notice of Termination to John Smith`, while gold expects `for John Smith`. The notice date and recipient are correct in the inspected report. | Preserve the supported recipient wording. This strict preposition mismatch does not establish a factual error. |
| Board deck | The source says `Prepared for Contoso Worldwide, Inc.` and `Presented by Ridgeline Cartography LLC`. The output names the audience with `for Contoso` and keeps Ridgeline in the description. Gold lists both parties but its composed expected filename uses only its first party, Ridgeline, after `for`. | Treat this as a subject/author naming-policy disagreement requiring gold review, not a demonstrated wrong-party claim. Keep source-supported behavior and the unchanged gold pending that review. |

These findings do not certify every output in the new Windows artifact. Exact
filename mismatch counts remain visible; no gold, readiness expectation,
probability threshold, baseline, tolerance or acceptance gate was relaxed.
In particular, no document is held for review merely because a hand-inspected
strict filename comparison prefers a different supported wording.

## Fix and validation

Two regression tests failed before the change and pass afterward. They cover
the copied whole heading through fact validation, natural and ISO terminal
dates, and preservation of substantive titles, invalid dates and date ranges.

- Portable Rust workspace: **1,498 passed, 0 failed, 6 ignored**, excluding the
  desktop crate whose local GTK dependencies remain unavailable. Strict
  Clippy and formatting pass.
- Existing 9B and 2B 77-document replays pass their unchanged baselines with
  zero stale prompts and identical semantic outputs to the previous passing
  replays. The 9B retains **73/77 filenames, 77/77 dates and parties, zero
  wrong-ready filenames** on that synthetic benchmark. The 2B retains 71/77
  filenames and its existing one wrong-ready result.
- Both fixture replay pipelines pass their unchanged baselines with identical
  outputs. The existing digest truncation and other fixture losses remain.
- A separately labeled controlled regression replay expands only the lease
  reply's type to the full heading observed in the verified published report.
  The same source, prompt identity, reply data and token probabilities are
  supplied to frozen before/after evaluators. Only that fixture's type,
  description, filename and corresponding score change. The filename becomes
  `2024-09-01 Lease Agreement between Cedar Finch Properties Llc and Orion Glass Studio inc.pdf`;
  date, parties, readiness and probabilities are unchanged. All other records
  agree. This is a constructed regression test, not a new live-model result.

No new timing or model-memory claim comes from these replays. Fresh Windows
QA must test this new source digest; the earlier run does not cover this
cleanup. Matching QA has started at
[41950475741](https://github.com/z-brenner/intern/actions/runs/41950475741),
head `be08aa5122758e4cc787cd64dc95b641063b9ed5`. It failed live acceptance:
7/19 named and 12/19 held for review (63.2%), above the unchanged 60% ceiling.
Installer and preceding checks passed. The branch remains unqualified.
The PC's interrupted control run remains untouched and its outcome
must be checked before any restart. Broader held-out checks, laptop behavior
and the required fidelity review remain release prerequisites. No PR,
approval, merge or release was performed.

Machine-readable receipts are in
[`post-alpha13-filename-audit.json`](post-alpha13-filename-audit.json).
