# Observed confidence groups and runtime interpretation

The completed action-noun QA
[41964133224](https://github.com/z-brenner/intern/actions/runs/41964133224)
passed live acceptance and the in-place paired replay at source
`b702a134fc12826bdbf654a75c3273d476d2c28d`, release-input digest
`e8a4ddb7775398f8cf902df329a973dc90ce11b8b7f68efe845ed29fcd871aeb`.
This supports that fix; the later report-ownership fix has separate Windows
QA [41970479658](https://github.com/z-brenner/intern/actions/runs/41970479658),
which remains in progress. Neither result alone qualifies a release.

## Confidence observations, without a vendor split

The parent independent artifact audit identifies these processors:

| Run | Processor identifier | Live ready / reviewed |
| --- | --- | --- |
| 41961091444 | AMD Family 25 Model 1 | 9 / 10 |
| 41955071603 | Intel Family 6 Model 207 | 7 / 12 |
| 41964133224 | AMD Family 25 Model 17 | 8 / 11 |

The original two runs have matching hashes for all 21 fixture files and
extractions, 19 prompts, the model and server executable, according to the
parent audit. This executor independently compared all 19 completed-record
confidence objects in their job logs; every object differs. Only the journal
and termination notice changed readiness under the unchanged 0.7 gate. The
SOW stayed in review for unsupported description wording in both runs.

| Case | Run 41961091444 minimum / readiness | Run 41955071603 minimum / readiness |
| --- | --- | --- |
| Journal | 0.7725790739059448 / ready | 0.6038329005241394 / review |
| Termination notice | 0.7505289316177368 / ready | 0.6733022928237915 / review |
| Statement of work | 0.9344902634620667 / review | 0.9439870715141296 / review |

The newer run's 19 confidence objects **exactly match run 41955071603**,
independently checked here from completed job logs. Yet its processor is
AMD Family 25 Model 17 according to the parent artifact audit. A split by
CPU vendor is therefore contradicted. Runtime and CPU variant effects remain
unresolved; no particular instruction set, dispatch path or cause has been
isolated. No confidence group is discarded and no routing uses processor
vendor. The original ambiguous-note counts, 16 versus 12, are counts of
critical-value confidence summaries, not total generated tokens.

## What the completed pair establishes

The job logs independently show 19 completed live documents: eight ready,
eleven reviewed (57.9%), with every below-0.7 result held. The in-place control
and current replays each complete the same 21 fixture records, including two
expected extraction errors. Their 19 completed replies retain identical
confidence objects; all logged current-replay fields equal the live run.

The comparison reports one change: the SOW moves from
`description_unsupported` review to ready. Filename, description, proposal,
status and confidence are unchanged, with no readiness regression. The parent
independent archive audit additionally confirms unchanged proposal/facts and
confidence. Archive bytes have not been independently materialized here; the
GitHub artifact metadata and the independently checked log ledger are retained
in the companion JSON.

The reported evaluation takes **463.2 seconds**, with median **20.692 seconds**.
The wall time is also visible in the completed job log. The apparent hour-long
step includes an approximately 52-minute pre-evaluation gap reported by the
parent audit. The cause of that gap is unresolved; it must not be presented as
an hour of model inference. The evaluation timer starts after server health,
so model provisioning and startup are outside these measurements. These hosted
Windows results establish neither application cold-start time nor laptop
qualification.

The successful pair supports the [action-noun fix](post-alpha13-subject-actions.md)
without crediting any older pass to it. The later ownership fix, capture-specific
fidelity review and release preflight remain separate. The completed matched
PC run and reported replay are recorded in the
[ownership evidence](post-alpha13-report-organization.md). No PR approval,
merge or release was performed.

All 19 rows from each observed run and their provenance are retained in
[`post-alpha13-hardware-cohorts.json`](post-alpha13-hardware-cohorts.json).
