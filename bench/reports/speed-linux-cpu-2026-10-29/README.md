# Making Intern faster: what was measured, October 2026

Status: measured on 2026-10-29. The question was: can shorter line ids, two documents at
once, or speculative decoding make Intern faster? The answer for all three is no on this
hardware. One change not on that list, the llama.cpp runtime itself, is worth 18%.

**Machine.** An otherwise idle 4-vCPU Linux x86-64 host.

**Model and runtime.** Qwen3.5-2B Q4_K_M (the pinned accurate model) on the packaged runtime,
Prism b10826, with the app's flags: `--parallel 1 --ctx-size 8192 --threads 4 --threads-batch 4
--jinja --no-webui --cache-ram 0 -ub 256 --prio -1 --poll 0 --no-mmproj`.

## Where a document's time goes

The figures come from a live 77-document InternBench run with production settings: today's
capture, about 33 minutes. Model inference is **90%** of the time. **Prefill** (reading the
prompt) is 60% and **decode** (writing the reply) 29%. Extraction (parsing, OCR) is 10%,
almost all of it OCR on scanned pages.

| Per document | Median | Mean |
| --- | ---: | ---: |
| Total | 22.9 s | 25.8 s |
| Prefill | 13.8 s (870 new prompt tokens at 64 tok/s; 477 instruction tokens cached) | 15.6 s |
| Decode | 7.3 s (88 reply tokens at 12 tok/s) | 7.5 s |
| Extraction | 0.01 s (born-digital) | 2.6 s |

So the levers, by size:
- how fast the runtime reads a prompt;
- how long the prompt is;
- only then how fast the model writes, which is all that speculative decoding speeds up.

## The battery: one setting at a time, same requests

Method:
- 26 of the 77 recorded requests (every third, a fixed subset) were replayed against a fresh
  server with the app's flags plus the setting under test. The tool is
  `scripts/bonsai-eval/replay-server.sh` with `EVERY=3`.
- Before timing, each server slot is warmed with the capture's warm-up request, as in an app
  that has read one document.
- Every reply is compared byte for byte with the reply the same request got in the live
  capture.
- Raw numbers are in `battery.json`.

| Setting | Time per document | vs production | Prefill (median) | Decode (median) | Replies identical to its baseline |
| --- | ---: | ---: | ---: | ---: | ---: |
| **Production** (Prism b10826) | 23.7 s | — | 14.1 s | 7.4 s | 26/26 (capture) |
| Without token probabilities | 23.7 s | 0% | 14.2 s | 7.3 s | 26/26 |
| **Upstream llama.cpp b11946** | **19.5 s** | **−18%** | **9.0 s** | 8.7 s | 5/26 vs Prism |
| Upstream, two documents at once | 17.8 s per document* | −25% | 10.9 s | 19.5 s | 3/26 vs upstream |
| Prism, two documents at once | 22.7 s per document* | −4% | 15.2 s | 26.1 s | 9/26 |
| Upstream + MTP speculation | 21.9 s | −8% (+12% vs upstream) | 10.3 s | 10.1 s | 12/26 vs upstream |
| Prism + MTP speculation | 28.5 s | **+20%** | 15.9 s | 9.6 s | 12/26 |
| Prism + n-gram speculation (`ngram-mod`) | 24.2 s | +2% | 13.9 s | 7.7 s | 20/26 |
| Prism + n-gram speculation (`ngram-simple`) | 23.9 s | +1% | 14.3 s | 7.3 s | 26/26 |
| Prism + 0.8B draft model | 44.0 s | **+86%** | 22.5 s | 15.7 s | 12/26 |

\* Throughput: wall time divided by documents. With two at once, each document waits about
twice as long (median 43 s on Prism, 34 s upstream).

### What it means

- **Token probabilities cost nothing measurable.** The review gate (#42) turned them on for the
  accurate model. Time was unchanged and every reply was byte-identical.
- **The upstream runtime reads prompts 36% faster.** The Qwen3.5 prefill in upstream b11946 is
  much faster than in Prism b10826, which is about 1,100 upstream builds behind (Prism's newest
  release, b10828, is still that far behind). Its decode is about 15% slower. Since prefill
  dominates, upstream is **18% faster** overall.

  Its replies are not the same: only 5 of 26 matched byte for byte, so it has to be judged by
  accuracy, not by identity. See [Upstream accuracy](#upstream-accuracy).

  Bonsai's ternary weights use a tensor type (142) that only Prism reads. Running Qwen on
  upstream therefore means shipping a second runtime, or dropping the fast tier.
- **Speculative decoding loses on this CPU, in every form tried.**
  - **MTP.** Qwen3.5's own multi-token-prediction head, converted from the Hugging Face
    weights, was accepted 55% of the time (runs of 2.2–2.8 tokens). Even so, decode slowed
    from 7.4 s to 9.6 s and prefill from 14.1 s to 15.9 s: the head runs over the prompt too,
    and verifying a draft on a CPU costs nearly as much as the tokens it saves.
  - **Draft model.** A 0.8B draft model nearly doubled the time.
  - **N-gram drafting.** Barely proposed anything (`ngram-simple`: 66 draft tokens in 26
    requests, none accepted).
  - **Why the hybrid hurts.** Qwen3.5 is a hybrid: its recurrent layers cannot simply drop a
    rejected draft. llama.cpp restores a checkpoint and replays the accepted tokens, except
    for MTP-family drafters, which get per-sequence state snapshots
    (`common_params_speculative::need_n_rs_seq`). That overhead is paid on every partial
    rejection.
  - **Drift.** Speculation also changed replies: 12 of 26 differed from plain decoding at
    temperature 0, because batched verification moves the arithmetic enough to flip a choice.
- **Two documents at once is not worth the redesign here.**
  - **Little to gain.** Prefill is compute-bound, and both sequences share 4 threads, so only
    decode batches. That nets 4% on Prism and 8.5% upstream, while each document's own wait
    doubles.
  - **Replies drift too.** Only 3–9 of 26 stay identical, so it would need its own accuracy run.
  - **The queue would need rebuilding.** It runs one document at a time under a lease; two at
    once means two leases, two cancellations and two progress streams.
  - **A cheaper variant has a low ceiling.** Overlapping the next document's extraction with
    the current inference is cheaper, but extraction is 10% of the time.

## Shorter line ids and a plainer prompt (live runs, accuracy included)

These change the prompt, so they were run live over all 77 documents, on the same machine and
runtime. All three are scored with the same rules, #40's; #42's issuer rules add three names
to each.

| Prompt | Names right | Wrong but ready | Trap dates | Time per document (mean) |
| --- | ---: | ---: | ---: | ---: |
| Production (`[p1.b4.r2]` line ids) | 58 | 10 | 0 | 25.8 s |
| Ordinal ids (`[3]`, `--- page N ---`) | 50 | 13 | 5 | 25.6 s |
| Plainer scaffold (`1 page, every line:`, `<<<`/`>>>`) | 58 | 12 | 1 | (not timed: compiles ran beside it) |

- **Ordinal ids.** They cut the prompt by 5% and the reply by 24% of its tokens. The time per
  document fell only 1%, because much of decode is fixed per-request cost. The model cited
  line 1 for most facts, which broke the title and date checks that read the cited line.
  **Rejected.**
- **Plainer scaffold.** It was meant to stop the model echoing "Evidence" and "The whole
  1-page document" as a type or party. It changed 19 names in both directions for a net tie
  and two more unsafe names. **Rejected.** The echo case it targeted is now fixed
  deterministically (#42, the price list).

## Upstream accuracy

(Live capture on upstream b11946 in progress; filled in below when it completes.)

## Reproduce

```bash
# 1. A live capture with production settings, keeping each request body (wire/) and token probabilities.
MODEL=Qwen3.5-2B-Q4_K_M.gguf LLAMA_SERVER=.../llama-server INTERN_BENCH=target/release/intern-bench \
INTERN_RUNTIME_DIR=... THREADS=4 LLAMA_SERVER_ARGS="--cache-ram 0 -ub 256 --prio -1 --poll 0" \
INTERN_BENCH_TOKEN_CONFIDENCE=1 BENCH_PROFILE=empty-profile.json \
  scripts/bonsai-eval/internbench-chunked.sh runs/qwen-prism-stable
# (intern-bench built with --features benchmark-transport; empty-profile.json is {"settings":{}})

# 2. One battery row: replay every third request against a server with the setting under test.
WIRE="$(ls -d runs/qwen-prism-stable/chunks/*/wire | tr '\n' ' ')" REFERENCE=runs/qwen-prism-stable/chunks \
EVERY=3 MODEL=... LLAMA_SERVER=... scripts/bonsai-eval/replay-server.sh out/prism-mtp \
  --spec-type draft-mtp -md Qwen3.5-2B-mtp-Q8_0.gguf
# Variants: STRIP_LOGPROBS=1; PARALLEL=2 CTX=16384 CONCURRENCY=2; --spec-type ngram-mod|ngram-simple;
# --spec-type draft-simple -md Qwen3.5-0.8B-Q4_K_M.gguf
```

The MTP head is the `mtp.*` tensors of `Qwen/Qwen3.5-2B` (`mtp_num_hidden_layers` 1),
converted to a GGUF of its own (588 MB at Q8_0) with the converter of the upstream llama.cpp
tag used. The shipped unsloth GGUF does not carry it.
