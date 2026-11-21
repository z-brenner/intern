# ATLAS model comparison

Measured candidates against the repository's own switch rule (BASELINE.md, section 3.1): a
smaller model replaces the 9B only if, on the same run, it names at least as many documents
right, files no more wrong, and takes at most 0.6 of the 9B's time.

Status, 2026-11-21: off-the-shelf models screened, and the first fine-tuned student
(`2026-11-21-lora1`, Qwen3.5 0.8B) trained and evaluated. This file is updated as runs land.

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

### 2026-11-21-lora1: Qwen3.5 0.8B, LoRA on CPU

Workflow run 44868846152, results in `results/2026-11-21-lora1/`. LoRA rank 16 on every language
model linear layer, 396 verified synthetic examples seen (the 250-minute budget stopped it at 50
of 75 planned optimizer steps, so the learning-rate schedule did not finish). Held-out synthetic
loss on the reply went from 0.712 to 0.013.

Compared with the control (the same base, converted and quantized the same way, no LoRA), live at
the app's 0.7 gate, on documents both scored:

| Set | Docs | Control right | Student right | Student right where control wrong | Student wrong where control right | Student unsafe ready |
|---|---|---|---|---|---|---|
| real4 | 55 | 16 | 18 | 8 | 6 | 1 |
| real5 | 55 | 5 | 8 | 5 | 2 | 1 |
| real6 | 50 | 8 | 11 | 3 | 0 | 0 |
| real4-6 | 160 | 29 (18.1%) | 37 (23.1%) | 16 | 8 | 2 |
| synthetic validation | 200 | 107 (53.5%) | 197 (98.5%) | 92 | 2 | 0 |

What this shows:

- The training data teaches the task. On held-out synthetic documents whose parties never appear
  in training, the student goes from about half right to almost all right and marks 183 of 200
  ready with none wrong.
- Most of that does not carry over to real documents. The real-set gain is 8 documents of 160,
  and with 16 gains against 8 losses it is not statistically distinguishable from no change
  (exact McNemar test, p = 0.15). The gap between 98.5% on synthetic and 23.1% on real documents
  says the synthetic documents are much easier than real ones, so data realism, not only model
  size, is what limits a trained student.
- The student starts filing some documents without review, and 2 of its 4 ready real documents
  are wrong (a garbled title on real4-notice-15, the wrong sender on real5-notice-10). The
  control filed none. Any trained student needs its readiness calibrated before it can file on
  its own.
- The student was faster: each real set finished 22% to 38% sooner in wall time than the control
  on the same kind of runner, consistent with shorter, cleaner replies. That is wall time per
  job, not a controlled speed measurement.

For scale, the untrained 2B scored 39 and the 9B 69 on 153 real4-6 documents in the screen
(different quantization, so only a rough reference).
