# Stronger local models for a 16 GB CPU laptop

Research date: 2026-11-03. Base: `7996324` (main after #55).
**No replacement is qualified. No new release is justified by this work yet.**
The environment's proxy refused the model-weight CDN with HTTP 403; the same pinned baseline
download failed on the authorized network retry. No weights were obtained, so there are no new
inference, accuracy, latency, or RAM measurements. Do not substitute the older runs below for a
same-machine control. Offline test results are in the accompanying qualification report.

## Recommendation and shortlist

Test **Qwen3.5-4B Q4_K_M first**, using the existing evidence pipeline and its grammar, with
thinking disabled through `chat_template_kwargs.enable_thinking=false` (already sent by
`client.rs`). It is the smallest straightforward stronger-family replacement. Its ability to
improve Intern is a hypothesis, not established by general model benchmarks.

Sizes below are actual files reported by the Hugging Face API on the research date, in decimal
GB. They are **not resident-memory estimates**. No projector is needed: Intern sends text.

| Model / file | Download | Licence and disposition | Why test it / obstacle |
|---|---:|---|---|
| Qwen3.5-2B Q4_K_M | 1.281 GB | Apache-2.0; current accurate tier | Mandatory local control, byte-identical to the desktop manifest |
| **Qwen3.5-4B Q4_K_M** | **2.741 GB** | Apache-2.0; first candidate | Same request format; earlier laptop measurement found substantially slower prefill |
| NuExtract3 Q4_K_M | 2.783 GB | Apache-2.0; second-stage experiment | Extraction-specific 4B fine-tune, but needs its typed template and output adapter; not a drop-in prompt replacement |
| Gemma 4 E4B it Q4_K_M (Unsloth) | 4.977 GB | Apache-2.0; independent-family challenger | 4.5B effective / 8B including embeddings; larger memory footprint and template/runtime qualification |
| Qwen3.5-9B Q4_K_M | 5.681 GB | Apache-2.0; quality ceiling experiment | Could fit a quiet 16 GB machine, but must measure compute buffers, KV cache and worker alongside it; likely expensive CPU latency |
| LFM2.5-2.6B Q4_K_M | 1.674 GB | LFM Open License v1.0; reject as unrestricted default | Commercial use restriction at $10M annual revenue conflicts with unrestricted enterprise distribution |
| LFM3-1.7B | See existing reports | Same commercial licensing obstacle | Already evaluated in this repo; no demonstrated upgrade to repeat without a new reason |

Primary sources: [Qwen 4B model card](https://huggingface.co/Qwen/Qwen3.5-4B),
[4B GGUF files](https://huggingface.co/unsloth/Qwen3.5-4B-GGUF/tree/e87f176479d0855a907a41277aca2f8ee7a09523),
[9B files](https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/tree/3885219b6810b007914f3a7950a8d1b469d598a5),
[NuExtract3 usage](https://huggingface.co/numind/NuExtract3),
[NuExtract3 GGUF](https://huggingface.co/numind/NuExtract3-GGUF/tree/28a1aae6288c5d50bdf999ca2e572240966d669b),
[Gemma architecture](https://huggingface.co/google/gemma-4-E4B-it),
[Gemma GGUF](https://huggingface.co/unsloth/gemma-4-E4B-it-GGUF/tree/bfc15c382204943c3a8fff0c750b94ae2364d7a3),
[LFM2.5 files and licence](https://huggingface.co/LiquidAI/LFM2.5-2.6B-GGUF/tree/e7caca5d835a3901a8e0d63e94009429bafafdfc).

### Licence review

The actual [Qwen licence](https://huggingface.co/Qwen/Qwen3.5-4B/blob/851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a/LICENSE)
and [NuExtract licence](https://huggingface.co/numind/NuExtract3/blob/c99dc8f5641b866aa0192b6ea78f84bf9f3535f1/LICENSE)
were retrieved and identify Apache 2.0. It permits commercial use and distribution subject to
its attribution, licence, modification-notice and applicable NOTICE obligations. Gemma's
[official licence](https://ai.google.dev/gemma/docs/gemma_4_license) identifies Apache 2.0;
retain the exact texts and notices with any shipped weights/runtime. The pinned Unsloth cards
also identify Apache 2.0. Check the exact candidate's packaged notices before promotion.

LFM's [actual licence](https://huggingface.co/LiquidAI/LFM2.5-2.6B-GGUF/blob/e7caca5d835a3901a8e0d63e94009429bafafdfc/LICENSE)
conditions commercial rights on annual revenue below $10M; a separate licence is required
above that. Redistribution does not eliminate downstream restrictions. Do not select it for
Intern's unrestricted default on the strength of a small footprint.

Intern is Elastic License 2.0; upstream llama.cpp is MIT. None of the proposed Qwen experiment
requires another inference runtime or a new library. Any future runtime bump must keep its
own licence inventory. This work does not alter Intern's licence or package assets.

## What prior evidence actually establishes

The [old bake-off](model-bakeoff.md) measured Qwen 4B on a Ryzen 8840U: 48 tok/s prefill and
7.3 tok/s generation, versus 157 and 17.5 for 2B. Those are older-runtime measurements, not a
forecast for this checkout. A default replacement must overcome this material latency risk.

The [LFM3 retest](../bench/reports/lfm3-retest-linux-cpu-2026-10-28/README.md) already found a
sidegrade: its best LFM3 arm named 51/77 correctly against 52/77 for Qwen, with fewer dates
correct and slower median latency. Thinking was slower and worse for both. Do not rebrand
those vendor claims as an untested new improvement.

PR [56](https://github.com/z-brenner/intern/pull/56), inspected initially at `052937f` and again
at `c08f31b`, changes evaluation
data and reports, not the engine. It reports only 7/58 adjudicated filenames correct on a harder
third real-document set. No branch, file, review, or result in that workstream was changed.
Use its final gold and all three real-document cohorts as independent checks before promotion.
Its latest revision fixes the readiness-label issue flagged in its review: the third sample
now has five `needs_review` expectations. It also updates the first sample. The first sample
in this branch's base still contains two `review` values the scorer ignores. The new
qualification preflight refuses that spelling. Use the corrected gold once PR 56 lands, or
correct a **separate qualification copy** explicitly to `needs_review` and preserve the changed
gold hash; never silently compare it with old scores or edit PR 56.
`either` remains a valid deliberately unscored routing expectation.

## Integration prepared in this branch

`scripts/cpu-model-eval/candidates.json` pins the exact 2B control and proposed 4B weights by
revision, length and SHA-256. It is laboratory input, never read by desktop setup.

`run.py` verifies both model files, requires committed code, fingerprints the corpus, model,
server and its libraries, parser runtime, worker and benchmark, and records CPU/memory cgroup
limits. It runs sequential **2B, 4B, 4B, 2B** arms with one CPU-only server at a time, the actual
Intern worker/engine, 4 threads, 8,192 context, existing grammar, and confidence threshold 0.5.
It holds an exclusive lock against other invocations of this runner. Keep other model,
worker, compiler and test jobs off the host during measurements: the RSS sampler sums process
names across the host, and CPU contention affects latency.

The empty benchmark profile preserves every production request setting while enabling exact
wire captures. These include document text: use only public/synthetic corpora and keep them
local. They allow auditing malformed replies and retries as well as the final parsed answer.

The candidate shell runner now has the normal app flags by default (`--cache-ram 0 -ub 256
--prio -1 --poll 0`). Previously a caller had to remember several of them. It measures process
startup without requiring the missing `bc` program. Extra flags remain available for experiments.

`compare.py` refuses replay, missing memory/timing/provenance, missing or duplicated documents,
different runtimes/corpora/machines, unequal metric denominators, and unsuccessful or stale
records. Both repeats must improve at least one critical-field/routing count, lose no previously
correct date or party, introduce no trap date or forbidden party, and avoid increases in wrong
names marked ready or decreases in filename/type/description counts. Median inference and total
time must stay within 1.5x control. Sampled combined process RSS must stay below 12 GiB, leaving
4 GiB on a 16 GiB machine. It reports p50/p95 and startup separately. It **never declares release
qualification**: semantic audit, other cohorts and native release evidence remain mandatory.

Provision the pinned model files through an allowed path, build the normal worker/benchmark
and stage the production runtime using the repo's existing scripts, then run:

```bash
npm run bench:generate
cargo build --release --locked -p intern-worker --features windows-native
cargo build --release --locked -p intern-bench --features benchmark-transport
python3 -I scripts/cpu-model-eval/run.py \
  --models /path/to/verified-models \
  --server /path/to/runtime/llama-upstream/llama-server --runtime /path/to/runtime \
  --corpus bench/generated --gold bench/gold.json --manifest bench/manifest.json \
  --threads 4 --out target/cpu-model-internbench
```

Use `--preflight-only` to check assets first. Use a fresh output directory for each entire
repeat. No incomplete run is resumed or silently accepted. Repeat the protocol on the real
document and CUAD corpora; omit `--manifest` only for corpora that have none. Freeze all prompt
and runtime decisions on separate calibration documents **before** heldout runs.

## Optimization experiments, in order

1. **CPU/backend and threads.** Use the packaged b11946 upstream runtime, not this environment's
   preinstalled `14e78dd` build, as the production control. Sweep 2/4 physical threads on
   calibration only. Portable AVX2/AVX-512 dispatch can change speed and outputs; measure both
   laptop paths and record the selected backend. The lab here is limited to 4 CPU cores and
   16 GiB by cgroups, despite the host-level machine metadata.
2. **Quantization kernels.** Compare Q4_K_M with a pinned Q4_0/QAT variant if the publisher has
   one. Do not assume a smaller bit count is faster on a particular CPU. Never requantize an
   already quantized file to claim an accuracy-preserving optimization. The older bake-off's
   Q5 variant halved prefill speed without a measured quality gain.
3. **KV/cache and batching.** On calibration only, test `-fa on`, Q8 KV cache, and microbatch
   128/256/512 one change at a time. Qwen's recurrent state means KV savings may be modest.
   Validate support in the exact runtime and repeat quality checks; cache quantization is not
   automatically neutral. Keep one slot and the bounded cache to prevent resident growth.
4. **Prompt/token cost.** Preserve evidence handles, literal-source validation and grammar.
   A shorter context or reply schema changes model inputs and invalidates recordings; measure
   context recall before shrinking it. Do not cut the current 1,024-token cap on the assumption
   every valid description and evidence list will fit. Thinking is already disabled.
5. **Speculation/distillation later.** Draft-model speculation adds another model and CPU
   contention; acceptance-rate and end-to-end evidence must pay for it. A task-specific
   fine-tune requires a clean training/calibration/heldout split and commercially usable data;
   never train on the heldouts or on private company documents sent to an external service.

Process-cold startup is measured after executable launch. It is not disk-cold startup: model
hash verification itself warms the page cache. True cold-SSD and battery/background usability
measurements require a representative laptop and a separate procedure. The first unscored
InternBench warmup is excluded from the warm document medians. Peak RSS sampling can miss brief
spikes; native Windows peak working-set and OS responsiveness checks must complement it.

## Promotion and release conditions

A 4B default cannot be installed by just changing its URL. `manifest.rs` currently enforces a
2 GiB total download cap, which rejects the 2.74 GB model by itself. Promotion must update that
explicit policy, the accurate model pin/label, model download contract tests and size copy,
`tiers.rs` resident estimates and available-memory guard from **measured peaks**, setup progress,
licence inventory and release evidence. Retain the 2B weights and a supported fallback path;
the optional Bonsai fast tier does not replace the accurate 2B fallback. Design that migration
only once a model wins, avoiding an unqualified third production tier now.

Audit the actual filenames, date roles, party roles and every description against source
documents, especially new ready results and disagreements. Report malformed outputs, retries,
OCR-limited cases and conservative review separately. Scores alone miss semantic errors and
changing the confidence threshold can trade safety for apparent accuracy.

Then run both corpus replays, InternBench, all heldouts, frontend/browser and Rust tests, and
the packaged Windows model qualification. Refresh recordings only from actual qualified
inference, keep the previous baselines for comparison, inspect current main/PR 56 compatibility
again, and satisfy the existing release preflight, version, licence, fidelity and provenance
gates. Only then prepare the next alpha and dispatch the release workflow. No tag, version bump,
release, PR approval or change of default was made here.
