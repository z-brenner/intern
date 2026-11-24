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
| Qwen3.5 4B (today's fast tier, screen2, 315 docs) | 129 of 315 | -33 vs 162 | 7 vs 4 | 0.61 | no: accuracy, unsafe filing, and just over the time limit |
| Qwen3.5 2B | 100 | -56 | 5 | 0.23 | no: accuracy and unsafe filing |
| Granite 4.0 H Tiny (MoE) | 82 | -74 | 9 | 0.25 | no |
| Qwen3.5 0.8B | 67 | -89 | 0 | 0.12 | no: accuracy |

Speed is from the same-runner speed job; accuracy from the six real development sets on
documents every model scored. The 4B row comes from a second run with its own same-run 9B. On
the 301 documents both runs share, the ladder is 9B 155-156, 4B 124, 2B 101. Not measured on a
laptop.

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

## Adaptive hybrid: small model first, 9B on the unsure documents

Derived 2026-11-24 from the committed screens, no model run. `scripts/simulate_hybrid.py`
replays each run's per-document readings: the small model reads every document, and any
document whose least probable chosen token is under a threshold goes to the same-run 9B as
well. The threshold is swept; cost is the small model's measured time ratio plus the share
escalated. Counts here are on the documents the small model and the 9B both scored, so the
totals differ slightly from the four-model table above.

```
python3 -I scripts/simulate_hybrid.py results/2026-11-21-screen1 --quick control-qwen3.5-2b-q4km --reference control-qwen3.5-9b --ratio 0.23
python3 -I scripts/simulate_hybrid.py results/2026-11-23-screen2 --quick fast-qwen3.5-4b-q4km --ratio 0.61
```

| First model | Docs | 9B alone | Best hybrid within 0.6 of the 9B's time | Escalated | Unsafe ready, hybrid vs 9B | Oracle escalation |
|---|---|---|---|---|---|---|
| Qwen3.5 2B | 313 | 160 | 129 | 34% | 5 vs 2 | 179 right at 0.47 |
| Qwen3.5 0.8B | 311 | 163 | 120 | 47% | 0 vs 1 | 171 right at 0.44 |
| Granite 4.0 H Tiny | 315 | 162 | 115 | 32% | 10 vs 2 | 175 right at 0.54 |
| Qwen3.5 4B | 315 | 162 | none: the 4B alone already costs 0.61 | | | 179 right at 0.77 |

What this shows:

- No off-the-shelf hybrid passes the switch rule, even with its threshold fitted on the very
  documents it is scored on, which flatters it. The best one, 2B first, is 31 documents short.
- The routing signal is the bottleneck, not the models. If the 2B could tell which of its
  readings are wrong and fixable, escalating exactly those 24% of documents would name 179 right,
  19 more than the 9B alone, at under half its time. The 2B's token confidence finds those
  documents only a little better than chance: escalating the least confident 34% nets 24 more
  right names, where escalating a random 34% would be expected to net about 19.
- Escalating on low confidence never removes the 2B's confident wrong filings: its 5 unsafe
  filings survive every threshold the budget allows. A hybrid cannot be made safe by routing alone;
  either the small model's readings are always held for review, as the app's quick tier already
  does, or the small model's readiness has to be calibrated separately.
- The two models are complementary: the 9B plus the 2B's 17 wins over it is the source of the
  oracle's 179. That is the case for a trained student whose confidence means something, not for
  the stock 2B.

So the answer to "does adaptive inference beat either architecture alone" is: not with today's
models, and the measurement says exactly what a student would have to add, namely a confidence
that separates its right readings from its wrong ones. The same script scores any future student
run unchanged.
