# ATLAS model comparison

Measured candidates against the repository's own switch rule (BASELINE.md, section 3.1): a
smaller model replaces the 9B only if, on the same run, it names at least as many documents
right, files no more wrong, and takes at most 0.6 of the 9B's time.

Status, 2026-11-21: off-the-shelf models screened; the first fine-tuned student
(`2026-11-21-lora1`, Qwen3.5 0.8B) is training. This file is updated as runs land.

## Off-the-shelf models (run 2026-11-21-screen1)

| Model | Right (of 299) | vs 9B | Unsafe ready | Time vs 9B | Passes the rule |
|---|---|---|---|---|---|
| Qwen3.5 9B (today's accurate tier) | 156 | | 1 | 1.00 | reference |
| Qwen3.5 2B | 100 | -56 | 5 | 0.23 | no: accuracy and unsafe filing |
| Granite 4.0 H Tiny (MoE) | 82 | -74 | 9 | 0.25 | no |
| Qwen3.5 0.8B | 67 | -89 | 0 | 0.12 | no: accuracy |

Speed is from the same-runner speed job; accuracy from the six real development sets on
documents every model scored. Not measured on a laptop.

## What the gap looks like

- The 2B is fast enough by a wide margin (0.23 of the 9B's time, far under 0.6) and is
  56 documents short. That is the gap a trained student has to close.
- The gap is not random noise: the 2B gets 17 documents right that the 9B misses and misses 73
  the 9B gets. A student that kept the 2B's wins and recovered most of its losses would pass.
- Speed headroom is large enough for a hybrid: the 2B plus the 9B on a minority of documents
  can stay under 0.6 of the 9B's time if the minority is under about 35% and the 2B can tell
  which documents those are. Today's readiness signal cannot: the 2B marks only 17 of 299 as
  ready, and 5 of those are wrong (BASELINE.md, section 5, found the same).
- The 0.8B is the speed and memory floor (0.12 of the 9B's time, under 1 GB) but starts 89
  documents behind. It is in the study because it is the student a free CPU runner can train,
  so it shows cheaply whether this training data moves accuracy at all.

## Fine-tuned students

| Student | Base | Training | Right vs its own untrained control | Status |
|---|---|---|---|---|
| 2026-11-21-lora1 | Qwen3.5 0.8B | LoRA r16, up to 600 verified synthetic examples, CPU | pending | training |
