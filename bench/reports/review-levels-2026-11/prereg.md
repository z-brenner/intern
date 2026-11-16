# Review levels on the held-out sets: what is measured, fixed before the run

Written 2026-11-15 ~19:25Z by the coordinator, before any level was applied to real7 or real8.
The levels and the rule below come from the gate track's report (the development analysis summarised in README.md beside this file),
written by an agent that never opened a held-out path, with two gaps closed that its checker named.
The coordinator knows how both sets read at today's gate (real7: alpha.18 report; real8: alpha.19 report).

## Data
- real7: the alpha.18 live recording (run 43302007759), replayed once through alpha.19's engine (--allow-stale, gate 0.8).
- real8: the alpha.19 live recording (run 43666145989), the replay already made for alpha.19.
- Accurate model only. Pooled over both sets; per-set numbers reported beside.

## Levels (engine terms)
A name is filed on its own only if it starts with a date and none of these hold it.
- Every level: DATE_MISSING, DATE_UNSUPPORTED, DATE_IS_PERIOD, DATE_IS_DEADLINE, DATE_IS_FORMULA, DESCRIPTION_INVALID,
  PARSER_WARNING (the whole reason), MODEL_REQUESTED_REVIEW.
- Strict (today): every review reason, and the least token probability below 0.8.
- Balanced: also DATE_AMBIGUOUS, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARTY_MISSING, a type of "Document"
  (proposal.document_type empty), and a title that stood in (TYPE_UNSUPPORTED and TYPE_INFERRED without TYPE_MISSING).
  Not holding: TYPE_MISSING, TYPE_UNSUPPORTED, TYPE_INFERRED, DESCRIPTION_UNSUPPORTED, the token probability.
- Relaxed: as Balanced, except PARTY_UNSUPPORTED holds only when the least token probability is below 0.6 or absent.
- A date the gold marks as a TODAY() value is counted as held by DATE_IS_FORMULA where the recording predates the
  worker reporting formulas (as the development analysis did).

## Severity
- right: the bench's filename_correct.
- really wrong: no date in the name; a date that is not the gold's date or an acceptable date; a type whose head noun differs
  from every gold type; a party shown in the name that the gold forbids for a reason other than being the individual who
  signs or is addressed, or that the gold does not know (a near spelling or a class of addressee aside).
- minor: everything else.
- Scored by `policy/severity.py` with the one fault its checker found repaired (a party the gold forbids is judged by
  the forbidden entry even when the gold also lists it among party_roles). Then every filed name that is not exactly right
  is read by hand against the gold. The number used for the decision is the larger of the script's count and the hand count.
- Reported beside, not used for the decision: the strict scale (a person or class named for the organisation counts as
  really wrong), and names that swap sender and recipient.

## Decision rule (the report's own, section 2.9, item 1)
- Balanced is offered, and becomes the default level, if on real7 + real8 pooled the really wrong share of the names it files
  is no more than 10%.
- Relaxed is offered as an opt-in if its share is no more than Balanced's plus 5 points.
- A level that fails is withdrawn, not tuned. If Balanced fails, the setting ships with Strict as the default and the owner decides.
- Nothing in a level is changed after this run. Any later change to a level has to be reported as made with both held-out
  sets in view.
