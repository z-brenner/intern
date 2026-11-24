# ATLAS recommendation

Status, 2026-11-24, on the evidence measured so far. Updated when the rented run lands.

## Recommendation

**Keep shipping Qwen3.5 9B as the accurate tier, and pursue one architecture: a Qwen3.5 2B
student distilled from a Qwen3.5 27B teacher on realistic public documents, used first as a
drop-in replacement candidate and second as the first stage of a cascade if its confidence
proves usable.** Do not change production until a student passes the switch rule twice and on
real7 and real8.

Every alternative the brief named was measured and none passes today. Off-the-shelf small
models are 33 to 89 documents short of the 9B. The only MoE that fits a 16 GB laptop is worse
than the dense 2B. A hybrid of today's models, even tuned on its own test documents, is 31 short.
The one direction that has not been ruled out is training, and the free experiment says the
missing ingredient is realistic data, which needs a teacher, which needs a GPU.

## Next step

Approve the first rented stage: screen the 27B teacher on the 315 real development documents,
about $1 and capped at $1.80 on a Hugging Face L40S. Everything else runs free. If the 27B is
clearly better than the 9B's 162, the rest of the plan, labelling about 3,000 public documents
and training 2B students, is about $8 to $11 in total, capped at $12.60 (DISTILLATION.md). If it
is not clearly better, distillation from it is not worth doing, and that $1 has saved the rest.

## Answers to the brief's seven questions

**1. Can a fine-tuned 2-4B model equal or beat the 9B?** Not shown, and not ruled out. Untrained,
the 4B is 33 documents short and the 2B 56 short. The one fine-tune run so far was a 0.8B trained
on CPU with synthetic data: 98.5% right on held-out synthetic documents but only 8 more of 160
real ones, which is not significant. The case for trying is that the 2B and 9B are
complementary: with perfect routing between them, 179 of 313 would be right against the 9B's
160. A 2B student needs teacher-labelled realistic data to test this, which waits on the GPU.

**2. What amount and type of data are needed?** The type matters more than the amount. 396
verified synthetic examples taught the format almost perfectly and moved real accuracy by a
statistically insignificant amount, so more synthetic data of the same kind is unlikely to
close the gap. What is needed is public documents that look like the real sets, scanned and
messy, with labels from a teacher better than the 9B and checked by the engine. The plan uses
about 3,000; that number is an estimate sized to one GPU run, not a measured requirement.

**3. Does distillation beat plain supervised fine-tuning?** Not yet measured. Plain SFT is done;
the distilled comparison uses the same student, script and control with teacher-labelled data
added, and waits on the GPU run.

**4. Does MoE improve the accuracy and speed trade-off?** No. Granite 4.0 H Tiny, the MoE that
fits, named 82 right against the dense 2B's 100, filed 9 wrong names unreviewed, and needs almost
three times the memory for the same speed. MoE saves compute, not memory, and this work is
prefill-bound on a memory-limited laptop. MoEs large enough to compete do not fit 16 GB.

**5. Does adaptive inference beat either architecture alone?** Not with today's models. The best
2B-then-9B routing within the time budget names 129 of 313 against the 9B's 160, and it keeps
all of the 2B's confident wrong filings. The models are complementary enough that perfect
routing would beat the 9B, so the cascade becomes worth building only for a student whose
confidence separates right readings from wrong ones.

**6. What is the fastest architecture that keeps the safeguards?** Today, the 9B with the 0.7
gate, as shipped. Nothing faster passes the switch rule, and the faster models all file more
wrong names unreviewed. The cheapest untried speed lever applies to every model: 61% to 77% of
each document's time is reading a prompt of about 980 tokens, so a shorter prompt would speed up
whichever model ships. That is an engineering change to measure, not a result.

**7. What should ATLAS deploy next?** Nothing yet. The next thing to run is the $1 teacher
screen; the next thing to deploy is a 2B student, in the quick tier if it beats the 4B and in the
accurate tier only if it passes the switch rule, following DEPLOYMENT.md.

## Integration plan, in order

1. Teacher screen on real1 to real6, about $1, with Zack's approval.
2. If the 27B is clearly ahead: build the public document pool with the contamination checks in
   DATASET.md, label it on the GPU, verify every label through the engine.
3. Train 2B students on the GPU: LoRA, QLoRA and full fine-tuning, each with the same untrained
   control. Score all of them free on runners.
4. For the best student: a second development run, then `simulate_hybrid.py` on its readings,
   then a laptop check.
5. Only if it passes: real7 and real8 once, with Zack's say-so, then the manifest change in
   DEPLOYMENT.md behind an alpha, with rollback by reverting that commit.

## What would change this recommendation

- The 27B teacher not clearly beating the 9B: then distillation from open models is off, and the
  realistic options are human-labelled public documents or staying on the 9B.
- A new MoE at about 4 to 8B total that scores near the 9B on real1 to real6: screen it, one line
  in a run spec.
- A laptop measurement showing the runner ratios do not hold: the 0.6 time limit would need
  re-checking on real hardware before any choice.
