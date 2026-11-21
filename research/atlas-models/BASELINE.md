# ATLAS model research: baseline

Status: 2026-11-21, written before anything in the product was changed, on branch
`claude/atlas-model-research-ogj49l` at engine v0.1.0-alpha.21. Nothing here changes how the
app reads a document.

Every figure below is marked as one of:

- **measured here (repo)**: a measurement already committed in this repository, with the
  report it comes from. Those runs were made on Linux cloud runners (4 threads on 2 cores, AMD
  EPYC 7763, a Zen 5 runner, or a 4-core Xeon), never on a Windows laptop.
- **derived here**: arithmetic done for this research on committed measurements, with the
  script that does it. No model was run for it.
- **read from the code**: what the source says, with the file.

No figure in this file comes from a run made for ATLAS. The first such runs are described in
[`README.md`](README.md) once they exist.

## 1. What the model is asked to do

ATLAS (the `intern` repository) reads a document and proposes a filename
`YYYY-MM-DD Type for/from Party.ext`, plus a short description, and decides whether the name
can be filed without a person looking. The model is one part of a longer pipeline (read from
the code):

| Stage | Where | What it does |
|---|---|---|
| Extraction | `crates/intern-worker` (PDFium, Tesseract, PP-OCR, Office readers) | Text and layout per page, OCR where a page has no text layer |
| Evidence index | `crates/intern-engine/src/evidence.rs` | Every block, line and field gets a short handle such as `p1.b4.f2` |
| Retrieval | `crates/intern-engine/src/retrieve.rs` | Picks the lines the model sees: whole, small, normal or dense tier, fitted to the context |
| **One model exchange** | `crates/intern-engine/src/prompt.rs`, `client.rs` | 477-token cached instructions + about 820 tokens of evidence lines; the reply is forced by a GBNF grammar |
| Validation | `validate.rs` | Every value must be found in the line it cites; invented values are dropped |
| Rules and composition | `facts.rs`, `compose.rs`, `naming.rs` | Rules confirm or replace the type, date role and party; the name is composed |
| Review gate | `tiers.rs` | A name is held for a person if the least probable token of its date or a party's name is under 0.7 |

The model's whole job is one structured reply (read from `prompt.rs`):

```json
{"type":["Notice of Rent Increase","p1.b3"],
 "date":["2026-05-12","notice","p1.b4.f1"],
 "parties":[["Imogen Castellanos","tenant","p1.b4.f2"],["Tamarack Property Management Co.","sender","p1.b8"]],
 "subject":["Notice of rent increase for Apartment 4C","p1.b3"]}
```

Date roles and party roles are closed lists; at most three parties; each evidence id must be
one of the handles the prompt shows, which the grammar enforces token by token. Temperature 0,
thinking off, token log-probabilities on (read from `client.rs`).

This matters for everything after: **the target is not a general document model. It is a model
that fills one 4-field form from about 800 tokens of numbered lines.** The reply averages
about 91 tokens. That narrowness is exactly why fine-tuning a small model is plausible here,
and also why general benchmark scores have predicted nothing in this repository before
(`docs/research-2026-11.md`, section 1).

## 2. Runtime the model must fit (read from the code)

- llama.cpp upstream b11946, CPU only, `--ctx-size 8192`, `--parallel 1`, `-ub 256`,
  `--no-repack`, `--cache-ram 0`, all logical processors up to eight, GGUF Q4_K_M.
- Two tiers: accurate Qwen3.5 9B (needs 8,192 MiB available), quick Qwen3.5 4B (6,656 MiB).
  Quick readings are always held for review. The quick-then-accurate cascade is implemented
  (`tiers::escalation`) and switched off.
- Both downloads together stay under 10 GiB (`manifest.rs`; 8.42 GB today).
- Licence rule: commercial use at any company size (this excluded LFM and d1 families).

