# Reducing unnecessary review without changing confidence policy

The branch remains experimental and unqualified for release. Source commit
`6c51362b5caf279d9fb1625f5092c3b06d442ab7` has release-input digest
`35f3e854a27f7b50ebc05e027a87569ebc05efcef0ed868fd8bf29fa79160c02`.

## The repeated live failure

Windows QA [41955071603](https://github.com/z-brenner/intern/actions/runs/41955071603)
again failed at 12/19 review, above the unchanged 60% ceiling. Its new job-log
diagnostics independently confirm journal minimum probability
**0.6038329005241394**, termination notice **0.6733022928237915**, and 19
completed records with probabilities. Both correctly remain in review under
0.7. The statement of work has minimum **0.9439870715141296** and is held only
for `description_unsupported`.

The independent audit reports that the recording, all 21 fixture hashes and
all 19 reply-confidence values match their report, and the two failed runs
agree across output fields and readiness. That archive inspection is reported
here; its bytes have not been materialized on this executor. The earlier
passing run has no equivalent recording. **Why that run differed remains
unproven.** Repeated current values are not treated as a transient failure to
be erased by resampling until green.

## A supported false-positive review

The existing fixture recording gives the statement of work the optional
subject `implement CRM platform and migrate records`. Its source says the
provider will implement the customer relationship management platform and
migrate records. The capitalized `CRM` was treated as an unsupported name.
The subject was already omitted from the description, but its review flag
still blocked otherwise supported filename and description facts.

The new rule recognizes one otherwise unstated, three-to-six-letter uppercase
acronym for this optional subject's claim check only. Its expansion must be
one unique sequence of consecutive substantive source words. Every other
significant subject word must occur in that same excerpted passage. An
acronym alone, an unmatched or ambiguous expansion, a match in another passage,
or evidence outside the model's excerpts does not qualify.

The original subject still goes through the literal-word check. An unstated
abbreviation therefore stays out of the output. The rule adds no filename
fact, name, date, identifier or amount. All other specific claims are checked
as before. The 0.7 confidence gate and 60% acceptance ceiling are unchanged;
no gold, baseline or tolerance was edited.

## Validation

- A real fact-validation regression test failed before implementation with
  `DescriptionUnsupported` and passes afterward. Additional cases preserve
  review for wrong acronyms, invented names, invented amounts, irrelevant
  passages, ambiguous expansions, acronym-only subjects and unseen evidence.
- **1,500 portable Rust tests passed, 0 failed, 6 ignored**; strict workspace
  Clippy excluding the local desktop crate and formatting pass. The desktop's
  GTK dependencies remain unavailable locally, so new Windows QA is needed.
- A paired replay of the existing 21-fixture recording changes only the
  statement of work's readiness, review reasons, subject support and matching
  readiness score. Its filename, description, proposal and token probabilities
  are identical. The unsupported abbreviation remains omitted. Other records
  agree. Named results increase from 8/19 to 9/19 on **this historical replay**;
  this is not a result from the latest Windows recording or fresh inference.
- In a separately labeled constructed guard replay, changing only that
  statement's minimum probability to 0.699 still sends it to review with
  `low_confidence`, despite the grounded acronym. Its emitted facts stay the
  same. The confidence gate is not bypassed.
- Both 77-document model replays pass unchanged baselines with no stale
  prompts and no semantic differences from the prior passing replays: 9B
  retains 73/77 filenames, 77/77 dates and parties, zero strict wrong-ready;
  the 2B control retains 71/77 filenames and its existing one wrong-ready.
  The digest fixture replay also remains identical and passes its baseline.

No timing or model-memory improvement is inferred from these replays. The
latest captured Windows recording must still be replayed before and after
this fix, without stale prompts, and fresh Windows QA must run this source.
Even if acceptance passes, held-out document quality, actual 16 GB CPU laptop
behavior and required fidelity review remain outstanding release gates. The
uncertain PC background run remains untouched. No PR, approval, merge or
release was performed.

Machine-readable receipts and frozen evaluator hashes are in
[`post-alpha13-grounded-subject.json`](post-alpha13-grounded-subject.json).
