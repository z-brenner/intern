# Making Intern more accurate: research, October 2026

Status: research summary plus the measured changes of 2026-10-29. The question was
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

Speed was measured the same day, in
[`bench/reports/speed-linux-cpu-2026-10-29`](../bench/reports/speed-linux-cpu-2026-10-29/README.md).
Shorter ids, two documents at once and speculative decoding do not pay on a 4-core CPU. The
upstream llama.cpp runtime reads prompts 36% faster than the packaged one.

## What was tried and measured

| Change | Result | Verdict |
|---|---|---|
| **Organization over staff, record relation "for"** (deterministic, `facts.rs`/`compose.rs`, PR #40) | Qwen production runtime **52 → 59** names, unsafe **17 → 11**; committed recording 54 → 62, 14 → 7; no right name lost on any of 8 recorded runs × 77 documents (5 models), apart from one name the earlier code had right by luck | **Shipped** |
| **Issuers: an order from its buyer, a header's organization before the kind of document, a title's type** (deterministic) | **+3 names and 2 fewer unsafe on every Qwen run** (committed 62 → 65, 7 → 5; production runtime 59 → 62, 11 → 9); no name lost on any of 9 runs | **Shipped** |
| **An insurance document is for its insured** (deterministic, `facts.rs`): "Named insured", "Insured", "Policyholder", "Member", "Patient" and "Subscriber" support the client role, as a field label or a table's column header ("Crew member" and other staff groups never do) | **+1–2 names on all five Qwen runs, 1 fewer unsafe on three** (committed 65 → 66, 5 → 5; production runtime 61 → 63, 8 → 7; upstream 63 → 65, 6 → 5); +1 name on 4 of the 5 other models' runs; no name lost on any of 10 runs. A loss notice on the insurer's form is no longer filed "from" the insurer. | **Shipped** |
| **A court document is filed between the two sides of its case** (deterministic, `facts.rs`/`compose.rs`): a plaintiff, petitioner, appellant or claimant and a defendant, respondent or appellee, each labelled beside its name, in a document with "A v. B"; letters and emails about a case are unchanged. A role word no longer completes a name ("Plaintiff Gilchrist ... LLC") | The court hearing notice right on 9 of 10 runs (8 were wrong and ready); +1 name and 1 fewer unsafe on all five Qwen runs (committed 66 → 67, 5 → 4); no other document changed | **Shipped** |
| **A record or a notice is for the side its label names** (deterministic, `facts.rs`): a reply's bare "recipient" gives way to a client, tenant or other side-it-is-about role that the document's own label gives the party ("Prepared for ...") | The watershed report right on the 3 runs that had it wrong and ready; committed 67 → 68, 4 → 3; no other document changed on 10 runs | **Shipped** |
| **A record its staff made comes from the organization they act for** (deterministic, `facts.rs`): when the reply named staff (an examiner, an inspector) and another organization, the first page's organization is added as the record's issuer | The loan condition report right on 6 of 10 runs (all were wrong and ready), and one maintenance record; committed 68 → 69, 3 → 2; no other document changed | **Shipped** |
| **Review gate on token probability** (least probable date or party token below 0.5 goes to review; `docs/model-candidates.md`) | Unsafe 9 → 3 and 8 → 4 on two production-runtime runs, for 9 and 12 right names sent to review; about +1–2% time | **Shipped** |
| Short ordinal line ids (`[3]` for `[p1.b4.r2]`) | Prompt −5%, reply −24% tokens, but −1% time per document; names 58 → 50, trap dates 0 → 5. The model cited line 1 for most facts. | Rejected |
| Plainer prompt scaffold ("1 page, every line:" and `<<<`/`>>>` for "The whole 1-page document follows" and BEGIN/END EVIDENCE) | Names 58 → 58 with 19 of 77 names changed in both directions; unsafe 10 → 12. The echo it targeted is now caught by the issuer rules instead. | Rejected |
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
3. **Better routing from token log-probabilities.** Shipped as a review gate (measured above). On
   the recorded replies, the minimum over date and party tokens beat the alternatives tried:
   party names alone, each name's first token, the type, the role words. Reported: log-prob
   confidence ranks errors but does not verify them. First-token aggregation ranked best on
   12–27B models (ConfBench, arXiv 2608.01792), and a single misattributing sentence flipped
   high-confidence extractions in 96–100% of counterfactuals (arXiv 2610.20417).
4. **Party names constrained by the grammar to candidate strings.** This adds a typed
   candidate block, organizations first. Reported: a candidate grammar took LLaMA-7B entity
   disambiguation from 37.4 to 77.2 (arXiv 2305.13971). It is less likely to help here,
   since Intern's errors are choices between real names, not invented ones. Cost +50–150
   prompt tokens.
5. **Organization-first output key and per-class role lists in the grammar.** The direction of
   the effect is inconsistent in the literature; it needs a live run.
6. **Remove words the model echoes from the prompt scaffolding.** Measured and rejected (above):
   any change to the prompt moved about a quarter of the names for no net gain.
7. **Type chosen from the document's title candidates.** Rules out a bare "Letter",
   "Agreement" or "Report". Targets about 5–7 held misses.
8. **A small NER model to tell people from organizations.** GLiNER family, Apache-2.0, ONNX.
   Only if 1 and 4 misclassify.
9. **Same-model yes/no verification.** Reported: P(True) did not help Qwen3.5-2B rank candidates
   (ΔAUROC +0.008), though it helped Qwen3.5-4B (arXiv 2609.33142). A two-step "validate the
   first answer" pass helped 8–70B models but hurt 3–4B ones (arXiv 2607.26780). Low priority.
10. **Better wording of each field's question.** Reported: optimizing the extraction questions
    alone moved clinical extraction F1 by about 18.6 points across five models, more than a
    larger model did (arXiv 2608.31058). It is worth doing only against a held-out set:
    tuning wording on these 77 documents would fit them, not documents in general.
11. **Not recommended:**
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

Measured live (2026-10-29): 77 documents, upstream llama.cpp b11946, the same machine, scored
with the rules above. MiniCPM5 and Granite were also run with the fast tier's few-shot prompt
profile (`bonsai_2`), which shows a small model the shape of a reply.

| Model, prompt | Names right | Wrong names ready | Time per document (median) |
|---|---:|---:|---:|
| **Qwen3.5-2B, production prompt** | **65** | **5** | 18.8 s |
| MiniCPM5-2B, production prompt | 48 | 2 (4 ready at all) | 24.4 s |
| MiniCPM5-2B, few-shot | 50 | 16 | 22.6 s |
| Granite-4.0-1B, production prompt | 0 (null replies) | 0 | 12.8 s |
| Granite-4.0-1B, few-shot | 47 | 10 | 18.5 s |

Neither replaces Qwen: both name fewer documents and mark more wrong names ready, and neither
is faster once its replies are usable. MiniCPM5's zero-shot replies echoed the prompt's words
as the type and sent nearly everything to review. NuExtract-2.0-2B was not run: it takes its
own `template` request argument instead of Intern's grammar.

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

## What the failures are (reported, and consistent with what is measured here)

A preregistered audit of LLM entity extraction (arXiv 2610.20417) found hallucination
behind only 1.2% of false positives. 65.6% were real strings from the wrong context, and
19.9% fell outside the gold set's scope. That matches Intern's errors: the right names are
in the context, and the model picks a person who works for the party, the wrong side of an
order, or a word from the prompt. Its conclusion matches the measurements above. Repairs work
when matched to the failure mode (the deterministic role rules), and confidence can set
review priorities but never settle truth (the gate sends a reading to a person; it does not
change the name).

## Still to measure

- A held-out set of real documents. This is the most important missing piece. Every rule
  so far was written from failures on the 77 synthetic documents it is scored on.
