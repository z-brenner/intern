# ATLAS distillation: teachers and method

Status, 2026-11-23: no teacher has labelled any document yet, and no distilled student exists.
The only trained student so far (`2026-11-21-lora1`, MODEL_COMPARISON.md) is plain supervised
fine-tuning on synthetic documents whose labels come from the generator's own gold, checked by
the engine. This file records what that run implies for distillation and the plan that waits
on GPU time.

## What the first run showed

- Training on verified ATLAS exchanges works on documents like the training data: Qwen3.5 0.8B
  went from 107 to 197 of 200 held-out synthetic documents named right.
- It barely carries to real documents: 29 to 37 of 160 on real4 to real6, which is not
  statistically distinguishable from no change (p = 0.15).

So the limiting factor is how much the training documents look like real ones, not whether the
model can learn the output format. A teacher matters because it is the way to label realistic
documents that no generator wrote and no human has labelled.

## Why the 9B is not enough as a teacher

On the 299 real development documents the 9B names 156 right (52.2%). A student trained on its
labels inherits its errors, and the switch rule asks the student to match the 9B, so a teacher
has to be clearly better than the 9B on these documents, not just bigger. Every teacher
candidate is first screened on real1 to real6 at the app's own prompt and grammar, exactly as
the students are, before it labels anything.

## Teacher candidates, in screening order

| Candidate | Why | Runs where |
|---|---|---|
| Qwen3.5 27B (dense) | Same family, tokenizer and chat template as the students, so its replies need no translation | One 24 GB GPU at 4-bit, or slowly on a large CPU |
| Qwen3.5 35B-A3B (MoE) | 3B active parameters, so fast per token once loaded; about 20 GB at 4-bit | One 24 GB GPU, or a 32 GB CPU host |
| Qwen3.5 122B-A10B (MoE) | Strongest open Qwen that is still practical to rent | Multi-GPU or a large-memory host |

Licenses, exact hardware and published GGUFs for each are to be confirmed in RESEARCH.md
before any is run. Paid inference APIs are excluded without Zack's authorization.

## Labelling method

1. **Documents.** Public documents of the same kinds as the real sets, from the same kinds of
   public sources, refused if they match any real1 to real8 document by source URL, content hash
   or near-duplicate text, or share a party with their gold. No private document is used.
2. **Exchanges.** The real worker extracts the evidence exactly as `dataset/fake_server.py`
   captures it today, so the prompt is byte for byte what the app sends.
3. **Teacher replies.** Each teacher answers at temperature 0 with the app's grammar. Where two
   teachers are screened, both answer.
4. **Verification.** A reply is kept only if every cited handle exists and prints the value it
   is cited for, the engine accepts it, and the composed name is well formed. Where teachers
   disagree, the document goes to a review list, not into training. Agreement between two models
   is treated as a signal, not as proof.
5. **Training.** Same script and control as `2026-11-21-lora1`, on a mix of verified teacher
   examples and the synthetic set, then the same live evaluation on real4 to real6 and the
   synthetic validation split. real1 to real3 stay development sets; real7 and real8 stay unused
   until a final answer.

## Comparisons the brief asks for

| Comparison | How it is measured | Status |
|---|---|---|
| Plain SFT on verified synthetic data | `2026-11-21-lora1` | done (0.8B, CPU) |
| Teacher-assisted distillation | same student, teacher-labelled public data added | waits on GPU |
| Error-focused examples | extra synthetic families built from the real-set misses in BASELINE.md | not started |
| Curriculum | same data, easy-to-hard order vs shuffled | not started |
| LoRA vs QLoRA vs full | LoRA done; QLoRA and full need a GPU | waits on GPU |
