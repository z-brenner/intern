# Captured subject wording and checked action nouns

This follow-up closes a confirmed limitation in the initial grounded-acronym
rule. App fix `f81b149fd0ba3731822f01f0e0b14de8af245dcc` is followed by
qualification harness `08567177b5cc1e055b34b8635367c290c8bc62d3`. The combined
release-input digest is
`e8a4ddb7775398f8cf902df329a973dc90ce11b8b7f68efe845ed29fcd871aeb`.
The branch remains experimental and unqualified for release.

## Reproduced gap

The independent audit supplied the latest captured SOW subject:
`CRM platform implementation and migration`. The existing source passage
says the provider will implement the customer relationship management
platform and migrate records. `implementation` occurs nowhere in the
historical fixture extraction, and the initial rule required every other
significant word literally in the same passage. A real fact-validation test
of this exact wording failed before the change with `DescriptionUnsupported`.
The earlier test of `implement CRM platform and migrate records` did not
cover this noun/verb distinction.

## Bounded change

Only the optional subject's acronym-grounding anchor lookup accepts four
explicit noun/verb pairs: implementation/implement, migration/migrate,
configuration/configure and validation/validate. There is no general stemmer
or prefix match. At least one other anchor must still be literal in the same
excerpt, all remaining anchors must be literal or one of these pairs, and
the acronym must have one unique expansion of consecutive substantive words.

The original subject still goes through the unchanged literal-word gate and
remains omitted. The emitted name and description cannot gain an unstated
abbreviation or action noun through this change. Specific claims, critical
field validation, the 0.7 confidence gate and 60% review ceiling are unchanged.
No gold, baseline or tolerance changed.

## Executed paired evidence

The complete latest Windows recording has **not** been materialized on this
executor. The paired experiment is therefore explicitly a controlled
substitution: only the SOW subject in the historical fixture recording is
replaced with the audit-supplied captured wording. Its extraction, prompt
hashes, other model fields and probabilities are unchanged. This is not
represented as a replay of the entire latest Windows recording.

Frozen before/after evaluators at 0.7 reproduce and correct the gap. Only the
SOW's readiness, review reasons, subject support and readiness score differ.
Its filename, description, proposal and probabilities are identical. Its
subject remains absent from the output, and every other fixture agrees.
Both arms have zero stale prompts. A constructed guard that sets only this
statement's minimum probability to 0.699 still yields `needs_review` with
`low_confidence` and identical emitted facts.

## Checks and remaining qualification

- **1,502 portable Rust tests passed, 0 failed, 6 ignored**; strict workspace
  Clippy excluding the local desktop crate and formatting pass. Local desktop
  compilation still lacks GTK development dependencies.
- New positive cases cover the captured CRM wording and independently derived
  API configuration/validation wording. Negative cases retain review for
  invented names/amounts, unknown actions, misleading `station`/`state`
  prefix similarity, and subjects with only derived anchors. Earlier guards
  for ambiguous expansions, separate passages and unseen evidence still pass.
- Both 77-document model replays and both original fixture pipelines pass
  unchanged baselines, with zero stale prompts and identical semantic outputs
  to the preceding grounded-subject version. The 9B retains 73/77 filenames,
  77/77 dates and parties, zero strict wrong-ready; the 2B retains 71/77 names
  and its existing one wrong-ready. Original fixture losses remain visible.

The full frontend check passes 765 tests in 78 files plus types and build.
The replay-comparison tool is also exercised on the real controlled pair; it
reports one readiness improvement with identical emitted fields. Its tests
expose changed emitted dates/probabilities and ready-to-review regressions,
and reject live, stale, duplicate or mismatched inputs.

Qualification branches replay their own fresh recording in place on the
Windows runner against pinned control `777071d2582eb8dbdc861367f9bd47d20e32125d`.
The runner checks the recording's binding to the live report, isolates and
builds the control, replays both frozen executables at 0.7 without stale
inputs, and records the commit and executable hashes. It restores the current
evaluator after the control build. The comparison is diagnostic only, runs
after a failed live acceptance too, and preserves that failure. Changed emitted
fields or lost readiness fail the comparison.

## Completed Windows evidence

Matching QA [41964133224](https://github.com/z-brenner/intern/actions/runs/41964133224)
completed successfully at head `b702a134fc12826bdbf654a75c3273d476d2c28d`.
It passes native product tests, installer smoke, live acceptance and the
complete-recording pair. The live run completes 19 documents: eight ready and
eleven reviewed (57.9%), passing the unchanged 60% ceiling, with all results
below the unchanged 0.7 probability gate held.

The pair changes only SOW readiness, from unsupported-description review to
ready, with unchanged emitted fields and probabilities and no readiness loss.
The parent independent archive audit confirms unchanged proposal/facts and
confidence. This executor independently checks the completed job-log comparison,
all 19 probability objects in both arms and equality of the current replay's
logged rows to the live run. The complete archive was not materialized here;
the earlier controlled substitution remains separately identified above.

Measured evaluation takes 463.2 seconds (also in the job log), with median
20.692 seconds according to the parent archive audit. An approximately
52-minute pre-evaluation gap, of unresolved cause, accounts for most of the
apparent long step duration. These hosted Windows timings do not establish
actual laptop behavior.

The run uses AMD Family 25 Model 17, according to the parent audit, yet its
19 confidence objects exactly match the earlier Intel Family 6 Model 207 run,
independently verified here from logs. A vendor split is contradicted; runtime
and CPU variant effects remain unresolved. The
[confidence-group audit](post-alpha13-hardware-cohorts.md) retains all runs
without discarding either original group or crediting an older pass to this fix.

The [later report-ownership fix](post-alpha13-report-organization.md) has its
own QA at [41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658).
Completion of this preceding run cannot qualify that newer source. Capture-
specific fidelity review and release preflight also remain separate. No PR
approval, merge or release was performed. All receipt edits stay under
`docs/qa/` and preserve the corresponding release-input digests.

Machine-readable receipts and frozen evaluator hashes are in
[`post-alpha13-subject-actions.json`](post-alpha13-subject-actions.json).
