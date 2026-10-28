# LFM3-Omni-2B as Intern's model, and d1-2B as a review gate (Linux CPU)

Status: complete (2026-10-28). Question asked: can LiquidAI's LFM3-Omni-2B replace
Qwen3.5-2B as the model that names documents, and can d1-2B make the pipeline better?

## Answer

1. **Do not replace Qwen3.5-2B with LFM3-Omni-2B, or with LFM3-1.7B (Omni's own text
   backbone).** On the same runtime, flags and code, both name fewer documents correctly
   (Omni 40/77 at best, LFM3-1.7B 45/77, Qwen 52/77), both are slower (their tokenizer
   needs ~46% more tokens for the same text), and Omni would have filed **3 documents as
   ready under a trap date**, which the production model never does here. Omni's vision
   and audio are separate projector files Intern would not load; for Intern it is a 1.7B
   text model and the weaker of the two.
2. **d1-2B is the interesting one, as a check, not a namer.** Asked one yes/no question
   about each name Qwen marked ready - "Is <party> the main organization or person this
   document belongs to or comes from?" - it held **8 of the 14 wrong names for 3 of the 46
   right ones** (threshold chosen leave-one-out, so not fitted to the document it scores;
   8 for 5 with the smaller Q4_K_M file at the app's thread count, ~15 s per ready name).
   Those 8 are the substantive errors: a board chair, an aircraft mechanic, a museum's loan
   contact, an amendment's signatories named instead of the organization. That roughly
   halves the wrong names Intern would file without a check. See the costs and caveats
   before reading it as a decision.
3. **Licence, for both models: LFM Open License v1.0.** It is Apache-2.0 in form, but §5
   licenses commercial use only to a legal entity under **US$10M annual revenue**
   ("Threshold"); "Any Commercial Use ... by a Legal Entity that exceeds the Threshold is
   not licensed". A law firm above that, using Intern on client work, would be using the
   model unlicensed, whatever Intern's own size, unless Sonomos or the firm holds a
   commercial licence from Liquid AI. There is no non-compete or share-alike (unlike the
   Indra OCR weights); the revenue cap alone is the obstacle, and it falls on Intern's
   intended customers.

## What was run

Host: 4 vCPU Intel Xeon @ 2.80 GHz (AVX-512), 15 GB RAM, Ubuntu 24.04, no GPU; one
measurement at a time.

**Runtime:** llama.cpp **b11946** (commit `8ca4060b7e30e76baaa5670ee1ec1b760b7f8552`, the
first release carrying LFM3 text #31252, LFM3-Omni #31257 and d1 #31275; the model cards
still say those PRs are pending). Built from source with `scripts/build-llama-cpu-linux.sh`
in portable mode (one CPU backend per micro-architecture, chosen at start-up, as the
release archives do). Every model, Qwen included, ran on this one build, so the
comparison is like for like. (Production ships PrismML's fork, which has no LFM3 support.)

**Models** (Hugging Face, verified against the Hub's LFS SHA-256):

| File | Repo @ revision | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| LFM3-Omni-2B-Q4_K_M.gguf | LiquidAI/LFM3-Omni-2B-GGUF @ 5bcc26f7968f71d26e066027a70978529dbb605d | 1,099,630,080 | 9f8d30e9d4fd7a9349d586e077988229b3ca0a0e4ad730897563d1eccc9893e2 |
| LFM3-1.7B-Q4_K_M.gguf | LiquidAI/LFM3-1.7B-GGUF @ b3f2ae5d73cd3c91bf40e8ba752c648aa05dffdf | 1,099,627,808 | 3793e1e6e2266919d6e015287221b41100175a98eddaba92e111aed3b05c93c4 |
| d1-2B-Q8_0.gguf | LiquidAI/d1-2B-GGUF @ 63be75e6586882defeb706ed4bda812d90256a6d | 1,846,240,576 | 073313d92b53b0ac8193db6e906cb3c78a98183d244b15b7b8f5426115c6738c |
| d1-2B-Q4_K_M.gguf | same | 1,099,629,888 | adca5789ad30ae6027e8e936749bb9f19673097492ab26fa1c5b69643ca50b4b |
| Qwen3.5-2B-Q4_K_M.gguf (production) | as pinned in the manifest | 1,280,835,840 | (manifest) |

Omni's GGUF is the language model alone: the same 1.1 GB as LFM3-1.7B, with vision and
audio in separate `mmproj` files that were not loaded (`--no-mmproj`), as Intern sends text.

**Accuracy:** live InternBench, 77 documents, `bench/gold.json`, current `main` (a759d11,
with #36's validation rules), the app's server flags plus `--cache-ram 0 -ub 256`, 4
threads, chunks of 8 documents on a fresh server (`scripts/lfm3-eval/run.sh`). The LFM3
models also ran with `--reasoning off` (they are non-reasoning; their template opens every
reply with an empty `<think></think>`).

**d1:** `scripts/lfm3-eval/d1_verifier.py` sends each document's recorded text (the first
6,000 and last 2,000 characters when longer, so the letterhead and the signatures stay in)
and questions about the name Qwen proposed in the live run to llama-server's
`/v1/systemone`. llama.cpp answers yes/no, choice and score questions; d1's extraction
(text, number, date) runs only in Python Transformers, which Intern cannot ship, so none
was asked. Each question is scored as a review gate over the names marked ready.

## Results: a replacement model

| Live InternBench, 77 docs | Names right | Dates right | Ready | Wrong but ready | Trap date proposed / filed | Forbidden party ready | Median per doc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **Qwen3.5-2B (production)** | **52** | **76** | 61 | 15 | 0 / 0 | 7 | **17.5 s** |
| LFM3-Omni-2B, app prompt | 22 | 33 | 0 | 0 | 0 / 0 | 0 | 20.8 s |
| LFM3-Omni-2B, few-shot | 40 | 63 | 50 | 19 | 6 / **3** | 4 | 21.2 s |
| LFM3-1.7B, app prompt | 40 | 69 | 1 | 1 | 6 / 0 | 0 | 21.2 s |
| LFM3-1.7B, few-shot | 45 | 63 | 40 | 7 | 2 / 0 | 1 | 22.7 s |

* **With Intern's own prompt both LFM3 models copied the instructions into the answer.**
  Omni's `document_type` was literally `"what the document is"` (the prompt's line
  "type: what the document is", `crates/intern-engine/src/prompt.rs`); LFM3-1.7B's was
  `"Document"`. Validation rebuilt a type from the title and held every document
  (`TYPE_UNSUPPORTED` on 77/77 and 75/77), so those rows measure prompt-following, not
  naming. Bonsai 2B failed the same way, which is what Intern's few-shot profile
  (`--prompt-profile bonsai_2`: the production instructions plus one neutral worked
  exchange) was built for; the few-shot rows are the fair comparison.
* **Few-shot, neither reaches Qwen.** Omni: 12 fewer names, 4 more wrong-but-ready, and 3
  trap dates filed as ready (`sow-under-msa-5p`, `scan-rotated-page-in-pdf`,
  `scan-remittance-advice-120dpi`); the release gate on trap dates is absolute. LFM3-1.7B
  is the more careful one - 7 wrong-but-ready against 15, no trap date filed - but it buys
  that by holding 37 documents for review against Qwen's 16, and it names 7 fewer
  correctly. Qwen with the d1 gate below gets to the same ~7 wrong-but-ready while keeping
  52 names right and holding 27.
* **Speed:** LFM3 decodes faster than Qwen (15 vs 12 tok/s) and prefills at the same rate
  (~100 tok/s at 4 threads), but its tokenizer turns the same documents into ~46% more
  tokens (median prompt 1,268-1,421 vs 870), so each document takes 3-5 s longer. A prompt
  change does not fix that.
* **Memory:** server peak ~2.0 GB (LFM3) vs 2.1 GB (Qwen) with `--cache-ram 0`.

Not tested: using Omni's vision to read scans in place of PP-OCR. Intern's pipeline needs
the line IDs of extracted text to check every fact the model cites, which a model reading
pixels cannot give it; that would be a different pipeline, not a model swap.

## Results: d1-2B as a review gate

Over the 60 names the live Qwen run marked ready that name a party (14 of them wrong; the
15th wrong-but-ready name, "2026-03-04 Invoice.pdf", names no party, so there is nothing
to ask). Seven questions per document, Q8_0, 4 threads. "Held" = sent to review.

| Signal (higher = more doubtful) | Leave-one-out, catching one wrong name worth holding 1 right one | worth 2 |
| --- | --- | --- |
| **1 - P(yes) "Is <party> the main organization or person this document belongs to or comes from?"** | **8/14 wrong held, 3/46 right held** | 8/14, 6/46 |
| 1 - P(yes) "Is <filename> an accurate filename ...?" | 7/14, 7/46 | 9/14, 10/46 |
| P(yes) "Is <party> only an individual who signed ... for an organization?" | 4/14, 2/46 | 4/14, 2/46 |
| relation word vs d1's choice of the party's relation | 2/14, 4/46 | 5/14, 4/46 |

Ranked by the main-party question, 10 of the 12 most doubtful ready names are wrong ones:
the board chair named for the cooperative (`board-minutes`), the mechanic for the air
charter company (`aircraft-maintenance-log`), the loan contact for the museum
(`field-condition-report`), the signatories for the licensee (`mixed-signature-region`),
the purchase order's wrong side (`purchase-order`, `scan-upside-down-po`), and four more.
The three it misses name the right party with "to" where the gold says "for"
(`quarterly-business-review`, `property-policy-declarations-mid-60p`,
`watershed-monitoring-report-100p`): d1 rightly says that is the main party.

For comparison, the token-confidence gate measured on Qwen in
`bench/reports/bonsai-linux-cpu-2026-10-27` (fixed threshold 0.5) held 8 wrong names for
7 right ones.

### As it could ship: one question, Q4_K_M, the app's 2 threads

`scripts/lfm3-eval/run-d1-gate.sh`: only the main-party question, only for names marked
ready, the 1.1 GB Q4_K_M file, 2 threads (Intern's count on this 4-core host), `-b/-ub
4096`. Results in `d1/d1-2B-Q4_K_M-party_main-t2.md`.

| d1 file, threads | Wrong names held (LOO) | Right names held (LOO) | Wrong among the 12 most doubtful | Time per ready name (median / p90 / max) | Server peak |
| --- | ---: | ---: | ---: | --- | ---: |
| Q8_0, 4 threads, 7 questions | 8/14 | 3/46 | 10 | 94 s / - / 335 s (all 7 questions) | 2.4 GB |
| **Q4_K_M, 2 threads, 1 question** | **8/14** | **5/46** | 7 | **14.8 s / 45 s / 59 s** | 2.3 GB |

Q4_K_M keeps the signal (its probabilities differ from Q8_0's by a median 0.016, at most
0.18) at a slightly worse trade: two more right names held. Shipped this way, Intern's
InternBench run would go from 15 wrong names filed to about 7, with 13 more documents held
for review (16 -> 29 of 77), and each ready document would wait a further ~15 s - on top
of Qwen's ~35-40 s per document at 2 threads on this host (`bench/reports/bonsai-linux-cpu-2026-10-27`). The 2.3 GB peak is mostly
compute buffers for the 4096-token batch the decision path needs, not weights.

### What it costs

* **llama.cpp reads the whole state once per question.** Seven questions over a median
  2,634-character state took a median 94 s per document (max 335 s) at 4 threads; the
  prompt cache does not change that (same answers to 4 decimal places, same time). Only
  the one question that works should ever be asked.
* Memory: the Q8_0 server peaked at 2.4 GB (16k context, `-ub 8192`), the Q4_K_M one at
  2.3 GB (8k context, `-ub 4096`).
* It is a second model: Intern would load it after Qwen for each ready name, or keep both
  resident (~4.4 GB together) - against the one-model-at-a-time rule the memory guard
  enforces on 14 GB laptops.

## Caveats

* One live run per configuration on 77 synthetic documents; a difference of 2-3 names is
  within run-to-run noise (the committed Qwen recording replays at 54/77, this live run
  52/77). The d1 thresholds are leave-one-out on 60 names, not validated on held-out real
  documents.
* d1's yes/no probabilities are calibrated by Liquid AI on their own benchmarks; the
  threshold here would need re-checking on real legal documents before shipping.
* Everything ran on an AVX-512 Linux host with upstream llama.cpp; Windows laptops on
  AVX2 and the PrismML runtime Intern ships were not measured.

## Recommendation

* Keep Qwen3.5-2B. Do not add LFM3-Omni-2B or LFM3-1.7B as either tier: no accuracy or
  speed gain, a trap-date regression for Omni, and a licence Intern's customers would
  breach.
* d1 as a gate is worth pursuing only if (a) Liquid AI will license it for Intern's
  customers above US$10M revenue, and (b) a check of the same question on real, consented
  legal documents holds up. If both clear, ship it as one question, asked only about names
  about to be marked ready.
* The same idea can be tried without a second model or a licence: ask Qwen itself the
  main-party question as a second, constrained turn (yes/no with log-probabilities). That
  is the cheaper next experiment.

## Reproduce

```sh
export S=/path/to/scratch
cargo build --release --locked -p intern-bench --features benchmark-transport --bins
cargo build --release --locked -p intern-worker --features windows-native
# models: pinned downloads verified against the Hub (see the table above)
S=$S scripts/lfm3-eval/run.sh            # builds b11946; Qwen, Omni, LFM3-1.7B live; d1 (7 questions, Q8_0)
S=$S scripts/lfm3-eval/run-fewshot.sh    # LFM3-1.7B and Omni with --prompt-profile bonsai_2
S=$S scripts/lfm3-eval/run-d1-gate.sh    # d1 as it could ship: one question, Q4_K_M, app threads
python3 -I scripts/lfm3-eval/d1_verifier.py analyze --answers $S/results/lfm3/<run>-answers.jsonl \
  --report $S/results/lfm3/qwen/report.json
```

Per-run reports are in `internbench/`, d1 analyses in `d1/`.