Any student has to load in b11946 (or a pinned upgrade), honour the grammar, report token
probabilities and run with thinking off. A fine-tuned Qwen3.5 student keeps the same
architecture and chat template, so it inherits all of that.

## 3. Accuracy today (measured here, repo)

### 3.1 The bar the repository already uses

From `docs/research-2026-11.md`, alpha.17 engine, 0.7 gate, InternBench plus real sets 1 to 5:

| | Names right | Wrong names filed | Run-to-run spread |
|---|---:|---:|---:|
| Qwen3.5 9B | 200-202 of 335 | 5-7 of 59-60 filed | 2 names |
| Qwen3.5 4B | 159-165 of 291 | (gate unusable: about one wrong per right one) | 6 names |

The repository's own switch rule (same file): a 4B-class model replaces the 9B only if, in
two runs on InternBench plus real1-6, it names at least as many right as the same-run 9B,
files no more wrong, and takes at most 0.6 of the 9B's time. ATLAS adopts that rule unchanged
as its acceptance test, because it was written before any ATLAS candidate existed.

### 3.2 Per set, all three Qwen3.5 sizes (alpha.13 engine replay)

From `bench/reports/model-qualification-2026-11/readings.json`. Older engine, so the absolute
counts are lower than 3.1; the comparison between sizes is what it is for.

| Set (documents) | 9B right | 4B right | 2B right |
|---|---:|---:|---:|
| InternBench (77, synthetic) | 70 | 64 | 71 |
| real1 (60) | 22 | 18 | 16 |
| real2 (60) | 21 | 16 | 17 |
| real3 (59) | 11 | 12 | 11 |
| real4 (58, unseen when read) | 15 | 13 | 13 |
| CUAD contracts (42) | 19 | n/a | 14 |
| fixtures (21) | 10 | n/a | 13 |

Two things stand out:

1. **InternBench is saturated and does not separate the sizes** (2B 71, 9B 70). Every
   decision has to be made on the real sets.
2. **On real documents every size is weak in absolute terms**: the 9B names 11 to 22 of about
   60 right. The headroom is large, which is the opportunity, and the per-set noise is a few
   names, which is the trap.

### 3.3 Where the errors are (measured here, repo)

`docs/research-2026-11.md`, section 3: of 13 really wrong names on real sets 4 to 6 whose date
and type a rule confirmed, **12 are wrong in the party**: a person where the organisation was
wanted, or the sender where the addressee was. The rule layer already lifts the type from 202
to 224 right of 247. So a student does not need to beat the 9B everywhere; it needs to beat
it on who the document is for and from. That is a narrow, learnable decision and the place
fine-tuning should be aimed.

## 4. Speed and memory today (measured here, repo)

`bench/reports/model-qualification-2026-11/README.md`, same 4-core Xeon, 4 threads, repacked
weights, per document:

| Model | p50 latency | Prefill p50 (about 860 tokens) | Decode | Peak RSS |
|---|---:|---:|---:|---:|
| Qwen3.5 2B Q4_K_M | 14.9 s | 8.7 s | 12.9 tok/s | 2.1 GB |
| Qwen3.5 4B Q4_K_M | 37.3 s | 23.7 s | 5.9 tok/s | 4.7 GB |
| Qwen3.5 9B Q4_K_M | 68.2 s | 42.7 s | 3.6 tok/s | 8.9 GB |

Where the time goes (`docs/research-2026-11.md`, section 2, 209 documents, shipped flags):
prefill 59.8%, reply 35.6%, extraction and OCR 4.5%.

**The arithmetic that decides the student size (derived here):** the 4B takes 52-58% of the
9B's time, so it can only ever pass the 0.6 rule by a hair. The 2B takes about 22%. A student
at 2B size that matched the 9B would be roughly 4.5x faster on the same CPU; a 4B student
roughly 1.8x. The 0.8B (never run here) should be faster again, but how much is unmeasured.

