# LFM3-1.7B retest: thinking on, other quantizations (Linux CPU)

Status: complete (2026-10-28). The question: "Can you test the LFM models again? They're
supposed to be better than Qwen." This follows `bench/reports/lfm3-d1-linux-cpu-2026-10-28/`,
which ran LFM3-1.7B with thinking **off**, greedy decoding, and the generic Q4_K_M file. That is
not how LiquidAI runs the model. Its card says "Reasoning is enabled by default. You can set
`enable_thinking=False` to disable thinking traces, but this behavior can be inconsistent",
and LiquidAI also publishes its own quantization-aware 4-bit file. So this retest gives
LFM3-1.7B both.

Correction to the first report: it called both LFM3 models non-reasoning. That holds for
LFM3-Omni-2B, whose card says "`enable_thinking=True` is not supported". It does not hold for
LFM3-1.7B, which is a hybrid reasoning model.

## Answer

1. **On Intern's task, LFM3-1.7B at its best matches Qwen3.5-2B; it does not beat it.**
   The best configuration is LiquidAI's **QAD-Q4_0** file, thinking off, with Intern's
   few-shot profile. It names **51/77** documents correctly against Qwen's **52**:
   - 40 documents both get right, 11 only LFM3, 12 only Qwen (exact McNemar p = 1.0);
   - it marks fewer wrong names ready (11 vs 15) and no forbidden party (0 vs 7);
   - but most of that is a stricter gate, not better judgment. It holds 24 documents for
     review against Qwen's 16, and files fewer right names unattended (42 vs 46);
   - it gets fewer dates right (70 vs 76), runs ~3 s slower per document (20.6 vs 17.5 s
     median), and its file is 230 MB smaller.

   It is a sidegrade with a different error profile, not an upgrade. The first report's
   **Qwen + d1 gate** files about as many right names (43) with fewer wrong ones (~7), so it
   dominates the QAD row.
