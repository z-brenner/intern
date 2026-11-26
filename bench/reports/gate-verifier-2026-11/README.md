# A learned verifier for the review gate, November 2026

Status: measured on 2026-11-23. **Negative result.** A small learned model that judges Qwen's
reading did not beat the token probability the gate already uses. It is worse at every volume
tried. Nothing here changes the engine.

## The question

Can a cheap classifier, run after the 9B's reply, decide which readings to file without a
person better than the least token probability does? If it could, the gate would hold fewer
good names (the over-flagging) at the same or lower risk.

## Data

Development sets only, one recording per set, every reading replayed through this branch's
engine (alpha.21) at the shipped gate of 0.7 and scored on the review-levels severity scale
(`../review-levels-2026-11/policy/severity.py`):

| Set | Recording (candidate-bench run) |
| --- | --- |
| InternBench | 44162258206 |
| real1 to real4 | 42983156560 (`--no-repack`, token traces kept) |
| real5, real6 | 43302007759 (alpha.18 engine at record time) |

That makes 412 completed readings: 245 right, 73 minor, 94 really wrong. real7 and real8 were
not used.

## Where today's gate holds good names (development, 0.7 gate)

| Outcome | Right | Minor | Really wrong |
| --- | ---: | ---: | ---: |
| Filed without review | 69 | 0 | 2 |
| Held by token probability alone | 61 | 9 | 10 |
| Held by an engine reason | 115 | 64 | 41 |
| Held: no date in the name | 0 | 0 | 41 |

Most good names are held by engine reasons, not by the token gate. `PARTY_UNSUPPORTED` is on
143 held readings (68 right) and is the only reason on 62 of them (36 right, 13 really wrong).

## The verifier

- **Features (95)**: least and mean token probability; every review reason; support and
  citation counts; document class; relation basis and role support; OCR share, pages, context
  size; type and title agreement; how many dates the text holds, cues on the chosen date's line,
  whether its year is the latest in the text; where the named parties sit and whether they read
  as organizations. Also per-field token probabilities from the traces (type, date, party,
  role, subject: minimum, mean, top-two margin).
- **Models**: LightGBM (5 leaves, 150 trees) and L2 logistic regression.
- **Label**: really wrong (the name would mislead a filing clerk).
- **Validation**: leave one set out. The scores of each set come from a model that never saw it.
- **Two pools**: v1 replaces only the token gate (dated readings with no engine reason, 148
  readings, 11 really wrong). v2 replaces the token gate and balanced's soft reasons (315
  readings, 32 really wrong).

## Result

At a matched number of names filed, the verifier picks worse names than token probability
alone (out-of-sample, pooled over the seven folds):

| Pool | Names filed | Verifier: right / minor / wrong | Token probability: right / minor / wrong |
| --- | ---: | --- | --- |
| v1 | 50 | 46 / 2 / 2 (LightGBM) | 50 / 0 / 0 |
| v1 | 75 | 67 / 3 / 5 | 74 / 0 / 1 |
| v1 | 100 | 90 / 4 / 6 | 94 / 2 / 4 |
| v2 | 50 | 44 / 2 / 4 (logistic) | 49 / 1 / 0 |
| v2 | 100 | 85 / 9 / 6 | 90 / 6 / 4 |
| v2 | 150 | 120 / 18 / 12 | 129 / 13 / 8 |

AUC for really wrong within the pool: v1 0.57 (verifier) against 0.77 (token probability);
v2 0.60 against 0.66. Each per-field trace signal on its own is weaker than the overall least
probability (v1: date 0.66, subject 0.63, type 0.53, party 0.51). Every row is in
`curves-dev.json`.

## Why it fails

- **Too few errors to learn from.** The pool where the gate decides holds 11 really-wrong
  readings in development. 95 features against 11 positives overfits, and leave-one-set-out
  shows it.
- **The signals are not independent.** Every feature is derived from the same reply and text
  that the engine's rules already read. What they would add, the rules have mostly already
  turned into review reasons.
- **The token probability is a better ranker than its reputation.** Within the v1 pool it
  files 74 names with 1 really wrong before its quality drops.

## What it means for Laya

A Laya fine-tune would face the same 11 to 32 positive examples. Its ModernBERT encoder reads
the text directly, which the trees cannot, but the label scarcity is the binding limit. Laya
was not run: this environment cannot reach huggingface.co. It is not recommended for the gate
until there are several hundred labeled errors, for example from review-panel corrections.

## Reproducing

The recordings are in this branch's history at commit `3622d8be`
(`bench-data/recordings/*.zip`). For each set:

    intern-bench run --corpus EMPTY --gold GOLD.json --replay recording.json \
      --min-token-confidence 0.7 --allow-stale --output REPORT.json

The real-set gold is the sample files' own (`scripts/realdocs-eval/sample*.json`, minus each
entry's `source`). Then `python3 curves.py SETS.json OUT` with SETS.json listing
`{set, report, recording, gold}` per set. The pinned versions were LightGBM 4.7.0 and
scikit-learn 1.9.1.