Already measured and rejected for speed (repo, `bench/reports/speed-linux-cpu-*`): two
documents at once, five forms of speculative decoding, shorter ids, batch and polling flags,
8-bit KV cache, Q4_K_S, Q5 and Q8 files, prompt compression. Quantization below Q4_K_M and
runtime flags are not where the next large gain is.

**Not measured anywhere in the repo:** a Windows laptop, 8 threads, battery or power
throttling, the integrated GPU. Every speed figure in ATLAS will carry that caveat until
someone runs the bench on a real laptop.

## 5. The cascade, re-checked (derived here)

The repository turned the cascade off with the 4B as the first model. ATLAS asked whether a
smaller first model changes that. `scripts/simulate_cascade.py` replays the committed alpha.13
readings: the quick model reads every document; only what it would hold for review goes to the
9B. Time is the p50 ratio above.

```
python3 -I research/atlas-models/scripts/simulate_cascade.py
```

| Set | 9B right | 2B→9B right | 9B wrong filed | cascade wrong filed | Sent on to 9B | Time vs 9B |
|---|---:|---:|---:|---:|---:|---:|
| InternBench | 70 | 75 | 1 | 1 | 16% | 0.37 |
| real1 | 22 | 24 | 2 | 3 | 85% | 1.07 |
| real2 | 21 | 22 | 0 | 1 | 87% | 1.09 |
| real3 | 11 | 12 | 0 | 2 | 92% | 1.13 |
| real4 | 15 | 16 | 0 | 1 | 90% | 1.12 |
| All 377 | 168 | 177 | 13 | 24 | 65% | 0.87 |

With the 4B first instead, the cascade costs 1.35x the 9B alone.

Reading: on real documents the off-the-shelf 2B holds 85-92% of documents, so nearly every
document pays for both models and the cascade is **slower** than the 9B alone, and it files
almost twice as many wrong names. It only looks good on InternBench, which is synthetic and
saturated. **An escalation design is only worth building if the first model is confident and
right on most real documents.** That is a property a fine-tuned student might have and the
stock 2B does not; it has to be measured, not assumed. (One caveat: this used the 2B's 0.5
gate from alpha.13, the only gate its readings were scored at.)

## 6. Evaluation assets and what they may be used for

| Asset | Size | Use in ATLAS |
|---|---|---|
| InternBench (`bench/generate.mjs`, `bench/gold.json`) | 77 synthetic | Test only. Its templates (`bench/docs/*.mjs`) are excluded from training data by template family. |
| `bench/recording.json` | 77, 9B, 2026-11-17 | Replay baseline and teacher outputs for analysis; not training data (it is the test set) |
| real1-real6 (`scripts/realdocs-eval/sample*.json`) | 349 public documents, fetched by pin | Development: screening and model selection. Never training data. |
| real7, real8 | 110 | **Never screened** (repository rule). Held out for the final answer only, and only with Zack's say-so. |
| CUAD contracts, fixtures | 63 | Development |

No separate training split exists in the repository. Every scored set above is reserved for
testing, so ATLAS training data has to be new: generated by new templates that share no
template family with InternBench, or public documents not in any sample, labelled and
verified (see [`DATASET.md`](DATASET.md)).

## 7. What the baseline says ATLAS should do

1. Treat **"a fine-tuned 2B-class (or smaller) Qwen3.5 student that matches the 9B on real1-6
   at about a quarter of its time"** as the main hypothesis. Off-the-shelf screening has been
   done thoroughly here already (Spark-X, Gemma 4 E4B, Nemotron, Granite 8B, Bonsai: none led);
   fine-tuning has never been tried.
2. Aim training at the party decision, the main error.
3. Keep the student inside the current interface (same prompt, grammar, handles, gate), so it
   is a drop-in GGUF and every existing benchmark applies unchanged.
4. Do not build escalation until a student's real-set hold rate is known.
5. Measure on runners first (free, comparable with the repository's history), and only then ask
   for laptop and GPU time.
