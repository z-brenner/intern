# Making Intern more accurate: research, October 2026

Status: research summary plus the first measured change (2026-10-29). The question was
"there has to be something we can do model-wise or pipeline-wise to increase accuracy".
Three research passes covered:
- other models;
- fine-tuning;
- pipeline and prompting techniques.

Their findings were checked against Intern's own data. Each finding below says whether it
was **measured here**, **checked** (a fact verified on the web or in the code) or **reported**
(a researcher's citation, not independently re-read: treat its numbers as leads, not facts).

## Where the errors are (measured here)

Qwen3.5-2B, production prompt, 77-document InternBench, before the change below: **52/77
names right; 15 wrong names marked ready** (filed without review). The gold type, date and
party were present in the model's context for **77/77** documents, so the errors are
choices, not missing information.

| Bucket | Wrong names | Of which ready |
|---|---:|---:|
| A person who works for the organization named instead of it (buyer contact, board president, conservator, inspector, signatories, winemaker, CEO) | 9 | 7 |
| Generic or wrong type ("Letter", "Agreement", "email"; prompt words echoed: "Evidence") | 7 | 1 |
| Relation word: 3 "to" where the gold says "for", 2 direction reversed | 5 | 5 |
| Date | 1 | 0 |
| Wording | 3 | 2 |

## What was tried and measured

| Change | Result | Verdict |
|---|---|---|
| **Organization over staff, record relation "for"** (deterministic, `facts.rs`/`compose.rs`, PR #40) | Qwen production runtime **52 → 60** names, unsafe **17 → 10**; reference runtime 52 → 61, 15 → 7; no right name lost on any of 8 recorded runs × 77 documents (5 models) | **Shipped** |
| Whole documents instead of retrieval (64K context) | 0 fixed, 2 new unsafe errors (a trap date, a CEO as party); 2–20x slower; the 60–100-page documents timed out or did not fit | Rejected |
| Thinking mode, 1,024-token budget (LFM3 and Qwen) | Both worse (Qwen 52 → 47, unsafe 15 → 21), ~5x slower; neither ever finished reasoning in budget | Rejected (`bench/reports/lfm3-retest-linux-cpu-2026-10-28/`) |
| LFM3-1.7B QAD-Q4_0 as the namer | 51/77, ties Qwen; LFM licence caps commercial use at US$10M revenue | Rejected |

## Pipeline techniques, ranked by expected gain per cost

Ranking from the research pass, adjusted for what was measured since.

1. **Organization outranks a person acting for it.** Shipped in PR #40, measured above.
2. **Class-specific relation words.** The record part shipped in PR #40. Two directional misses
   remain, and they may be the model mislabelling roles:
   - a purchase order "from" its supplier;
   - a loss notice "from" the insurer.
3. **Better routing from token log-probabilities.** Score the role token and each party's first
   token, request `top_logprobs` 5 for a top-two margin, and hold a document when the model's
   party disagrees with item 1's candidate. Reported: log-prob confidence alone reaches
   AUROC 0.65–0.83 on 7–8B models (arXiv 2605.02241), and agreement between two
   differently-built readings helped most in ExtractConf (2606.24420). Cost about 0 s.
   Next to try.
4. **Party names constrained by the grammar to candidate strings.** This adds a typed
   candidate block, organizations first. Reported: a candidate grammar took LLaMA-7B entity
   disambiguation from 37.4 to 77.2 (arXiv 2305.13971). It is less likely to help here,
   since Intern's errors are choices between real names, not invented ones. Cost +50–150
   prompt tokens.
5. **Organization-first output key and per-class role lists in the grammar.** The direction of
   the effect is inconsistent in the literature; it needs a live run.
6. **Remove words the model echoes from the prompt scaffolding.** "BEGIN EVIDENCE" and "The whole
   N-page document follows" became types and parties in held documents. Cost none; needs a
   live run.
7. **Type chosen from the document's title candidates.** Rules out a bare "Letter",
   "Agreement" or "Report". Targets about 5–7 held misses.
8. **A small NER model to tell people from organizations.** GLiNER family, Apache-2.0, ONNX.
   Only if 1 and 4 misclassify.
9. **Same-model yes/no verification.** Reported: P(True) did not help Qwen3.5-2B rank candidates
   (ΔAUROC +0.008), though it helped Qwen3.5-4B (arXiv 2609.33142). Low priority.
10. **Not recommended:**
    - self-consistency sampling (N× cost, little routing gain);
    - one call per field (+9 s per field);
    - few-shot examples chosen per document type (breaks the cached instruction prefix);
    - thinking (measured worse).

## Models (checked: licences, repos, file hashes)

Qwen has released no small model since Qwen3.5. Qwen3.6 and 3.8 start at 27B (reported). The
models worth benchmarking under Intern's limits (CPU, ~3 GB RAM, commercial licence) are below.
All three files are downloaded and verified against the Hub's SHA-256.

| Model | Licence | Why | Risk |
|---|---|---|---|
| **MiniCPM5-2B** (`openbmb/MiniCPM5-2B-GGUF` @ `2079a22`, Q4_K_M, 1.56 GB) | Apache-2.0 | Vendor IFEval 86.7 and IFBench 66.3, against Qwen3.5-2B's 77.5 and 46.0; plain Llama architecture | Thinking-first post-training; non-thinking quality unproven here; ~1.0–1.25x Qwen's time |
| **Granite-4.0-1B** (`ibm-granite/granite-4.0-1b-GGUF` @ `b27c2fe`, 1.02 GB) | Apache-2.0 | Non-reasoning instruct model, IFEval 77.4; ~0.8x Qwen's time | Weaker knowledge; nothing compares it with Qwen3.5-2B |
| **NuExtract-2.0-2B** (`numind/NuExtract-2.0-2B-GGUF` @ `d038fea`, 0.99 GB) | MIT | Trained for verbatim extraction | 2024 base (Qwen2-VL); no published scores |
| Intern-Decision-2B (`internlm/Intern-Decision-2B`) | Apache-2.0 | A Qwen3.5-2B fine-tune for calibrated yes/no and choice answers; could fill d1's review-gate role without d1's licence | Trained on agent tasks; its official readout is Transformers-only |

Rejected:
- 3–4B dense models (Granite 4.x 3B, Gemma 4 E4B, Ministral 3B, SmolLM3, Phi-4-mini): too
  slow.
- Gemma 4 E2B: ~4.5 GB RSS.
- Small-active MoEs: 3.8–4.8 GB files.
- Hunyuan: its licence excludes the EU, the UK and South Korea.
- Llama 3.2: branding and acceptable-use terms.
- NuExtract-2.0-4B: non-commercial.
- LFM: revenue cap.

## Fine-tuning (reported; needs a decision and a budget)

This is the largest remaining model-side lever, and the one that needs the most care.

- **Evidence (reported).** A LoRA fine-tune of Qwen3.5-2B on 6,125 examples took an extraction
  task from F1 0.49–0.55 to 0.952 (arXiv 2606.08051). NuExtract3, a Qwen3.5-4B fine-tune,
  scored 0.651 against 0.417 for base 4B and 0.479 for 9B. Intern already has a grammar, a
  validator and retrieval, so its headroom is smaller. Estimate: 52 → 58–66 names; no gain
  if the training data does not resemble real documents.
- **Data.** Do not reuse InternBench's generator: new seeds would produce near-duplicates of
  the benchmark. Permissive sources:
  - CUAD (CC BY 4.0);
  - SEC EDGAR exhibits and letters (sec.gov content may be redistributed);
  - US federal documents (public domain).

  FUNSD, DocILE, Kleister and RVL-CDIP are non-commercial or unlicensed.
- **Teacher.** Anthropic, OpenAI and Google terms restrict training competing or distributed
  models on their outputs; a model Intern ships is distributed. Use self-hosted Apache-2.0
  teachers instead (a large Qwen3.5 or gpt-oss-120b). Keep a label only when two labellers
  and Intern's validator agree.
- **Mechanics.**
  - Train a bf16 LoRA with Unsloth (Qwen3.5 support confirmed by the vendor; about 5 GB
    VRAM).
  - Merge, convert to GGUF with the converter of the llama.cpp tag the app ships, and
    quantize to Q4_K_M.
  - Training prompts must be Intern's own byte for byte. Capture them with
    `intern-bench --wire-output`.
- **Cost.**
  - GPUs: a few hundred dollars.
  - Labelling: 40–60 hours of human adjudication.
  - Timeline: a 1-week evaluation set first, a 2–3-week pilot, then 4–6 weeks for the full
    run.
- **Gate.** A held-out set of about 300 real documents, labelled before any training. About
  140 documents can detect +10 points, and about 400 can detect +5. InternBench alone is
  too small and synthetic: 11 vs 12 discordant names give McNemar p = 1.0.

## Still to measure

- The candidate models above, live on the same runtime as a Qwen control.
- Log-probability routing (item 3).
- The prompt scaffolding cleanup (item 6).
- A held-out set of real documents. This is the most important missing piece. Every rule
  so far was written from failures on the 77 synthetic documents it is scored on.
