# ATLAS research: candidates, licenses and method

Status, 2026-11-24. This file records which models were considered as students and teachers,
why, under what license, and how each claim in the other files was produced. Results live in
BENCHMARK_RESULTS.md, MODEL_COMPARISON.md and MOE_EVALUATION.md; this file does not repeat them.

## The task, in one line

Fill one four-field JSON form, with a type, a date, up to three parties and a subject, each tied
to an evidence handle, from about 980 tokens of numbered lines, under a GBNF grammar, at
temperature 0 with thinking off, and report token probabilities for the 0.7 review gate
(BASELINE.md, section 1). General benchmark scores have not predicted accuracy on this task in
this repository, so every candidate is judged only on the app's own real development sets.

## Hard requirements a candidate must meet

| Requirement | Source |
|---|---|
| Runs in llama.cpp b11946 or a pinned upgrade, CPU only, Q4_K_M or similar | `src-tauri/resources/model-manifest.json`, `tiers.rs` |
| Honours the grammar and returns token log-probabilities | `client.rs` |
| Thinking can be switched off in the chat template | `client.rs` |
| Fits a 16 GB laptop alongside the app, the OS and a browser; both tiers' downloads under 10 GiB | `manifest.rs` tests |
| License allows commercial use at any company size | repository licence rule, BASELINE.md section 2 |
| Weights downloadable from a pinned, immutable revision with a SHA-256 | `manifest.rs` validation |

## Students considered

All Qwen3.5 sizes below share one tokenizer, chat template and architecture with the shipped
9B and 4B, so a fine-tuned one is a drop-in GGUF: same prompt, grammar, handles and gate.

| Model | Parameters, from the Hub | License | Q4_K_M download | Status |
|---|---|---|---|---|
| Qwen3.5 0.8B | 0.87B | Apache 2.0 | 0.53 GB | Screened; LoRA-trained on CPU once |
| Qwen3.5 2B | 2.27B | Apache 2.0 | 1.28 GB | Screened; the main student candidate |
| Qwen3.5 4B | 4.66B | Apache 2.0 | 2.74 GB | Screened; today's quick tier |
| Qwen3.5 9B | 9.65B | Apache 2.0 | 5.68 GB | Today's accurate tier and the reference |
| Granite 4.0 H Tiny | 6.94B total, about 1B active, hybrid Mamba MoE | Apache 2.0 | 4.23 GB | Screened; ruled out (MOE_EVALUATION.md) |

Earlier repository work already screened Spark-X, Gemma 4 E4B, Nemotron, Granite 8B and Bonsai
off the shelf, and LFM and d1 were excluded on license (BASELINE.md, sections 2 and 7). ATLAS
did not repeat those screens.

The 2B is the main student because it is the smallest model whose untrained gap to the 9B
looks closable: 0.23 of the 9B's time and 1.6 GB of server memory, with 17 documents it gets
right that the 9B misses. The 0.8B is the cheap probe that a free CPU runner can train, used to
learn whether the training data moves real-set accuracy at all.

## Teachers considered

A teacher has to be clearly better than the 9B on the real development sets, because a student
learns its teacher's errors and the switch rule asks the student to match the 9B. None has been
screened yet; that is the first stage of the rented run in DISTILLATION.md.

| Model | Parameters, from the Hub | License | Published GGUF checked | Fits one 48 GB L40S |
|---|---|---|---|---|
| Qwen3.5 27B, dense | 27.8B | Apache 2.0 | unsloth, Q4_K_M 16.7 GB, Q8_0 28.6 GB | Yes, at Q8_0 with room for 4 parallel 8k contexts |
| Qwen3.5 35B-A3B, MoE | 36.0B total, 3B active | Apache 2.0 | unsloth, Q4_K_M 22.0 GB | Yes, at Q4_K_M |
| Qwen3.5 122B-A10B, MoE | 125.1B total, 10B active | Apache 2.0 | unsloth, split GGUFs from about 34 GB at 1-bit; Q4_K_M is several split files | No; needs a larger or multi-GPU host |

The 27B goes first: it is dense, so its quality per byte is the most predictable, it shares the
students' tokenizer exactly, and at Q8_0 it loses almost nothing to quantization. Parameter
counts and licenses above were read from each model's Hugging Face page on 2026-11-24; file
sizes from the unsloth repositories the same day. Paid inference APIs were excluded by the brief.

## How every number in this work was produced

- **Real development sets.** real1 to real6, 349 public documents fetched by pinned URL and
  scored against the repository's gold. Development only, never training data. real7 and real8
  were never screened and the workflows refuse them.
- **Screens.** `.github/workflows/atlas-research.yml` on free GitHub runners: the app's own
  worker, prompt, grammar, llama-server b11946 and 0.7 gate, live, one model per job, every
  model file pinned by revision and SHA-256. Speed comes from a separate job that runs every
  model in turn on one runner so the CPU is the same.
- **Training.** `.github/workflows/atlas-train.yml`: synthetic data generated and verified by
  the engine, LoRA on CPU, conversion and quantization at the app's llama.cpp pin, and an
  untrained control converted the same way (TRAINING.md).
- **Teacher path.** `.github/workflows/atlas-teacher.yml` captures the exact request bodies the
  engine sends, has a real llama-server answer them, then replays the answers, with their token
  probabilities, through the engine for scoring. Only the answering step needs a GPU. Validated
  on the 2B: replay scored 107 right against 104 live on 314 documents (DISTILLATION.md).
- **Derived figures.** `scripts/compare_common.py` for same-document comparisons,
  `scripts/simulate_cascade.py` and `scripts/simulate_hybrid.py` for routing replays. No model
  runs in these; they read committed reports.
- **Statistics.** Model comparisons are paired on the same documents. The 9B moved by one
  document of 301 between two runs, so gaps of 25 or more are real and gaps of a few are noise.
  The first fine-tune's real-set gain was tested with an exact McNemar test.

## What was not done, and why

- **No laptop measurement.** No Windows laptop is reachable from this work. Speed figures are
  ratios on one runner, which should carry over roughly, but that is an expectation.
- **No GPU work.** Teacher screening, teacher labelling, 2B training and QLoRA or full
  fine-tuning all need a GPU and wait on Zack's authorization. The scripts, workflow and price
  are ready (DISTILLATION.md).
- **Error-focused and curriculum training not run.** Both are defined in DISTILLATION.md. On a
  free CPU runner each would be another 0.8B run of about 400 examples, and the first such run
  showed that more synthetic data barely moves real-set accuracy, so these were judged a poor
  use of the remaining free runs until teacher-labelled realistic data exists.
- **No custom MoE trained.** MOE_EVALUATION.md gives the reasons; in short, memory follows total
  parameters and the work is prefill-bound.
