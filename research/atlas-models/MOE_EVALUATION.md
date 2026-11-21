# ATLAS: mixture-of-experts evaluation

Question from the brief: would a mixture-of-experts model give 9B-class accuracy at a small
model's speed on a 16 GB CPU-only laptop?

Short answer: not with any MoE that fits, and the one that fits was measured worse than the
dense 2B on every count that matters.

## Measured: Granite 4.0 H Tiny

IBM's Granite 4.0 H Tiny is the most plausible MoE for this hardware: a hybrid of Mamba-2 and
attention layers with mixture-of-experts feed-forward layers, about 7B parameters in total and
about 1B active per token, published as a Q4_K_M GGUF that llama.cpp runs. Screened in run
2026-11-21-screen1 beside the same-run Qwen3.5 9B and 2B (BENCHMARK_RESULTS.md):

| | Qwen3.5 9B | Qwen3.5 2B | Granite H Tiny |
|---|---|---|---|
| Right on 299 real documents | 156 | 100 | 82 |
| Filed under a wrong name without review | 1 | 5 | 9 |
| p50 per document, same runner | 79.5 s | 18.3 s | 20.2 s |
| Server memory | 6.1 GB | 1.6 GB | 4.4 GB |
| Download | 5.7 GB | 1.3 GB | 4.2 GB |

It is slightly slower than the dense 2B, needs almost three times its memory and download,
names 18 fewer documents right, and files the most documents unsafely of the four models. Its
speed is what a 1B-active model should give; its accuracy is not that of a 7B. It is not a
candidate.

## Why MoE does not fit this problem well

- **Memory follows total parameters, not active ones.** Every expert has to be resident,
  because any token can route to any expert. A MoE saves compute, not RAM or download. On a
  16 GB laptop that already runs the app, an OS and a browser, the total size is the binding
  limit.
- **The MoEs that might match the 9B do not fit.** Qwen3.5 35B-A3B (35B total, 3B active)
  is about 20 GB at 4 bits before any context, and the Qwen3.5 122B and 397B MoEs are far
  larger. They are useful as teachers on rented hardware, not as the shipped model.
- **The work is prefill-bound.** About 980 prompt tokens per document and a short JSON
  answer: 61% to 77% of the time is reading the prompt (BENCHMARK_RESULTS.md).
  A sparse model's advantage is per generated token on memory-bandwidth-bound hardware; it
  helps less when the cost is a batch of prompt tokens on a CPU.
- **Task-specific routing is not a lever we have.** Training our own expert layout means
  pre-training-scale work; fine-tuning an existing MoE keeps its size problem.

## What would change this conclusion

A MoE published at roughly 4-8B total and 1-2B active that scores near the 9B on the real
sets would be worth screening; `atlas-research.yml` takes it as one more candidate line in a
run spec. Until then the evidence points to a dense 2B-class student, trained for this task,
with the 9B kept for the documents the student is unsure about (MODEL_COMPARISON.md).