2. **Thinking does not help either model on this task.** A thinking budget is the only way to
   make LFM3-1.7B answer with thinking on at all. Within it:
   - LFM3-1.7B with a 1,024-token budget names 50/77, marks **19 wrong names ready**, and
     **files 3 trap dates** (Intern's release gate on trap dates is absolute);
   - it takes **87 s per document**, 5x Qwen;
   - Qwen3.5-2B given the same budget gets *worse*: 47/77 (from 52), 21 wrong but ready (from 15),
     3 trap dates filed (from 0), 93 s per document;
   - both models hit the budget on every one of the 77 documents, so neither ever finished
     reasoning. On a 4-core CPU, a budget that lets them finish is out of reach (see below).
3. **Quantization matters for LFM3, and the vendor's QAD file is the one to use.** Same
   prompt, thinking off: generic Q4_K_M 45/77, Q8_0 49/77, QAD-Q4_0 51/77. QAD-Q4_0 is also
   the fastest of the three (20.6 s median against 22.7 s and 32.7 s) and the smallest. The
   ordering is plausible, but each step is within noise (see "How much to read into it").
4. **New failure mode: LFM3 copies the worked example's date.** Intern's few-shot profile
   (`crates/intern-engine/src/profile.rs`) shows one worked example dated 2026-01-16. Every
   LFM3 few-shot run proposed that date for unrelated documents:

   | Run | Documents |
   |---|---:|
   | LFM3-1.7B Q4_K_M | 10 |
   | LFM3-1.7B Q8_0 | 6 |
   | LFM3-Omni-2B | 6 |
   | LFM3-1.7B QAD-Q4_0 | 3, plus a "2016-01-16" |

   Intern's validation caught every one (`DATE_UNSUPPORTED`, held for review), so none
   was filed. But each one costs a correct name. Bonsai 2B (production's quick tier) and
   Bonsai 4B, given the same worked example in their requests, never did it (0/77 each,
   `bench/reports/bonsai-linux-cpu-2026-10-27/`). A different example
   might win LFM3 two or three names, but tuning it on this same corpus would overfit, so
   that is speculative.
5. **Nothing here changes the obstacles to switching.**
   - **Licence:** the LFM Open License v1.0 licenses commercial use only below US$10M
     annual revenue, which falls on Intern's intended customers.
   - **Runtime:** production ships PrismML's llama.cpp fork, which has no LFM3 support, so
     switching means a second runtime.
   - **Evidence:** a tie on 77 documents does not pay for either. **Keep Qwen3.5-2B.**

## Where "better than Qwen" comes from

The LFM3-1.7B card compares it with Qwen3.5-**4B** and **9B**, not the 2B Intern ships. Its
average across the card's benchmarks is 49.4, against 39.3 for Qwen3.5-4B and 47.2 for
Qwen3.5-9B. It leads clearly on instruction following (IFBench 73.0 vs 38.2 for Qwen3.5-4B,
IFStruct 93.7 vs 87.1) and on long context. It trails on knowledge (GPQA Diamond 63.4 vs
75.8).

Those numbers come from a setup far from Intern's: "served with vLLM in BF16 at their full
context length ... temperature 0.6 and top_p 0.95, with no output limit other than the
context length". That means unbounded thinking, unquantized weights and a GPU server. None of
the card's benchmarks is short-document extraction under a grammar on a 4-core CPU.

So the claim is credible for what it measures. It does not transfer here. On this corpus the
1.7B model ties Qwen's 2B; it does not leap past it the way the card's average suggests.

## What was run

Same host, runtime and code path as the first report:
- host: 4 vCPU Xeon @ 2.80 GHz (AVX-512), 15 GB RAM, no GPU;
- runtime: llama.cpp b11946 (`8ca4060b`), portable build;
- server flags: the app's, plus `--cache-ram 0 -ub 256`, 4 threads, chunks of 8 documents
  on a fresh server;
- corpus: live InternBench, 77 documents, `bench/gold.json`, at `main` 0414b3e plus this
  report's lab-transport change (the engine is otherwise unchanged).

`scripts/lfm3-eval/run-retest.sh` runs every new row. The Qwen3.5-2B baseline and the
thinking-off Q4_K_M rows are the first report's runs, made on the same build.

The model files come from LiquidAI/LFM3-1.7B-GGUF at revision
`b3f2ae5d73cd3c91bf40e8ba752c648aa05dffdf`. Each was verified against the Hub's LFS SHA-256:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| LFM3-1.7B-QAD-Q4_0.gguf | 1,049,615,680 | 43a7d09a0e9cba91b63eb7052488b9564569bddfd65754d2e426465e572a7452 |
| LFM3-1.7B-Q8_0.gguf | 1,846,238,496 | 8f6b24182258205e9e01e21f1f24a1f208c1e8115578825d202ce39976af20b6 |
| LFM3-1.7B-Q4_K_M.gguf | 1,099,627,808 | 3793e1e6e2266919d6e015287221b41100175a98eddaba92e111aed3b05c93c4 |

### Making thinking work at all

Three problems had to be solved before LFM3 answered a single document with thinking on.

1. **Grammar.** Intern sends a GBNF grammar with every request, and llama-server applies it
   from the first generated token. With thinking on, the chat template opens the reply
   inside `<think>` (Qwen3.5's does too), so the grammar forced the JSON *into* the
   thinking block and the answer came back empty.

   The lab transport (feature `benchmark-transport`) gained `grammar_after_thinking`. It
   wraps Intern's grammar so that free text without `</think>` comes first, then the tag,
   then exactly the original reply. The free text is matched state by state, so the tag
   llama-server forces at the budget is accepted after any text, a stray `<` included.

   This was checked with llama.cpp's own grammar validator. A first draft with
   `"<" [^/]`-style alternatives rejected exactly those cases, and was fixed before any
   recorded run.
2. **Unbounded thinking never ends.** Two documents were probed with no thinking budget.
   - With greedy decoding, both ran past 4,096 tokens (322 s).
   - With the card's own "General" sampling (temperature 0.1, top_k 50, repetition penalty
     1.1), both ran past 8,192 tokens (700-780 s).

   Neither answered. The traces re-read Intern's instructions and argue with them: "This
   seems like three columns: date (string), role (maybe 'issued' etc.), id ...".
3. **So thinking is bounded.** llama-server's `reasoning_budget_tokens` closes the block
   after N tokens; with 1,024, both probes answered in ~80 s. That is the configuration
   measured: `scripts/lfm3-eval/think1024-profile.json`, Intern's greedy decoding,
   `max_tokens` 2,048. Qwen3.5-2B ran with the identical profile as the control.

On every document, for both models, the reasoning hit the budget. Completion tokens ran
1,088-1,143; every reply ended `stop`, and none was cut off by `max_tokens`. These rows
therefore measure each model answering after a forced end to its reasoning. Not tested, and
why:
- **A larger budget.** It costs ~70 s per extra 1,024 tokens at ~15 tok/s, so a budget of
  2,048 would put LFM3 at ~160 s per document, 9x Qwen. The unbounded probes say even 8,192
  may not be enough.
- **Thinking with sampling.** Qwen's and LiquidAI's sampled settings make runs
  non-reproducible. Intern decodes greedily so that the same document always gets the same
  name.

## Results

Live InternBench, 77 documents. "Wrong but ready" = a wrong filename the app would file
without review; "Trap date filed" = a ready name built on a date the gold marks as a
trap.

| Run | Names right | Dates right | Types right | Ready | Right & ready | Wrong but ready | Trap date proposed / filed | Forbidden party ready | Median per doc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **Qwen3.5-2B Q4_K_M, thinking off (production)** | **52** | **76** | 70 | 61 | **46** | 15 | 0 / 0 | 7 | **17.5 s** |
| Qwen3.5-2B Q4_K_M, thinking 1,024 | 47 | 72 | 70 | 63 | 42 | 21 | 4 / **3** | 3 | 92.8 s |
| LFM3-1.7B Q4_K_M, thinking 1,024 | 50 | 73 | **73** | 58 | 39 | 19 | 3 / **3** | 0 | 86.9 s |
| **LFM3-1.7B QAD-Q4_0, thinking off, few-shot** | **51** | 70 | 66 | 53 | 42 | **11** | 1 / 0 | **0** | 20.6 s |
| LFM3-1.7B Q8_0, thinking off, few-shot | 49 | 67 | 68 | 46 | 35 | 11 | 3 / **2** | 0 | 32.7 s |
| LFM3-1.7B Q4_K_M, thinking off, few-shot (first report) | 45 | 63 | 66 | 40 | 33 | 7 | 2 / 0 | 1 | 22.7 s |
| LFM3-1.7B Q4_K_M, thinking off, app prompt (first report) | 40 | 69 | 55 | 1 | 0 | 1 | 6 / 0 | 0 | 21.2 s |

* **Where LFM3 and Qwen differ.** Every LFM3 variant beats Qwen on the same handful of
  documents:
  - invoice-date-in-table, explanation-of-benefits, field-condition-report;
  - scan-cancellation-notice-150dpi, mixed-signature-region, and usually board-resolution
    and watershed-monitoring-report-100p.

  It loses on another consistent set:
  - notice-rent-increase, vendor-registration-form, credit-agreement-50p,
    product-launch-plan;
  - scan-low-res-receipt, scan-fax-two-frames, scan-patient-intake-form,
    benefits-change-checkbox-form.

  The losses lean one way: LFM3 names the **issuer** where the gold wants the person the
  document is about. Examples are the landlord's agent instead of the tenant, the insurer
  instead of the insured, the clinic instead of the patient, and the employer instead of
  the employee.
* **Types.** LFM3 with thinking gets the most types right (73), but it reads "use the
  document's own words for it, whole" literally, giving "Straight Bill of Lading - Short
  Form" and "Quarterly Business Review - Q3 2026". The gold's shorter types count those as
  wrong names.
* **The three trap dates LFM3 with thinking filed:**
  - a vendor form's date of incorporation (2014-11-03);
  - a patient intake form's insurance effective date;
  - a policy's countersignature date where the gold wants the policy period's start.

  Q8_0 filed two more: a statement period's start, and a credit agreement's first covenant
  test date. Qwen with thinking filed three, all traps it never takes without thinking:
  - the master agreement's date on a SOW issued under it;
  - an annual meeting date on an annual report;
  - a qualifying event date on a benefits change form.
* **Speed and memory.** The QAD-Q4_0 file decodes at 15.9 tok/s and prefills at 109 tok/s,
  against 15.0 and 100 for Q4_K_M, at the same ~2.0 GB server peak. Q8_0 is the slowest of
  the three on this CPU: 11.1 and 63 tok/s, 32.7 s per document. The ~3 s per document
  LFM3 loses to Qwen is not its tokenizer, as this report first said ("~46% more tokens"):
  that compared LFM3's whole prompt (median 1,421 tokens few-shot) with only the uncached
  part of Qwen's (870 of 1,347). llama-server serves Qwen's 477-token instruction block from
  its prefix cache and served none of LFM3's (`cached_tokens` 0 on every document), so LFM3
  prefills its instructions again each time. Corrected 2026-10-29.

### How much to read into it

77 documents, one run per configuration, greedy decoding. Paired on the same documents,
names right against Qwen, exact McNemar test:

| Run | Only LFM3 right | Only Qwen right | p |
|---|---:|---:|---:|
| QAD-Q4_0 | 11 | 12 | 1.0 |
| Q8_0 | 10 | 13 | 0.68 |
| LFM3 with thinking | 10 | 12 | 0.83 |
| Q4_K_M few-shot | 8 | 15 | 0.21 |
| *Qwen with thinking (against Qwen without)* | *8* | *13* | *0.38* |

None of these is distinguishable from Qwen. The counts of wrong-but-ready names (7-19) and
trap dates (0-3) are small too, so a difference of two or three either way is not a finding.
What is robust:
- every LFM3 configuration fails to clear Qwen;
- thinking at a usable budget costs ~5x the time and buys nothing for either model;
- the few-shot date copying, in every LFM3 few-shot run and neither Bonsai run.

## Per-run reports

Each run's full InternBench report is in `internbench/`:
- `lfm3-1.7b-q4km-think1024.md`
- `lfm3-1.7b-qad-q4_0-fewshot.md`
- `lfm3-1.7b-q8_0-fewshot.md`
- `qwen-think1024.md`

The baseline rows are in `../lfm3-d1-linux-cpu-2026-10-28/internbench/`.
`runtime-version.txt` is the same runtime build as the first report.
