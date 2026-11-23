# ATLAS benchmark results

Everything here was measured on GitHub's free Linux runners (AMD EPYC 7763, 4 vCPU, 16 GB),
live, with the app's own worker, prompt, grammar, llama-server build (llama.cpp b11946) and
0.7 review gate. Not one number here comes from a Windows laptop; laptop speed remains
unmeasured (see "What these numbers are not").

## Run 2026-11-21-screen1

Spec: `runs/2026-11-21-screen1.json`. Workflow run 44867542540. Results:
`results/2026-11-21-screen1/`. Every model file is pinned by commit and SHA-256 in its
`model.json`.

| Label | File | Bytes |
|---|---|---|
| control-qwen3.5-9b-q4km | unsloth/Qwen3.5-9B-GGUF, Q4_K_M | 5.68 GB |
| control-qwen3.5-2b-q4km | unsloth/Qwen3.5-2B-GGUF, Q4_K_M | 1.28 GB |
| qwen3.5-0.8b-q4km | unsloth/Qwen3.5-0.8B-GGUF, Q4_K_M | 0.53 GB |
| granite-4.0-h-tiny-q4km | ibm-granite/granite-4.0-h-tiny-GGUF, Q4_K_M (hybrid Mamba MoE, about 7B total, about 1B active) | 4.23 GB |

### Accuracy on the real development sets, same documents for every model

A document that failed to download in one job is dropped from that set for every model, so
all four are scored on the same 299 documents (`scripts/compare_common.py`). "Right" means the
file name the app would propose is scored right. "Ready" means it would be filed without
review. "Unsafe ready" means it would be filed without review under a wrong name.

| Model | Right of 299 | Right % | Ready | Unsafe ready | Right where 9B wrong | Wrong where 9B right |
|---|---|---|---|---|---|---|
| Qwen3.5 9B | 156 | 52.2 | 21 | 1 | | |
| Qwen3.5 2B | 100 | 33.4 | 17 | 5 | 17 | 73 |
| Granite 4.0 H Tiny (MoE) | 82 | 27.4 | 18 | 9 | 13 | 87 |
| Qwen3.5 0.8B | 67 | 22.4 | 0 | 0 | 8 | 97 |

Per set (right of common documents):

| Set (docs) | 9B | 2B | Granite H Tiny | 0.8B |
|---|---|---|---|---|
| real1 (53) | 31 | 22 | 20 | 13 |
| real2 (49) | 33 | 22 | 17 | 11 |
| real3 (44) | 23 | 17 | 9 | 12 |
| real4 (52) | 31 | 19 | 21 | 19 |
| real5 (55) | 18 | 8 | 6 | 6 |
| real6 (46) | 20 | 12 | 9 | 6 |

On real4 to real6 only (153 documents, the sets least tied to how the training data was
designed): 9B 69, 2B 39, Granite 36, 0.8B 31.

### Speed and memory, every model in turn on one runner

The InternBench synthetic set (77 documents), all four models on the same runner one after
another, so the CPU is the same for each.

| Model | p50 per document | p95 | Prefill p50 | Decode | Server peak RSS | Time vs 9B |
|---|---|---|---|---|---|---|
| Qwen3.5 9B | 79.5 s | 173.0 s | 60.5 s | 4.8 tok/s | 6.1 GB | 1.00 |
| Qwen3.5 2B | 18.3 s | 42.5 s | 13.1 s | 17.8 tok/s | 1.6 GB | 0.23 |
| Granite 4.0 H Tiny | 20.2 s | 43.4 s | 15.4 s | 21.0 tok/s | 4.4 GB | 0.25 |
| Qwen3.5 0.8B | 9.6 s | 20.9 s | 5.8 s | 31.4 tok/s | 0.86 GB | 0.12 |

Accuracy on that synthetic set, for completeness: 9B 72 of 77, 2B 67, Granite 54, 0.8B 41.
InternBench barely separates the 9B from the 2B, as BASELINE.md found; the real sets do.

## Run 2026-11-23-screen2: the shipped 4B fast tier

Spec: `runs/2026-11-23-screen2.json`. Workflow run 45219400851. Results:
`results/2026-11-23-screen2/`. Qwen3.5 4B at the app's own manifest pin (unsloth/Qwen3.5-4B-GGUF,
Q4_K_M, 2.74 GB) against a same-run Qwen3.5 9B.

### Accuracy, same 315 real documents

| Model | Right | Right % | Ready | Unsafe ready | Right where 9B wrong | Wrong where 9B right |
|---|---|---|---|---|---|---|
| Qwen3.5 9B | 162 | 51.4 | 29 | 4 | | |
| Qwen3.5 4B | 129 | 41.0 | 25 | 7 | 17 | 50 |

The 4B is behind on every one of the six sets (by 3 to 7 documents per set).

### Run-to-run variation of the 9B

The 9B ran in both screens. On the 301 documents both runs scored, it named 155 right in the
first and 156 in the second, and the two runs disagreed on 7 documents. So a gap of a few
documents between models is within noise, and the gaps measured here (25 or more) are not.
On those same 301 documents: 9B 155 and 156, 4B 124, 2B 101.

### Speed and memory, both models on one runner

This run's speed job landed on an Intel Xeon 6975P-C runner, not the AMD EPYC 7763 of the first
screen, so its seconds are not comparable with the first screen's; the ratio between the two
models on the same runner is.

| Model | p50 per document | p95 | Server peak RSS | Time vs 9B |
|---|---|---|---|---|
| Qwen3.5 9B | 49.3 s | 97.9 s | 6.2 GB | 1.00 |
| Qwen3.5 4B | 29.9 s | 58.4 s | 3.3 GB | 0.61 |

## What these numbers are not

- **Not laptop speed.** A runner has 4 vCPUs of a server chip and Linux. A 16 GB Windows
  laptop will be slower or faster depending on its cores and its power plan. The ratios between
  models are the useful part; they should carry over roughly, because every model ran the same
  way on the same CPU, but that is an expectation, not a measurement.
- **One run per candidate.** The repo's switch rule asks for two runs. The 9B, run twice, moved
  by one document of 301 (see screen2), so one run is enough to separate gaps of 25 or more but
  not small ones.
- **Prefill dominates.** For every model, 61% (0.8B) to 77% (Granite) of the median time per
  document is reading the prompt (median 978 tokens of instructions and evidence with the Qwen
  tokenizer), not writing the answer.
  That matters for speed work: a shorter prompt helps every model.
