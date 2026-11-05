# Alpha.14 fidelity review packet — pending

## QA and source identity

Whole-product branch QA
[41975965963](https://github.com/z-brenner/intern/actions/runs/41975965963),
at `e4a61211e916921afefdbbd0112661245ff89e45`, completed successfully.
All substantive gates passed, including installer and live acceptance: 19
completed, ten ready, nine reviewed; all six probabilities below 0.7 held.
The in-place pair changes only SOW readiness, with no emitted-field changes
or readiness regressions. This executor independently checks the completed
job-log rows and comparison. The parent independent artifact audit verifies
all 11 artifact hashes and the blocked-on-fidelity manifest.

The QA and current checkout each have the same 730 non-QA Git tree entries,
independently reconstructed here, and digest
`01f0d67e72987a0cf196dad7ed1dd1c34b5abe4163b3673d787b7ee9d9208f24`.
The archive is GitHub artifact `13395287384`, 65,213,785 bytes, SHA-256
`fffbe4c98d9663dafccca2e0a7131c8e2b93a21740935cd2b5e98e3e2e761d37`
from GitHub metadata. This is branch QA, not the exact-main QA required by
the release procedure. No software gate is being described as failed.

## Exact fresh capture: independent AI inspection reported

The parent independent auditor reports inspecting the exact alpha.14 QA PNG,
1536 × 1024, SHA-256
`c37dbd5cffe9951863dfbd7fb24834f96997d7f4032c670b2dcecf15bb402797`.
This differs from the historical reference hash below; the earlier local
reference inspection cannot stand in for it. The fresh review is recorded as
an independent **AI inspection**, not this executor's own view and not named-
human acceptance.

The auditor reports that visible ledger criteria are met: distinct toolbar,
table and review planes; local privacy text; wrapped date-first filenames;
icon and color statuses; right-aligned confidence; focused filename; and pinned
controls. No visible collisions or unreadable text were observed. Date and Type
evidence are visible. Parties is below this frame's fold, so this still image
does not establish that portion or scrolling behavior.

The exact fresh image remains unavailable locally: one supported artifact
materialization attempt returned `download failed`, with no complete archive
installed and no fallback or retry. This is a local access limitation, not an
uncompleted independent AI review. The [Library SKILL.md](skill://plugin_connector_1p_1b8ff8edfc1481918b252c8277e23125/library/SKILL.md)
routes resolved references through its
[materialization instructions](skill://plugin_connector_1p_1b8ff8edfc1481918b252c8277e23125/library/references/materialization.md):
“Invoke it once per signed-URL transfer. Never use raw `curl`.” No new automatic
approval rejection occurred during that technical failure.

Existing browser tests already check full filename wrapping, focused input,
status contrast and pinned decision buttons at the default window, and keep
the inspector visible while a long queue scrolls (`tests/e2e/qa.spec.ts`). They
do not turn this still-image review into visual verification of Parties below
the fold. No extra requirement that every evidence field fit this one frame
is introduced.

## Historical reference inspected by the assistant

The assistant opened the existing committed **alpha.13** image at full size:
`docs/qa/latest-implementation.png`, 1536 × 1024, 168,334 bytes, SHA-256
`a95052639f75b6e2bdad0438da20645b59490271748ca2a9dab901fdb31cbd25`.
This is a reference inspection of historical bytes, not current capture
acceptance or a person-specific attestation.

Observed in that reference:

- Header, queue and review panel form distinct planes. The header visibly says
  `Private · On this device`; queue/completed tabs and action buttons are legible.
- Visible proposed names begin with dates. The panel shows a right-aligned 72%
  confidence value beside its track, and the active filename has a visible focus ring.
- Ready, Needs review, Proposing a name and Waiting use distinct icons and words
  as well as color. The selected row and review reason are clear.
- Date and Type appear separately under the evidence explanation. The explanation
  identifies the model's reading in its own words; it does not present it as a quote.
- No collisions or unreadable visible text were observed at this size. Long Check
  text is intentionally shortened in the table while its full reason appears in
  the panel. The fixed action footer is visible. Parties are below the visible
  panel area and cannot be assessed from this single frame; scrolling behavior
  and keyboard interaction are not established by a static image.

These observations cannot be silently carried forward to an unverified image.

## Exact remaining release blockers

1. **Permitted integration.** Main is still `8299690cd2713fd53bb120f81a9a0adbee58c432`
   and latest is alpha.13. The alpha.14 branch and a concrete PR body are prepared.
   An earlier draft-PR submission to z-brenner/intern was denied by automatic
   approval review. Available history retains the generic denial but not its
   specific reason. No submission retry, self-approval or main-write substitute
   has been attempted.
2. **Exact-main QA.** [docs/releasing.md](../releasing.md#the-sequence) requires
   all release changes, versions and notes integrated before QA on main. The
   branch's matching source digest supplies quality evidence but does not change
   that recorded sequence. Nothing outside docs/qa/ changes after final QA.
3. **Exact final capture and truthful fidelity acceptance.** Independent AI
   inspection of the branch PNG is complete, but no person has accepted the
   release capture. The exact final-tree image must be committed and reviewed
   under the main release procedure. The
   [reviewer rule](../releasing.md#the-reviewer-rule) says: “The sign-off is a
   statement that a named person looked at this exact capture and accepts it.”
   The existing sign-off's status, reviewer, hashes and date remain pending.
4. **Preflight and deliberate publication.** Preflight currently exits 1 for
   the pending fidelity record. After the prerequisites, the exact-main release
   workflow still must verify updater signing, installer, SBOM/checksums,
   evidence/provenance and the annotated tag before publication. Afterward,
   verify the published updater manifest, installer attestation and download page.

There is no alpha.14 tag or release. The constrained PC accuracy gain and
latency tradeoff remain recorded with their original scope; actual laptop
performance and the CPU/runtime cause of probability variation remain unproven.
