# Intern v0.1.0-alpha.14 release checklist

**Release status: pending/blocked.** This is release preparation, with no tag,
publication or accepted fidelity attestation. The final release-input tree
must complete QA and capture review before the deliberately dispatched release.

## Qualification evidence before version preparation

Application-source QA runs
[41964133224](https://github.com/z-brenner/intern/actions/runs/41964133224) and
[41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658)
passed. The latter completes 19 live documents, ten ready and nine reviewed,
passes installer/native checks and preserves emitted fields in the paired
comparison. The separate PC worker reports 73/77 accepted 9B filenames, all
dates and parties correct and zero strict wrong-ready after the shared fixes;
this reuses recorded replies. See [the evidence](post-alpha13-report-organization.md).
Those runs cover the preceding version. Alpha.14 branch QA
[41975965963](https://github.com/z-brenner/intern/actions/runs/41975965963)
subsequently passed all substantive gates, including installer and live acceptance,
on `e4a61211e916921afefdbbd0112661245ff89e45`: 19 completed, ten ready, nine reviewed,
all six below 0.7 held. Its paired replay changes only SOW readiness. The parent
audit confirms all 11 artifact hashes; local reconstruction confirms the QA and
current checkout's identical 730-entry release-input digest. The manifest remains
blocked on fidelity; this is still branch QA, not exact-main QA.

| Gate | Current status | Required evidence |
| --- | --- | --- |
| Aligned release versions, notes and local checks | local checks passed | Version contracts, 765 frontend tests/types/build, 1,507 portable Rust tests (six ignored), strict portable Clippy, formatting and optimized builds pass. Release preflight intentionally remains blocked on pending fidelity. |
| Integration and exact-main QA | pending/blocked | Integrate through an authorized route, then run Whole-product QA on the exact main release-input tree. |
| Browser, frontend, Rust and native runtime checks | branch passed; main pending/blocked | All pass in 41975965963. Repeat/verify under the exact-main release procedure. |
| Installer build and installed-core smoke | branch passed; main pending/blocked | Build, install, core runtime/OCR and uninstall-data retention pass in 41975965963. Exact-main procedure remains. |
| Live model acceptance | branch passed; main pending/blocked | 41975965963 passes at 0.7 and unchanged accuracy/review limits; all six below-threshold readings held. |
| Rendered fidelity review | pending/blocked | Exact final-tree capture, truthful named-person acceptance, matching digest and image hash. |
| Release preflight | pending/blocked | Accepted sign-off, matching capture, notes and versions on exact main. |
| Deliberately dispatched release workflow | pending/blocked | Exact-main rebuild, updater verification, SBOM/checksums, evidence acceptance, provenance, annotated tag and publication. |
| Published updater and download verification | pending/blocked | Verify the installer attestation, latest.json version and download-page deployment after publication. |

The fidelity record is pending. Nothing in this checklist authorizes a tag or
publication or substitutes for the gates in [the release procedure](../releasing.md).
