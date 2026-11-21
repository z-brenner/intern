# ATLAS model research

Research workspace for finding ATLAS's next local document model: a student smaller and faster
than Qwen3.5 9B that names documents at least as well on a 16 GB, CPU-only Windows laptop.
Nothing here is used by the app. No model weights or training data are committed.

| File | What it holds | Status |
|---|---|---|
| [`BASELINE.md`](BASELINE.md) | What the model does today, how well, how fast, from committed measurements | Written 2026-11-21 |
| `scripts/simulate_cascade.py` | Offline 2B/4B to 9B escalation replay of committed readings | Runs, no model needed |
| `DATASET.md`, `TRAINING.md`, `DISTILLATION.md` | Training data and fine-tuning | Next |
| `MOE_EVALUATION.md`, `BENCHMARK_RESULTS.md`, `MODEL_COMPARISON.md` | Measured comparisons | After runner screens |
| `DEPLOYMENT.md`, `RECOMMENDATION.md` | What to ship and how | Last |

Limits this work keeps: no paid GPU or inference APIs without Zack's authorization; no private
documents sent anywhere; real7 and real8 are never screened; the production model, gate and
pipeline are untouched; no result is reported that was not run.
