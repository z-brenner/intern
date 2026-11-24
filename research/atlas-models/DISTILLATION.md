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

## The rented GPU run: plan and price

Nothing below has been rented, paid for or run. Prices were read on 2026-11-24 from the
providers' own public pages; every duration is an estimate, not a measurement.

### Where the GPU is needed, and where it is not

Only answering requests with the 27B teacher, and training the student faster than a free CPU
runner can, need a GPU. Everything else runs free on GitHub runners:

1. **Capture, free.** `.github/workflows/atlas-teacher.yml` runs the real worker over each real
   set against `dataset/fake_server.py` and keeps the exact request the engine sends for each
   document (`results/teacher/<run>/<set>/requests.jsonl.gz`).
2. **Answer, rented.** `teacher/hf_job.py` runs on a Hugging Face Jobs GPU inside
   `ghcr.io/ggml-org/llama.cpp:full-cuda-b11946` (the app's own llama.cpp build), downloads the
   teacher, sends each captured request unchanged to llama-server through `teacher/answer.py`,
   and prints the answers, compressed, in the job log. Nothing is written to any account.
3. **Score, free.** `teacher/unpack_log.py` turns the log back into answers files; a `score`
   run of the same workflow serves them through `fake_server.py`, which now also replays the
   model's own token probabilities, so the engine parses, gates and scores each reply exactly
   as it would live.

The first `local` run (`teacher-runs/2026-11-24-local2b.json`, workflow run 45441020012) tested
this path for free: the 2B answered every captured request on a CPU runner through `answer.py`,
and the answers were scored by replay. Against the 2B's live run in screen1, on the 314
documents both scored:

| | Right | Ready | Unsafe ready |
|---|---|---|---|
| Qwen3.5 2B, live (screen1) | 104 | 17 | 5 |
| Qwen3.5 2B, answered by `answer.py`, scored by replay | 107 | 16 | 7 |

314 of the 320 prompts were byte for byte the ones the live run sent (the six others differ in
their extracted text), so capture reproduces the app's requests. The totals agree within a few
documents, but the two runs disagree on 25 individual documents (14 one way, 11 the other),
more than the 9B's 7 of 301 between its two screens. The runs landed on different runner CPUs,
whose arithmetic differs slightly, and a 2B is likely more sensitive to that than a 9B; the
replay jobs did not record their CPU, which the workflow now does. For the teacher screen this
means a difference of a few documents is noise, and only a clear lead over the 9B counts.

Its captured requests (`results/teacher/2026-11-24-local2b/real1` to `real6`) are the ones the
GPU job answers.

### Hardware

| Provider, from its pricing page | GPU | GPU memory | Price per hour | Billing |
|---|---|---|---|---|
| Hugging Face Jobs, `l40sx1` | 1x L40S | 48 GB | $1.80 | per minute; needs a positive credit balance on the account |
| Hugging Face Jobs, `a100-large` | 1x A100 | 80 GB | $2.50 | per minute |
| Hugging Face Jobs, `l4x1` | 1x L4 | 24 GB | $0.80 | per minute |
| Runpod, L40S pod | 1x L40S | 48 GB | $1.09 | per second; needs a new Runpod account |

Chosen: Hugging Face Jobs `l40sx1`. The Hugging Face account is already connected, so no new
account is opened, and 48 GB holds the teacher at 8 bits (`unsloth/Qwen3.5-27B-GGUF`,
`Qwen3.5-27B-Q8_0.gguf`, 28.6 GB, Apache 2.0) with room for four parallel requests, and a full
fine-tune of the 2B in bf16. The 24 GB L4 would force the teacher down to 4 bits, which risks
the accuracy the teacher is there for. Runpod is about 40% cheaper but needs a new account.

### Stages, durations and cost

| Stage | What runs | Estimated GPU time | Estimated cost | Job timeout, so the most it can cost |
|---|---|---|---|---|
| 1. Teacher screen | 27B answers the 315 captured real1 to real6 requests | 30 to 40 min, including about 15 min to start and download | about $1.00 | 60 min, $1.80 |
| 2. Labelling | 27B answers about 3,000 public documents | 2.5 to 4 h | $4.50 to $7.20 | 4 h, $7.20 |
| 3. Students | 2B with LoRA, QLoRA and full fine-tuning on about 4,000 examples, 2 epochs each | about 1.5 h | about $2.70 | 2 h, $3.60 |
| 4. Student scoring | GGUF conversion and live scoring on real4 to real6 | none, free CPU runners | $0 | |
| Total | | about 5 to 6 h | about $8 to $11 | $12.60 |

Stage 2 runs only if stage 1 shows the 27B naming clearly more documents right than the 9B
(162 of 315 on the same documents in screen2); otherwise the study stops at about $1. Stage 2
also needs a pool of public documents that do not overlap real1 to real8, which is free work
not yet done. Stage 3 needs somewhere to put the trained weights, which are too large for a job
log: a private model repository on the connected Hugging Face account, created only once that
stage is approved.

How the durations were estimated: each request is about 1,000 prompt tokens and about 110
generated tokens (screen runs). An L40S reads an 8-bit 27B at an estimated 20 to 25 tokens per
second per request, and four parallel requests share the prompt work. The 2B training estimate
scales the CPU run's measured work (6 x 2B parameters x about 1,100 tokens per example) to an
L40S at an assumed 30% of its bf16 peak.

### Command for stage 1, once approved

After a `capture` or `local` run has committed the requests at commit `C`:

```
hf jobs run --flavor l40sx1 --timeout 60m \
  -e ATLAS_COMMIT=C \
  -e ATLAS_REQUESTS=research/atlas-models/results/teacher/2026-11-24-local2b/real1/requests.jsonl.gz,...(real2 to real6 the same way) \
  -e MODEL_REPO=unsloth/Qwen3.5-27B-GGUF -e MODEL_FILE=Qwen3.5-27B-Q8_0.gguf -e PARALLEL=4 \
  ghcr.io/ggml-org/llama.cpp:full-cuda-b11946 \
  python3 -c "import urllib.request; exec(urllib.request.urlopen('https://raw.githubusercontent.com/z-brenner/intern/C/research/atlas-models/teacher/hf_job.py').read())"
```

Then `hf jobs logs JOB > job.log`, `python3 -I teacher/unpack_log.py job.log
results/teacher/2026-11-2x-teacher27b/answers-teacher-qwen3.5-27b-q8`, commit, and push a
`score` spec naming that directory.

One unverified assumption: that Hugging Face Jobs replaces the image's entrypoint
(`/app/tools.sh`) with the given command. If it does not, the job stops within a minute with
"Unknown command" and costs a few cents; the fallback is the same script in
`nvidia/cuda:12.8.1-devel-ubuntu24.04` with llama.cpp b11946 built from source, which adds
about 15 minutes (about $0.45).
