# ATLAS model research

Research workspace for finding ATLAS's next local document model: a student smaller and faster
than Qwen3.5 9B that names documents at least as well on a 16 GB, CPU-only Windows laptop.
Nothing here is used by the app. No model weights or training data are committed.

| File | What it holds | Status |
|---|---|---|
| [`BASELINE.md`](BASELINE.md) | What the model does today, how well, how fast, from committed measurements | Written 2026-11-21 |
| `scripts/simulate_cascade.py` | Offline 2B/4B to 9B escalation replay of committed readings | Runs, no model needed |
| [`DATASET.md`](DATASET.md), `dataset/` | Synthetic documents, labels verified by the engine itself, exported as the exact production exchange | Built and verified locally (1,000 of 1,000) |
| [`TRAINING.md`](TRAINING.md), `training/`, `train-runs/` | CPU LoRA fine-tune, GGUF conversion at the app's llama.cpp pin, student vs. untrained control | First run started 2026-11-21 |
| [`DISTILLATION.md`](DISTILLATION.md), `teacher/`, `teacher-runs/` | Teacher choice, labelling method, and the rented-GPU plan with its price; capture and replay scoring run free (`atlas-teacher.yml`), only the teacher's answers need a GPU | Plan and price written 2026-11-24; nothing rented |
| [`MOE_EVALUATION.md`](MOE_EVALUATION.md), [`BENCHMARK_RESULTS.md`](BENCHMARK_RESULTS.md), [`MODEL_COMPARISON.md`](MODEL_COMPARISON.md) | Measured comparisons | Screens 1 and 2 and the first fine-tune measured |
| `DEPLOYMENT.md`, `RECOMMENDATION.md` | What to ship and how | Last |

Limits this work keeps: no paid GPU or inference APIs without Zack's authorization; no private
documents sent anywhere; real7 and real8 are never screened; the production model, gate and
pipeline are untouched; no result is reported that was not run.
