# Review-rate failure and preserved evidence

The later [grounded-subject improvement](post-alpha13-grounded-subject.md)
records the repeated failure and a tested optional-subject false-positive fix.

The branch remains **unqualified for release**. Windows QA
[41950475741](https://github.com/z-brenner/intern/actions/runs/41950475741),
head `be08aa5122758e4cc787cd64dc95b641063b9ed5`, failed live acceptance after
its preceding checks and installer smoke passed. The completed job log
independently confirms **19 evaluated, 7 named, 100% correct dates when named**,
and the failure `review rate 63.2% exceeds the 60% ceiling`. The 60% ceiling
and the app's 0.7 probability gate remain unchanged. The log totals 1,766
passing workspace Rust tests, 6 ignored, plus 4 native fixture and 6 PP-OCR
integration tests. This success does not cancel the model acceptance failure.

## What changed between the live runs

The parent workstream independently inspected the two artifacts and reports:

| Measurement | Earlier run 41940189938 | Failed run 41950475741 |
|---|---:|---:|
| Named / completed | 9/19 | 7/19 |
| Held for review | 10/19 | 12/19 |
| Exact filenames | 9/18 | 10/18 |
| Journal minimum probability | 0.773 | 0.604 |
| Termination notice minimum probability | 0.751 | 0.673 |

The journal and termination filenames stayed the same; their reported
probabilities crossed below 0.7, correctly changing readiness. The lease's
date suffix disappeared from its type, improving its filename. All 19
completed records returned probabilities, below-threshold outputs were held,
and no forbidden dates or parties were reported. The only remaining strict
ready-filename mismatch is the board deck's source-supported audience naming,
discussed in the [filename audit](post-alpha13-filename-audit.md).

The per-record numbers above are parent-reported on this executor. The
failed run's archive is artifact `13381733292`, GitHub digest
`sha256:db391ad38961a99bc2694df95ba1ed0b2ac8a665c1742e29fde12eb22c84d9dd`.
Its supported materialization helper failed to download it; no alternate
transfer route was attempted. The decoded job log was independently inspected
and its failure and expected/actual filename pairs preserved.

## Causal limits

Only `facts.rs` and `phrases.rs` differ outside QA documentation between the
two evaluated heads. The production change is the terminal-date type cleanup,
after model inference. It changes no prompt, grammar, sampler, parser or
confidence threshold. The earlier frozen-binary paired replays held replies
and probabilities constant and changed only the lease's type/description/name;
all other outputs and all readiness decisions agreed. All existing corpus
replays also agreed before and after that cleanup.

This supports inference or input variance rather than a demonstrated
readiness regression in the cleanup. It does **not** establish why the model
probabilities changed: raw prompt identities, reply tokenization, hardware and
server details must be compared. Identical filenames alone do not prove
identical model responses. The low-confidence results are still held for
review. No product decision is loosened merely to turn this failure green.

## Diagnostic change and next experiment

Commit `4259e1c354ecbf286884d024b52034c2f1448921` makes future QA preserve the
missing replay evidence. Its release-input digest is
`2c28afbf6fff33193047b7abff7959ba4a1cd511882650eaf08d06ce3ad3fc53`.

- QA explicitly records the synthetic fixture extraction, parsed model replies,
  prompt hashes and probabilities, including when acceptance subsequently
  fails. The recording is included in the existing always-uploaded artifact.
- Reports bind the recording's SHA-256 and server executable's SHA-256 and
  record the processor identifier beside the existing thread count.
- The evaluator logs each completed record's readiness, reasons and token
  probabilities. It logs no source text or credentials. The report retains
  the full evidence, and the existing gates remain authoritative.

Local checks pass: 17 evaluator tests; a real fixture replay with zero stale
prompts and zero baseline regressions; strict evaluator Clippy; formatting;
and the full frontend check, 761 tests in 77 files plus types and build.
All 19 logged diagnostic rows exactly match their report records, and the
replay's semantic outputs agree with the previous passing replay. The new
PowerShell recording path still requires execution on Windows QA.

The next live QA run is for preserved causal evidence, not an attempt to
discard the failed run. Replay its same recording through frozen before/after
evaluators at 0.7, refuse stale prompts, and compare every readiness decision
and filename. Retain both live outcomes even if a subsequent run passes; a
single pass does not resolve this variance or establish laptop qualification.
The interrupted PC run remains untouched. No PR, approval, merge or release
was performed.

Matching QA is running at
[41955071603](https://github.com/z-brenner/intern/actions/runs/41955071603),
head `20ac4c267881ea1108401d739d561cdc2abdb7f6`. It failed again at 12/19 review.
The journal and termination minimum probabilities repeated the prior failed
values. Its recording was preserved; the passing run's difference remains
unproven. See the later improvement above for current validation status.

Machine-readable receipts are in
[`post-alpha13-review-variance.json`](post-alpha13-review-variance.json).
