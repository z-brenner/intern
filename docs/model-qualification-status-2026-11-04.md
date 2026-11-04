# Model qualification status, 4 November 2026

Inspection base: main `c427aff0deb490c8eda1aeccc28a5416c233050b`. This is a coordination
and qualification plan, not a new inference result or a release sign-off.

## Work already under way

- [Run 41718479983](https://github.com/z-brenner/intern/actions/runs/41718479983) completed
  its candidate-model matrix. Do not duplicate it. It contains no incumbent 2B live arm.
  Its candidate scores cannot be treated as a matched comparison with an old 2B replay.
- [PR 59](https://github.com/z-brenner/intern/pull/59), head `91c8f35`, already implements
  model-role fallback with a review hold, and title identifier cleanup. Its CI run
  [41737578063](https://github.com/z-brenner/intern/actions/runs/41737578063) passed. Those
  engine changes are not duplicated here.
- The separate `zack/qwen35-9b-4b-tiers` branch, inspected at `fe8cf3a`, proposes 9B accurate
  and 4B stand-in tiers, new memory guards and a 0.6 confidence gate. Its alpha.13 release
  notes are drafts, including a placeholder for shipped-quant measurements. The branch is
  not main and is not release qualification.
- [41730968866](https://github.com/z-brenner/intern/actions/runs/41730968866) is already
  testing 9B n-gram speculation on InternBench.
- [41730957300](https://github.com/z-brenner/intern/actions/runs/41730957300) completed a
  9B MTP InternBench experiment. Success means the job ran, not that its quality improved.
- [41730932267](https://github.com/z-brenner/intern/actions/runs/41730932267) is already
  testing Q4_K_S/UD-Q4_K_XL alternatives. At inspection, several jobs remained in progress.

The most recent 30 workflow runs and the jobs of all these related runs showed no live 2B
control. This is bounded to visible repository activity; it does not assert that no private
or uncommitted experiment exists. No run was dispatched, cancelled or restarted here, and
none of the parallel branches was edited.

## Why a release remains blocked

The six-model matrix's candidate outputs provide useful failure cases, but changing a
model, deterministic rules, confidence threshold and quantization together does not isolate
the source of an improvement. The historical 2B replay and candidate live runs are different
experimental arms. A ready result is not necessarily a correct result, and a confidence
threshold selected on the real sets must not be advertised as validation on unseen data.

The proposed tier release also changes a quick reading from mandatory human review to
potential automatic filing. That policy change needs its own wrong-ready analysis and a
semantic audit of every newly ready result. Merely retaining old 2B files does not establish
a working fallback to that model. Download size, model process RSS and total app/worker RAM
are separate quantities; a 16 GB laptop must keep usable headroom for the person's work.

Do not release from the tier draft solely because the candidate matrix and ordinary CI
are green. Require the live comparisons, heldouts and existing packaged release gates below.

## Nonduplicative next experiment

1. Coordinate with PR 59 and freeze one exact engine commit containing its final reviewed
   rules. Keep the baseline and candidate on that same engine. For a model-only comparison,
   apply the same 0.5 gate to both first; evaluate any separately frozen deployment policy
   in a distinct comparison. Preserve both results.
2. Run **2B / 4B / 4B / 2B sequentially on the same CPU host**, using the same pinned
   upstream runtime, worker, corpus bytes, prompt/grammar, context and thread count. The
   existing matrix gives each model its own runner, so submitting two rows to it does not
   by itself create a same-machine control. The research branch's pinned sequential runner
   is a starting point; do not rerun all six candidates merely to fill the missing control.
3. Pin the actual GGUF bytes before use. Main's 2B control is 1,280,835,840 bytes, SHA-256
   `aaf42c8b7c3cab2bf3d69c355048d4a0ee9973d48f16c731c0520ee914699223`, at
   `unsloth/Qwen3.5-2B-GGUF` revision `f6d5376be1edb4d416d56da11e5397a961aca8ae`.
   The proposed 4B is 2,740,937,888 bytes, SHA-256
   `00fe7986ff5f6b463e62455821146049db6f9313603938a70800d1fb69ef11a4`, at
   `unsloth/Qwen3.5-4B-GGUF` revision `e87f176479d0855a907a41277aca2f8ee7a09523`.
   Keep the Apache-2.0 licence/notice inventory with the exact runtime and model assets.
4. Validate complete cohort identity before comparing scores. The real-document downloader
   intentionally permits partial sets, so percentages over different downloaded documents
   are not directly comparable. Keep exclusions explicit and require the same document
   bytes/gold for each arm; a common-subset analysis does not establish full-set coverage.
5. Report date/party/type correctness, filename correctness, wrong names ready, forbidden
   values, malformed/retried outputs, descriptions and review routing. Read the source for
   disagreements and every newly ready result. Record median/p95 total latency, model
   process startup, worker/model combined peak RAM and background responsiveness. A replay
   cannot measure current inference latency, and process-cold is not disk-cold.
6. Freeze any prompt/runtime/threshold choice on calibration documents before the final
   heldouts. Since all three real sets have now informed rules or model selection, include
   genuinely untouched documents for the final promotion check rather than calling those
   three sets unseen. Never send private company documents to an external service.
7. Only proceed to 9B if the 4B comparison demonstrates a worthwhile quality/safety gain and
   acceptable latency. Then run a similarly controlled 4B/9B pair. Keep the prior incumbent
   available as an operational fallback until migration and release checks are complete.

The current documented replacement acceptance still requires no trap dates or forbidden
parties, no date/party regressions, a repeatable critical-field/routing gain and bounded
latency/memory. A deliberate change to those requirements must state the tradeoff rather
than silently redefining success around the candidate's errors.

## Safe independent implementation

The already tested ASCII normalization optimization was cherry-picked onto current main in
the isolated `research/qualification-evidence` worktree. It touches `evidence.rs`, which is
unchanged between the earlier base and current main and is outside PR 59's changed files.
Its original matched-build replay measurement showed lower deterministic processing cost,
not faster inference or better model accuracy. On this current-main base, all 505 engine
tests passed, with four existing tests ignored, and both fixture replays and the 77-document
InternBench replay passed with zero regressions. The benchmark-transport release build and
Rust formatting check passed. Local reports are `target/current-main-{evidence,digest,internbench}.json`
in the original research checkout. No model default, threshold, memory guard or release
setting changes.

## Execution blockers

This environment still has no verified inference weights. The prior Hugging Face official
CDN route returned proxy HTTP 403, and the prescribed Library transfer helper failed before
transferring existing research parts. Neither route was retried or bypassed in this follow-up.
Those parts' model identities and full-file hashes remain unverified. The available connected
GitHub tools can read runs and rerun existing jobs, but expose no new workflow-dispatch action;
rerunning an existing candidate job would not supply different baseline inputs. The parent
is assessing the user's requested `zgbrenner-pc` alternative. No release is qualified here.
