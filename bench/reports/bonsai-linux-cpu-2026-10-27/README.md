# Bonsai 2 (2B / 4B) on CPU for Intern: runtime, flags, accuracy and routing

Status: complete (2026-10-27). Written incrementally across three container restarts.

Machine: 4 vCPU Intel Xeon, 15 GB RAM (all reported measurements: Cascade Lake @ 2.80 GHz with AVX-512 VNNI - the VM
moved hosts during the work, see "Host CPU caveat"),
Ubuntu 24.04, no GPU. Shared with other agents; every timing below ran under an exclusive lock
(`flock -x bench.lock`), builds under a shared lock. The app's default thread count on this VM is 2
(half the logical cores, clamped 2..12, `crates/intern-engine/src/server.rs`).

## Summary (numbers first)

**Accuracy (InternBench, 77 documents, live, same host/runtime/flags, 4 threads):**
Bonsai 2B **37/77** names, 23 unsafe-ready, 30% review, 1,085 s inference · Bonsai 4B **48/77**, 22 unsafe-ready, 18%
review, 2,729 s · current Qwen3.5-2B (production file, live here) **52/77**, 18 unsafe-ready, 14% review, 1,585 s
(its committed recording replays at 54/77, 15 unsafe). A 4B control run without the few-shot profile: 47/77, 24 unsafe.
**Neither Bonsai model matches the current Qwen model on this corpus**; 4B is ~1.7x Qwen's inference time, 2B ~0.7x.

**Runtime:** ship **Prism `prism-b10826-c9c34bc` official prebuilt archives** (Windows CPU zip 19,559,127 bytes,
sha256 `1e88ab9e...daa6`; Linux tar.gz 17,912,930 bytes, sha256 `66a250f3...28c`). They are as fast as a source
build of the same code (interleaved repeat: 53.3 vs 53.8 tok/s prefill, 12.5 vs 12.4 decode at 2 threads), slightly
faster than b10777, and carry CPU correctness fixes. Use **PQ2_0** files (PTQ1_0 halves prefill; Q2_0 is ~20x slower
on the fork). Upstream llama.cpp b11912 runs the Q2_0 files but prefills ~40% slower than Prism PQ2_0 on the same
AVX2 code path (2B 18.0 vs 29.1 tok/s, 4B 6.5 vs 11.1 at 2 threads).

**Exact server command to ship** (app flags + two additions; `--prio -1 --poll 0` optional for background):
`llama-server --model <Ternary-Bonsai-2-{2B,4B}-PQ2_0.gguf> --parallel 1 --ctx-size 8192 --n-gpu-layers 0
--threads N --threads-batch N --jinja --no-webui --no-mmproj --cache-ram 0 -ub 256` with N = half the logical cores
(2..12).

**Flags:** keep the app's flags (one slot, 8192 context, threads = half the logical cores, f16 KV, flash-attention
auto) and add **`--cache-ram 0`** and **`-ub 256`**:
* `--cache-ram 0`: identical replies, +1-2% time, and it stops the server growing by ~80 MB (2B) / ~280 MB (4B) per
  document (4B reached 6.0 GB after 12 documents with the default 8 GiB cache; 2.95 GB with it off). The 628-token
  prompt prefix is still reused. This applies to today's Qwen setup too.
* `-ub 256`: 13-14% faster documents on AVX2-only CPUs (most laptops), neutral on AVX-512.
* Background: `--prio -1 --poll 0` (+ OS nice) cost nothing measurable; idle CPU of a loaded server is ~0.
* **Do not use MTP on CPU**: +10-24% wall time at 69% draft acceptance, more RAM, and 1/12 replies changed.
  `-ub 1024/2048`, q8_0 KV and turning flash attention off: no gain.

**Speed (app default 2 threads, this host, Intern-shaped requests ~1.0-2.7k prompt tokens with a 628-token cached
prefix, 50-120 generated):** 2B ~21 s/document inference (prefill ~58 tok/s, decode ~12 tok/s), 4B ~61 s
(22.5 / 5.6 tok/s), Qwen ~39 s. On the AVX2-only code path (most laptops) the same requests take 1.5x (2B) / 1.7x
(4B) as long even with `-ub 256` (1.7x / 1.9x with the default -ub 512): ~41 s and ~108 s per 1.2-2.1k-token
document at 2 threads. Start-up 2-5 s warm, up to ~1-2 min cold for 4B.

**Routing policy:** do **not** use "2B first, escalate to 4B when 2B asks for review": 4B clears 8 of those review
flags under wrong names (unsafe-ready 23 -> 31). If both Bonsai models ship, the best simple rule found is
*2B first; re-ask 4B when 2B sends to review OR names fewer than two parties; take 4B's name but never clear a
review flag*: 46/77 names, 18 unsafe-ready, 33% review, 84% of 4B-always inference time (selected on these 77
documents - optimistic). "4B always" is simpler and as accurate (48/77, 22 unsafe). Requiring 2B and 4B to agree
before filing as ready cuts unsafe-ready to 4 (2-fold cross-validated) but sends 65% of documents to review and costs
both models on every document. None of these reaches the current Qwen model; on this evidence keep Qwen as the
default and treat Bonsai 2B as a low-memory / fast fallback with a sticky review flag. Independently of routing,
a token-confidence review gate (ready -> review when the date/party minimum token probability < 0.5) removes 4
unsafe-ready names for 2-3 correct ones on either Bonsai model.

**RAM (14 GB laptop):** with `--cache-ram 0`: 2B server ~1.4 GB, 4B ~2.95 GB on the AVX-512 variant (0.9 / 1.9 GB on
the AVX2 variant), worker peak 0.71 GB (25-page OCR); total ~2.2 / ~3.7 GB. One model resident at a time.

## Pins

### Models (Hugging Face, prism-ml)

| File | Repo @ commit | Bytes | SHA-256 | Verified |
| --- | --- | ---: | --- | --- |
| Ternary-Bonsai-2-2B-PQ2_0.gguf | prism-ml/Ternary-Bonsai-2-2B-gguf @ 683f7be2629342a50987627b40881e54761a2430 | 517,312,992 | d5f01a4ecaf49239653308ee732ad1a0c5f802c0ddb997549fc7b694c8b45928 | sha256sum OK |
| Ternary-Bonsai-2-2B-PTQ1_0.gguf | same | 429,183,456 | 36061ad406df8a4903a6db005be350c50fabfaf717f9ef0acf4b76adcd9929fa | OK |
| Ternary-Bonsai-2-2B-Q2_0.gguf | same | 546,689,504 | c2893fa6f951daee8b7263d5f49b455e57989ef56417a22125b03566dc560650 | OK |
| Ternary-Bonsai-2-4B-PQ2_0.gguf | prism-ml/Ternary-Bonsai-2-4B-gguf @ 8d6557168168f91ea6697a4a49ed2ef622b53adb | 1,146,465,120 | d406b1e912c0d2a4c124ad10bf38a76ae662fdc4d24b176451403dc7b7f30409 | OK |
| Ternary-Bonsai-2-4B-PTQ1_0.gguf | same | 949,549,920 | 56711263ec9a9ffeaa74a3a0e58dece8127cf9268404746c721113ca76994d60 | OK |
| Ternary-Bonsai-2-4B-Q2_0.gguf | same | 1,212,103,520 | 64a5183b44921bdb5ac4a6f5f92007821d6ebe5c66997d3c147fd214ab9433d4 | OK |
| Ternary-Bonsai-2-4B-PQ2_0-MTP-Q8_0.gguf | same | 1,274,642,176 | 0acf517de1dc3d85c403a60df6b8b07a4878ac174f368d4390be98406a4d6c0d | OK |

URLs: `https://huggingface.co/prism-ml/Ternary-Bonsai-2-<size>-gguf/resolve/<commit>/<file>`.
SHA-256 values are the repositories' LFS object ids (`cache/bonsai/SHA256SUMS-*`) re-hashed locally.

### Runtimes

| Runtime | Source | Archive | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| Prism b10777 (tested on Windows earlier) | PrismML-Eng/llama.cpp tag prism-b10777-e3de715, commit e3de715222b2667c54a267441159a5729f11155e | llama-prism-b10777-e3de715-bin-ubuntu-x64.tar.gz | 17,882,710 | 3f97b6fbec0065fede2be92f05df531c1050636fc2613e1ba0d975249df19713 |
| | | llama-prism-b10777-e3de715-bin-win-cpu-x64.zip | 19,530,958 | cff7eeff19163e6263720618cf98023715b8dac14d0873f5be0918765de323c4 |
| Prism b10826 (candidate) | tag prism-b10826-c9c34bc, commit c9c34bc9c4042c13a31598ebf43461cd0ad8f00d | llama-prism-b10826-c9c34bc-bin-ubuntu-x64.tar.gz | 17,912,930 | 66a250f38c3f8e2064e2f913b3a26ff5bae10955b67e8ff38c72ab48cbecf28c |
| | | llama-prism-b10826-c9c34bc-bin-win-cpu-x64.zip | 19,559,127 | 1e88ab9e858b5c2dbc30cf847947a9f923b53511ba0baf0b265353ebe1f4daa6 |
| Upstream llama.cpp b11912 (runs Q2_0 only) | ggml-org/llama.cpp tag b11912, commit 961c376a2daa42220ec09663b9bde32c5fba1f70 | source build, `scripts/build-llama-cpu-linux.sh ... avx2` | | |

Archive URLs: `https://github.com/PrismML-Eng/llama.cpp/releases/download/<tag>/<archive>`.

### What b10826 changes relative to b10777 (49 commits, from the tagged sources)

The GitHub release pages could not be read from this session (repository API access is not enabled), so this
reviews the commit log between the two tags instead (`runtime/prism-b10777..b10826-commits.txt`,
details in `runtime/prism-b10826-cpu-relevant-commits.txt`). Most of the 49 commits are CUDA/Hopper work that
does not touch a CPU build. The ones that matter for Intern's CPU path:

| Commit | Change | Relevance to Intern |
| --- | --- | --- |
| 30bc022 | AVX-512 VNNI, AVX2+AVX-VNNI and AVX2 `vec_dot` for PTQ1_0 x Q8_0 ("about 2x CPU prefill and decode over the SSSE3 path; greedy output identical") | Makes PTQ1_0 usable on CPU; b10777 runs PTQ1_0 on an SSSE3 fallback |
| 67a9ab9 | ggml-cpu: fix crash on tensors without repack traits / reads of packed layouts | CPU correctness (crash fix) |
| a75f770 | grammar clone by binary search (8.4 ms -> 0.4 ms per clone for a big grammar) | Intern always sends a GBNF grammar; the server clones the grammar on each speculative step, so this matters for MTP |
| 5ae2e16 | recurrent memory: keep a pending rollback across `seq_cp`; MTP depth default 2 on CUDA | Correctness of recurrent-state rollback (Bonsai 2 is a hybrid recurrent model); CPU MTP default unchanged (3) |
| ac9c5fe | server: cheaper per-token `n_probs`/top-k host time, byte-identical | Cheaper token-probability requests (Intern's optional token confidence) |
| 3d04d9b | sanitize malformed UTF-8 in chat parsing instead of failing | Robustness: OCR text can contain odd bytes |
| 5979922 | json-schema-to-grammar: empty unions no longer produce an unparsable grammar | Not used by Intern (it sends GBNF directly) |
| b87ca72 | ggml-cpu: split GET_ROWS/CONCAT work when there are few rows (recurrent state copies) | Small CPU speed-up for hybrid models, results unchanged |

## Method

* **Runtime matrix and flag sweeps**: `llama-bench` (`scripts/bonsai-eval/phase1.sh`, `phase2a.sh`), JSONL in
  `microbench/runtime-matrix.jsonl`, `microbench/flag-sweep.jsonl`. CPU variants that a laptop would load
  (AVX2 "haswell", AVX2+AVX-VNNI "alderlake") were forced on this AVX-512 machine by giving the server a
  directory holding only that `libggml-cpu-<variant>.so` (`scripts/bonsai-eval/variant-dir.sh`). By default the
  Prism/ggml loader picks `sapphirerapids` here.
* **End-to-end server measurements**: `scripts/bonsai-eval/serverbench.py` starts `llama-server` exactly with
  the app's flags (one slot, CPU, 8192 context, `--jinja --no-webui --no-mmproj`, threads = threads-batch) plus the
  flag under test, sends real Intern requests (the 12 exact forwarded requests captured in the Windows
  qualification, `bench/reports/bonsai-native-windows-2026-10-18/*-raw.jsonl`, and 2k/4k/6.5k-token stretched
  variants from `make_requests.py`), and records the server's own prefill/decode timings, wall time, server CPU
  seconds, start-up time, VmHWM/RSS (anon and file-backed), and idle CPU. `phase2b.sh`, results in
  `microbench/server.jsonl`.
* **Accuracy**: full live InternBench (77 documents, `bench/gold.json`; corpus regenerated and verified
  77/77 against `bench/manifest.json`) through `scripts/run-internbench-candidate.sh` with the frozen few-shot
  greedy profiles (`plans/profile.json` for 2B, `plans/bonsai4-profile-17.json` for 4B; both carry the same
  prompt, which equals the engine's current system prompt plus one synthetic few-shot pair, seed 17, temperature 0,
  top-k 1) via the `benchmark-transport` feature, Prism b10826 prebuilt runtime. Because this container restarted
  twice, runs are made in chunks of 8 documents (`scripts/bonsai-eval/internbench-chunked.sh`: a fresh server and
  warm-up per chunk, each chunk under the exclusive lock), then the chunk recordings are merged
  (`intern-bench merge-recordings`) and replayed into one report whose timings are the live ones.
  `INTERN_BENCH_TOKEN_CONFIDENCE=1` (a lab-only opt-in added to `intern-bench`, commit in the worktree) asks the
  server for token log-probabilities so each record carries `token_confidence` (greedy decoding: no token changes).
* **Qwen reference**: the committed recording of record (`bench/recording.json`, Qwen3.5-2B Q4_K_M, recorded live
  on this same VM type at 4 threads) replayed with the current code (`internbench/qwen-replay-current/`): the same
  scores as `bench/reports/2026-10-07-phase3-after.md`. The Qwen GGUF in the shared cache (422,870,040 bytes,
  sha256 fce1ec...) is **not** the production file (1,280,835,840 bytes, sha256 aaf42c8b...), so Qwen was not re-run.

## Host CPU caveat

The VM was moved to a different host by the container restarts. Until about 23:40 UTC it was a Sapphire Rapids
class Xeon @ 2.10 GHz (AVX-512, AVX-VNNI, AMX; the committed Qwen baseline was recorded on that type). From the
restart at 23:40 UTC on it is a **Cascade Lake Xeon @ 2.80 GHz (family 6, model 85, stepping 7: AVX-512 with
VNNI, no AMX, no AVX-VNNI)** - `host/lscpu-*.txt`. All InternBench runs and all microbenchmarks reported below
as "this host" ran on the Cascade Lake host; measurements made earlier on the Sapphire Rapids host are kept in
`microbench/*.host-spr-2.1GHz.jsonl` and are not mixed into the tables. Cross-host time comparisons (e.g. with the
committed Qwen timings) are therefore indicative only.

## Accuracy on InternBench (77 documents)

Bonsai runs: Cascade Lake host, **4 threads** (to finish in the time available; the app default on this 4-vCPU VM
is 2 - see the 2-thread check below), app server flags, Prism b10826 prebuilt (auto CPU variant), few-shot greedy
profile, token log-probabilities on. Qwen, two ways: the committed recording of record replayed with the current code (4 threads, upstream b10361,
Sapphire Rapids host; its timings are from that host), and a fresh live run of the pinned production file
(`src-tauri/resources/model-manifest.json`: unsloth/Qwen3.5-2B-GGUF @ f6d5376be1edb4d416d56da11e5397a961aca8ae,
1,280,835,840 bytes, sha256 aaf42c8b..., re-downloaded and verified) on this host with exactly the Bonsai runtime and
flags and the app's own prompt (`internbench/qwen-t4-app/`). Live-to-replay difference (52 vs 54 names, 18 vs 15
unsafe) is the run-to-run/runtime noise to keep in mind when reading single-document differences.
Reports: `internbench/{2b-t4-app,4b-t4-app,qwen-replay-current}/report.{md,json}`, recordings next to them, wire
captures under `internbench/*/chunks/*/wire/`.

| Run | Filename | Type | Date | Parties | Relation | Unsafe ready | Review | Needless review | Trap date | Forbidden party | Sum inference | Total p50 / p95 / max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Bonsai 2B PQ2_0 | **37/77 (48%)** | 66 | 67 | 55 | 46 | **23** | 23 (30%) | 2 | 5 | 1 | 1,085 s | 14.4 / 27.5 / 99.9 s |
| Bonsai 4B PQ2_0 | **48/77 (62%)** | 70 | 70 | 68 | 61 | **22** | 14 (18%) | 4 | 6 | 2 | 2,729 s | 34.6 / 65.3 / 114 s |
| Qwen3.5-2B Q4_K_M (production, replay) | **54/77 (70%)** | 72 | 75 | 67 | 61 | **15** | 12 (16%) | 2 | 1 | 8 | 1,492 s (other host) | 19.2 / 33.9 s (other host) |
| Qwen3.5-2B Q4_K_M (production file, **live on this host**, same runtime and flags as Bonsai, app prompt) | **52/77 (68%)** | 70 | 75 | 64 | 62 | **18** | 11 (14%) | 3 | 1 | 10 | 1,585 s | 20.5 / 35.6 s |

* **Both Bonsai models are less accurate than the current Qwen model on InternBench**, and both file more wrong
  names without review. Speed on the same host and flags: Bonsai 2B inference 1,085 s (0.68x Qwen's 1,585 s),
  Bonsai 4B 2,729 s (1.72x Qwen). The 12-document Windows cohort (2B 8/12, Qwen 9/12, 4B 11/12) did not show this; on 77
  documents 4B vs Qwen is 48 vs 54 names and 22 vs 15 unsafe-ready. The paired bootstrap interval for 4B minus 2B is
  +11 names [+2, +20]; for Qwen vs 4B the gap is in the same direction on names and unsafe-ready.
* **The few-shot profile is not what holds 4B back.** A control run of 4B with the app's own prompt and no
  example (`internbench/4b-t4-noprofile/`) scores 47/77 names, 24 unsafe-ready, 9 reviews (few-shot: 48 / 22 / 14),
  i.e. the same within run-to-run noise; the profile mainly makes 4B send a few more documents to review.
* Typical 2B failures: issuer/recipient left out (21 documents with fewer than two parties, 18 of them wrong),
  "Document" as the type (letters, email, receipt, remittance), cover/issue dates on long reports, wrong date
  on SOW/lease (referenced or earlier dates). 4B fixes 17 of 2B's wrong names, mostly by naming the missing
  party, and breaks 6 that 2B had right (e.g. "The Lenders Party Hereto" as a party, generic "MINUTES",
  "RESOLUTION", a ".. - Parts and Quotes" suffix).
* Prompts with the profile are 1.0k-2.7k tokens in total; the 628-token system + example prefix is served from
  llama-server's prompt cache for every document after the first (`cached_tokens` = 628). Replies are 50-118
  tokens (p50 88-90), far below the 250-400 assumed in the brief. Prefill dominates inference time.
* Prompt SHA-256 identical for 2B and 4B on 77/77 documents (same extraction, same retrieval), so the escalation
  cost model below (re-ask 4B with the same prompt) is exact.

## Routing: when should Intern use 2B vs 4B?

Data: the two 77-document runs above (same prompts, 4 threads). Signals are only what the app has after the 2B
pass (readiness and review reasons, the 2B reply, validation support counts, route/page count/format, prompt
length, token confidence). Tables: `routing/routing.md` (every single signal and "review OR x" pair),
`routing/policies-readiness.md` (readiness rules), per-document CSV `routing/per-document.csv`.
Sum inference is model time over the 77 documents at 4 threads; extraction (~190 s) is the same for every policy;
a model swap (unload 2B, load 4B from page cache, ~3-4 s) is extra per escalated document unless batched.

| Policy | 4B calls | Filename | Ready | Unsafe ready | Needless review | Sum inference |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2B always | 0 | 37 | 54 | 23 | 2 | 1,085 s |
| 4B always | 77 | 48 | 63 | 22 | 4 | 2,729 s |
| 2B, escalate on 2B review, take 4B's result | 23 | 41 | 69 | **31** | 1 | 1,822 s |
| 2B, escalate on review, keep 2B's review flag ("sticky") | 23 | 41 | 54 | 23 | 6 | 1,822 s |
| 2B, escalate if 2B names < 2 parties, take 4B | 21 | 44 | 56 | 21 | 5 | 1,818 s |
| 2B, escalate on review OR < 2 parties, sticky review | 37 | 46 | 51 | 18 | 9 | 2,292 s |
| 2B, escalate on review OR token-confidence min < 0.7, sticky | 47 | 46 | 49 | 16 | 9 | 2,777 s |
| Both models on every document; 4B's name; ready only if both ready and the names agree | 77 (+77 2B) | 48 | 27 | **4** | 17 | 3,814 s |
| Oracle (escalate exactly where 4B fixes the name) | 17 | 54 | 56 | 11 | 5 | 1,664 s |
| Qwen live on this host (reference) | - | 52 | 66 | 18 | 3 | 1,585 s |
| Qwen committed recording, replayed (reference, other host) | - | 54 | 65 | 15 | 2 | 1,492 s |

Findings:

1. **"2B first, escalate on review" is worse than both alternatives on safety.** The 23 documents 2B sends to
   review are mostly ones 4B gets wrong too (only 5 of 17 4B fixes are among them), and 4B marks 8 of them ready
   under a wrong name: unsafe-ready rises from 23 to 31. 2B's review flag is a good *safety* signal (0 of its 23
   review documents is unsafe by definition, 17 of 23 have a wrong name) but a poor *escalation* signal.
2. **Where 4B helps is on documents 2B marked ready**: 12 of the 17 fixes are 2B-ready documents (missing issuer,
   wrong type word). The best single runtime signal for them is "2B named fewer than two parties" (fires on 21;
   18 of them wrong; 9 of the 17 fixes; 11 of the 23 unsafe-ready). Token confidence ranks errors (min < 0.3: 8 of
   10 wrong; min >= 0.9: 12 of 13 right) but is not separable: at 0.7 it fires on 40 documents (27 wrong) and
   catches 12 of 17 fixes - nearly the cost of running 4B on everything.
3. **Any escalation must never clear a review flag** ("sticky" rule): with it, escalating on review OR < 2 parties
   gives 46 names (vs 48 for 4B always), unsafe-ready 18 (vs 22) at 84% of 4B-always inference time - the best
   cost/benefit of the simple rules, but it is chosen after looking at these 77 documents.
4. **Agreement between the two models is the strongest readiness signal found**: where 2B and 4B produce the same
   name (33/77), 28 are right (85%); requiring agreement cuts unsafe-ready to 4 but sends 65% of documents to
   review and costs 2B+4B on every document. 2-fold cross-validation over 200 random splits (choose the rule on one
   half by names - 2 x unsafe, score on the other) picks this rule in 198/200 splits; held-out gain over 2B-always
   median +25 (5-95%: +16..+34) on 38-39 documents.
5. **Token confidence works better as a review gate than as an escalation trigger.** Sending a *ready* document to
   review when the minimum token probability of its date/party values is below 0.5 (the threshold
   `docs/model-candidates.md` already tried; not tuned here; `Engine::with_min_token_confidence` exists):

   | Model | Unsafe-ready without -> with gate | Correct names newly sent to review |
   | --- | ---: | ---: |
   | Bonsai 2B | 23 -> 19 | 2 |
   | Bonsai 4B | 22 -> 18 | 3 |
   | Qwen (live) | 18 -> 10 | 7 |

   Stricter thresholds (0.7-0.9) remove most unsafe-ready documents but send 9-40 correct names to review.
   Log-probabilities cost +0.7-2% time and do not change replies (server benchmark).
6. **Overfitting caveat**: 77 synthetic documents, one run per model, rules and thresholds were read off the
   same data (only the agreement rule was checked with cross-validation, and its selection is stable). The
   escalation gains are small in absolute terms (4-9 names) and their bootstrap intervals include zero for
   review-escalation vs 2B; none of the policies reaches the current Qwen model (52-54 names, 15-18 unsafe-ready).

## Thread count and determinism

The same 16 documents (two chunks: short native PDFs plus scans, a newsletter, column layouts) were re-run at
**2 threads, the app's default on this VM**, without token log-probabilities (`internbench/*-t2-sub16`), and
compared reply by reply with the 4-thread runs (`scripts/bonsai-eval/compare-runs.py`).

| Model | Identical model reply | Identical filename + readiness | Inference 2 threads / 4 threads |
| --- | ---: | ---: | ---: |
| Bonsai 2B | 12/16 | 15/16 | 354 s / 239 s = 1.48x |
| Bonsai 4B | 15/16 | 16/16 | 855 s / 501 s = 1.71x |
| Qwen3.5-2B | 11/16 | 15/16 | 548 s / 292 s = 1.88x |

* **Greedy output is not bit-identical across thread counts for any of the three models.** The differences are
  near-tied tokens (a party role "licensee" vs "tenant", an extra third party, an evidence id, `issuance` vs
  `invoice`, "INVOICE" vs "Invoice"); one Qwen document flipped from a ready (wrong) name to review. Single-document
  flips between runs are noise; read the aggregates.
* Projected 77-document inference at the app default of 2 threads on this host (4-thread sums x the ratios above):
  **2B ~1,600 s (~21 s/document), 4B ~4,670 s (~61 s/document), Qwen ~2,980 s (~39 s/document)**, plus ~190 s
  extraction for the corpus. On a 2-thread laptop core budget 4B costs about 2.9x 2B and 1.6x Qwen per document.

## Memory and the llama-server prompt cache

**Finding: with the app's current flags the model server's RSS grows with every document.** llama-server's
`--cache-ram` defaults to 8192 MiB: after each request it keeps the finished prompt's state in a host-memory cache
for possible reuse. Intern's prompts never repeat (only the 628-token system + example prefix is shared, and that is
served from the slot's own context checkpoint), so this cache only grows, by roughly the prompt's state per document.

| Server (4 threads unless noted) | RSS after load + warm-up | After 8 InternBench docs (peak in a chunk) | Growth / doc |
| --- | ---: | ---: | ---: |
| Bonsai 2B, default flags | ~1.3 GB | 1.85-2.13 GB | ~70-90 MB |
| Bonsai 4B, default flags | ~2.7 GB | 4.4-4.8 GB | ~200-250 MB |
| Qwen3.5-2B, default flags | | 2.96 GB peak | |
| Bonsai 2B, `--cache-ram 0` (12 real requests) | 1.37 GB | 1.42 GB after 12 | ~4 MB |
| Bonsai 4B, default flags (12 real requests) | 2.58 GB | 5.97 GB after 12 | ~280 MB |
| Bonsai 4B, `--cache-ram 0` (12 real requests) | 2.58 GB | 2.95 GB after 12 | ~30 MB (settles) |

(InternBench `chunks/*/rss.json`, sampled every 100 ms; server benchmark rows `S-2B-t4-real12` vs
`S-2B-t4-real12-cram0` in `microbench/server.jsonl`.) The growth is capped only at 8 GiB of cache, so a long
background session with 4B could approach ~10-11 GB for the model server alone - more than half of a 14 GB laptop.

`--cache-ram 0` on the same 12 requests: **identical replies (12/12 for both models), the 628-token prefix is still
reused on every request (`cache_n` = 628), wall time +0.9% (2B) / +1.6% (4B), RSS flat** (2B 1,371 -> 1,418 MiB
instead of 1,483 -> 2,447 MiB; 4B peak 2,953 MiB instead of 5,968 MiB).
This should ship regardless of model choice (it applies to the current Qwen setup too).

RAM headroom on a 14 GB laptop with `--cache-ram 0` (peak RSS; one model resident at a time):

| Component | Peak RSS |
| --- | ---: |
| Bonsai 2B server (model file mapped 0.5 GB + context/compute ~0.9 GB anon) | ~1.4 GB |
| Bonsai 4B server | ~2.95 GB |
| intern-worker (PDFium + PP-OCR/ONNX on the 25-page scan) | 0.71 GB (InternBench peak) |
| intern-bench / app process | < 0.1 GB here (the Tauri UI is extra, not measured) |
| **Total, 2B / 4B** | **~2.2 GB / ~3.7 GB** (+ UI) |

With `--cache-ram 0` either model fits comfortably beside a user's other work on a 14 GB laptop, and 4B leaves
~10 GB; without it, 4B's server alone reaches ~6 GB after a dozen documents and keeps growing toward ~10-11 GB.
Never keep both resident: 2B + 4B together is ~4.4 GB with the cache off, and swapping between them costs only the
load time (2-5 s warm, up to a minute cold).

## Server flags measured end to end (llama-server, Prism b10826, this host)

`microbench/server.jsonl`, summarised by `scripts/bonsai-eval/summarize-server.py`. "real12" = the 12 captured
Intern requests (1.2k-2.1k prompt tokens of which 628 cached, 57-109 generated tokens; a warm-up request first);
"long" = stretched 2.1k / 4.0k / 6.5k-token prompts. Each row is one fresh server; replies are compared with the
default-flag row of the same model and thread count.

| Model | Threads | Flags beyond the app's | Sum wall, 12 requests | Prefill tok/s | Decode tok/s | Peak RSS after 12 | Replies identical | Verdict |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- | --- |
| 2B | 4 | (app flags) | 157.7 s | 104.7 | 19.4 | 2,447 MiB | - | baseline |
| 2B | 2 | (app flags) | 274.1 s (1.74x) | 57.8 | 12.1 | 2,461 MiB | 11/12 vs 4 threads | app default |
| 2B | 4 | `--cache-ram 0` | 159.2 s (+0.9%) | 103.6 | 18.8 | **1,418 MiB** | 12/12 | **ship** |
| 2B | 4 | `-ub 1024` | 165.4 s (+4.8%) | 101.4 | 18.8 | 2,396 MiB | 12/12 | no |
| 2B | 4 | log-probabilities on | 158.9 s (+0.7%) | 104.0 | 19.1 | 2,466 MiB | 12/12 | cheap; fine for a confidence signal |
| 2B | 2 | `--cache-ram 0 --poll 0 --prio -1`, nice 10 | 282.0 s (+2.9% vs 2-thread default) | 56.5 | 11.6 | 1,417 MiB | 12/12 | background profile |
| 4B | 4 | (app flags) | 385.9 s | 42.0 | 9.3 | 5,968 MiB | - | baseline |
| 4B | 2 | (app flags) | 685.7 s (1.78x) | 22.5 | 5.6 | 5,968 MiB | 12/12 vs 4 threads | app default |
| 4B | 4 | `--cache-ram 0` | 392.2 s (+1.6%) | 40.5 | 8.7 | **2,953 MiB** | 12/12 | **ship** |
| 4B | 4 | `-ub 1024` | 410.3 s (+6.3%) | 40.2 | 8.9 | 5,655 MiB | 10/12 | no |
| 4B | 4 | log-probabilities on | 393.8 s (+2.0%) | 41.6 | 8.9 | 5,961 MiB | 12/12 | cheap |
| 4B | 2 | `--cache-ram 0 --poll 0 --prio -1`, nice 10 | 669.2 s (-2.4% vs 2-thread default) | 22.6 | 5.6 | 2,954 MiB | 12/12 | background profile |
| 4B (MTP file) | 4 | `--spec-type draft-mtp --spec-draft-n-max 2 --spec-draft-n-min 0 --spec-draft-p-min 0` | 438.2 s (**+13.5%**) | 37.7 | 7.7 | 6,563 MiB | 11/12 | **no** (see MTP) |

Long prompts (fresh server, 628-token prefix cached, stretched lease text):

| Model | Threads | 2.1k-token prompt | 4.0k | 6.5k | Prefill tok/s at 2.1k -> 6.5k | Decode tok/s |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| 2B | 2 | 37.0 s | 53.5 s* | 70.4 s* | 55.9 -> 50.0 | 11.7 -> 10.3 |
| 2B | 4 | 20.5 s | 31.4 s* | 40.2 s* | 106.1 -> 90.6 | 18.7 -> 15.5 |
| 4B | 2 | 81.0 s | 128.9 s* | 165.6 s* | 22.9 -> 20.0 | 5.4 -> 4.8 |
| 4B | 4 | 44.8 s | 67.5 s* | 88.9 s* | 42.5 -> 37.8 | 9.0 -> 8.1 |

\* the 4.0k and 6.5k prompts extend the previous one, so the server reused 1.6k / 3.5k tokens of it; the wall
times are for evaluating the remaining 2.4k / 3.0k tokens. Throughput falls only 10-15% from 2k to 6.5k of context
(hybrid model: only a quarter of the layers are full attention).

Start-up: 2.2-2.6 s (2B) and 3.2-5.3 s (4B) with the model in the page cache; after a container restart (cold disk)
the first start took 46-56 s (2B) and 41 s / >60 s (4B, one InternBench chunk failed the old 60-second health wait),
and the first start of the 1.27 GB MTP file took 117 s. Plan for a minute-plus cold load of 4B on a slow laptop disk.
Idle CPU of a loaded server is ~0 (no spinning between requests at the default `--poll 50`).
Background cost: the server uses about the same CPU-seconds per document at 2 and 4 threads (2B: 536 vs 604 s for
the 12 requests; 4B: 1,324 vs 1,441 s), so 2 threads simply leaves half the cores free at ~1.75x the wall time.

## MTP speculative decoding (4B)

`Ternary-Bonsai-2-4B-PQ2_0-MTP-Q8_0.gguf` with `--spec-type draft-mtp --spec-draft-n-max 2 --spec-draft-n-min 0
--spec-draft-p-min 0` (the model card's recommendation), Prism b10826 (which has the grammar-clone and recurrent
rollback fixes), on the 12 real requests with Intern's grammar:

| Threads | Wall vs plain 4B | Decode time vs plain | Prefill tok/s | Draft acceptance | Replies identical to plain | Peak RSS (default cache) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 438.2 s vs 385.9 s (**+13.5%**) | +22% | 37.7 vs 42.0 | 615/886 (69%) | 11/12 | 6,563 vs 5,968 MiB |
| 2 | 844.3 s vs 685.7 s (**+23%**) | +36% | 18.9 vs 22.5 | 615/886 (69%) | 11/12 | 6,568 vs 5,968 MiB |
| 4, depth 1 | 432.5 s (+12.1%) | +15.5% | 37.5 | 487/570 (85%) | 11/12 | 6,509 MiB |
| 4, depth 3 | 424.9 s (+10.1%) | -2.6% | 36.0 | 671/1134 (59%) | 12/12 | 6,607 MiB |
| 4, MTP file, no speculation | 404.3 s (+4.8%) | +5.0% | 39.3 | - | 12/12 | 5,961 MiB |
| 2, long 2.1k-6.5k prompts, depth 2 | 466.0 s vs 375.5 s (+24%) | +70% | 18.3 vs 21.4 | 140/226 (62%) | 3/3 | |

**MTP makes Intern slower on CPU**, not faster: with a 69% acceptance rate the extra draft layer and the
3-token verify batches cost more than they save on a CPU (the card's speed-ups are for CUDA/Metal), prefill is
also ~10-16% slower (the MTP layer runs over the prompt), the file is 128 MB larger, RSS is ~0.3-0.6 GB higher, and
1 of 12 greedy replies differs from plain decoding (as the card warns). Do not ship MTP for CPU.
Only depth 3 at 4 threads recovers decode speed (-2.6%), and it still loses overall because of prefill.

## Runtime choice

`llama-bench`, pp512 / tg64, tokens per second, Bonsai 2B PQ2_0, this (Cascade Lake) host, under the exclusive lock.
Host speed drifts by a few percent over hours, so the decisive comparison was repeated three times interleaved
(`X-*` rows in `microbench/flag-sweep.jsonl`, means of 3 runs x 3 repetitions):

| Runtime (same CPU variant: `cascadelake`, auto-selected) | pp 2 thr | tg 2 thr | pp 4 thr | tg 4 thr |
| --- | ---: | ---: | ---: | ---: |
| **Prism b10826, official prebuilt Linux archive** (GCC 11) | **53.3** | **12.5** | **94.7** | **19.9** |
| Prism b10777, official prebuilt Linux archive | 50.7 | 11.8 | 89.2 | 17.7 |
| Prism b10777, own portable source build (`scripts/build-llama-cpu-linux.sh`, GCC 13, all variants) | 53.8 | 12.4 | 93.2 | 18.9 |

* **The prebuilt Prism release archive is as fast as a source build** (b10826 prebuilt vs b10777 source: within
  1-5%, prebuilt ahead on decode). Ship Prism's own archives, pinned by size + SHA-256 (table above), rather than
  maintaining a build. Their portable `GGML_BACKEND_DL` layout loads the best `libggml-cpu-<arch>.so` at start-up:
  `sapphirerapids` on the first host, `cascadelake` on this one.
* **Bump to b10826**: no CPU regression (slightly faster), both models load and run all InternBench documents on it
  (every run in this report used it), and it carries CPU fixes (repack crash fix, recurrent-state rollback fix,
  grammar clone speed-up, UTF-8 sanitising, 2x faster PTQ1_0 on AVX2/AVX-512).
* A `-march=native` build made on the first (Sapphire Rapids) host dies with *Illegal instruction* on this host -
  exactly why only portable builds may ship. Its earlier numbers on the first host were not faster than the
  portable build there (`microbench/*host-spr*`).

CPU variant matters a lot for prefill - what a laptop gets (forced variants on this host):

| 2B PQ2_0, b10826 prebuilt | pp 2 thr | tg 2 thr | pp 4 thr | tg 4 thr |
| --- | ---: | ---: | ---: | ---: |
| `cascadelake` (AVX-512 + VNNI; auto here) | 47.7 | 11.1 | 80.9 | 17.3 |
| `haswell` (AVX2/FMA only: most laptops before Ice Lake / Zen 4) | 29.1 | 10.5 | 51.8 | 17.2 |

(same session of the runtime matrix, `C-*` rows.) On an AVX2-only laptop prefill is ~40% slower than these
AVX-512 numbers while decode is unchanged; end to end on Intern requests that is 1.5x (2B) / 1.7x (4B) the time with
`-ub 256`, 1.7x / 1.9x with the default 512 (see "AVX2 laptops"). The `alderlake` variant
(AVX2 + AVX-VNNI, Intel 12th gen+ laptops) could not be measured: this host has AVX-512 VNNI but not AVX-VNNI, so
the loader refuses it ("CPU backend is not loaded").

Quantisation packing (same ternary weights, different kernels), 2B, b10826 prebuilt:

| Packing | File | pp 2 thr | tg 2 thr | pp 4 thr | tg 4 thr |
| --- | ---: | ---: | ---: | ---: | ---: |
| **PQ2_0** (cascadelake) | 517 MB | **47.7** | **11.1** | **80.9** | **17.3** |
| PTQ1_0 (cascadelake) | 429 MB | 20.0 | 10.5 | 38.7 | 17.0 |
| PTQ1_0 (haswell) | | 14.8 | 8.5 | 29.0 | 15.0 |
| Q2_0 on the Prism fork | 547 MB | 2.7 | 1.8 | 4.9 | 3.0 |
| PTQ1_0 on Prism **b10777** (SSSE3 fallback) | | 7.8 | 4.6 | 13.8 | 7.1 |
| Q2_0 on the Prism fork | 547 MB | 2.7 | 1.8 | 4.9 | 3.0 |
| Q2_0 on **upstream llama.cpp b11912** (single AVX2 backend, source build) | 547 MB | 18.0 | 9.8 | 33.4 | 15.3 |
| (for a fair upstream comparison: PQ2_0 on Prism's `haswell` AVX2 variant) | | 29.1 | 10.5 | 51.8 | 17.2 |

4B, same session:

| 4B | pp 2 thr | tg 2 thr | pp 4 thr | tg 4 thr |
| --- | ---: | ---: | ---: | ---: |
| PQ2_0, b10826 prebuilt, cascadelake (auto) | 20.8 | 5.4 | 35.4 | 9.0 |
| PQ2_0, b10777 prebuilt, cascadelake | 19.4 | 5.6 | 38.1 | 8.7 |
| PQ2_0, b10826 prebuilt, haswell | 11.1 | 4.9 | 20.8 | 9.1 |
| PTQ1_0, b10826 prebuilt, cascadelake | 7.5 | 4.9 | 13.3 | 7.8 |
| PTQ1_0, b10826 prebuilt, haswell | 5.3 | 3.8 | 10.2 | 6.6 |
| PTQ1_0, b10777 prebuilt (SSSE3 fallback) | 3.0 | 2.0 | 5.5 | 3.5 |
| Q2_0, upstream b11912 (AVX2 build) | 6.5 | 4.1 | 13.5 | 7.1 |

PQ2_0 is the right packing on CPU: PTQ1_0 saves 88 MB (2B) / 197 MB (4B) but prefills 2.4-2.8x slower even after
b10826's 2.5x PTQ1_0 speed-up (b10777 runs it on an SSSE3 path); Q2_0 on the Prism fork falls back to a very slow
path. **Upstream llama.cpp b11912 runs Bonsai Q2_0 correctly but ~40% slower in prefill than Prism PQ2_0 on the same
AVX2 instruction set** (2B 18.0 vs 29.1, 4B 6.5 vs 11.1 tok/s at 2 threads; decode 7-16% slower), so it is only an option if leaving the Prism fork matters more
than speed.

## Flag sweep (llama-bench, b10826 prebuilt, this host)

`microbench/flag-sweep.jsonl` (`F-*` rows), tokens/s, mean of 2 repetitions (± about 2-5%).

Micro-batch (`-ub`, with `-b 2048`), prefill of a 2,048-token prompt:

| Model / CPU variant / threads | ub 256 | **ub 512 (default)** | ub 1024 | ub 2048 |
| --- | ---: | ---: | ---: | ---: |
| 2B, cascadelake (AVX-512), 4 | 92.3 | 95.1 | 91.2 | 92.2 |
| 2B, cascadelake, 2 | | 48.3 | 50.0 | |
| 2B, **haswell (AVX2 only)**, 4 | **66.9** | 52.1 | 47.4 | |
| 4B, cascadelake, 4 | 35.8 | 34.5 | 35.7 | 34.3 |
| 4B, cascadelake, 2 | | 18.5 | 19.5 | |
| 4B, **haswell**, 4 | **25.2** | 20.7 | 17.1 | |

On the AVX-512 variant `-ub` makes no difference; **on the AVX2-only variant, which most laptops load, `-ub 256`
prefills 21-28% faster than the default 512** (and larger batches are slower). `phase2c.sh` checks ub 128 and the
effect end to end at 2 threads (see "AVX2 laptops" below when present). `-ub 1024` end to end on AVX-512 was 5-6%
slower (server table).

KV cache and flash attention (4 threads; prefill 2,048 tokens, decode 32 tokens, at depth 0 and after 1,024 tokens):

| Model | Setting | pp @0 | pp @1024 | tg @0 | tg @1024 |
| --- | --- | ---: | ---: | ---: | ---: |
| 2B | FA on, f16 KV | 88.2 | 85.3 | 18.2 | 19.5 |
| 2B | FA on, q8_0 K+V | 77.7 | 69.1 | 19.3 | 18.4 |
| 2B | FA off, f16 KV | 90.9 | 87.0 | 19.2 | 19.1 |
| 4B | FA on, f16 KV | 34.0 | 33.7 | 9.0 | 8.7 |
| 4B | FA on, q8_0 K+V | 30.8 | 26.7 | 9.1 | 8.1 |
| 4B | FA off, f16 KV | 35.2 | 32.8 | 8.8 | 8.5 |

q8_0 KV costs 10-20% prefill for no decode gain; its memory saving was not measured separately and is bounded by
the KV share of the server's anonymous memory (the hybrid model keeps a KV cache only in its full-attention
layers). Flash attention on/off is within noise; keep the default (`auto`) and f16.

Thread scaling (pp512 / tg32 tokens/s): 2B 1 thread 27.9 / 6.9, 2: ~48-53 / ~11-12.5, 3: 75.8 / 16.8, 4: ~81-95 /
~17-20; 4B 1: 10.6 / 2.9, 3: 26.6 / 7.3, 4: ~34-42 / ~9. Prefill scales nearly linearly to 4 threads; decode
flattens after 3 (memory bandwidth). The app's "half the logical cores" (2 on a 4-thread laptop, 4 on 8) is a sound
background default; the CPU-seconds per document are about the same at 2 or 4 threads.

## AVX2 laptops: micro-batch 256

The AVX2-only CPU variant (`haswell`, forced via `variant-dir.sh`) is what laptops without AVX-512 load. On it the
default `-ub 512` is the slowest choice (`H-*` rows in `microbench/flag-sweep.jsonl`, prefill of 1,024 tokens):

| AVX2 variant | ub 128 | ub 256 | ub 512 (default) |
| --- | ---: | ---: | ---: |
| 2B, 2 threads | 36.4 | 35.6 | 28.6 |
| 4B, 2 threads | 15.1 | 14.2 | 10.9 |
| 4B, 4 threads | 30.1 | 24.1 | 21.9 |
| (AVX-512 `cascadelake` variant, 2B, 2 threads) | 51.4 | 47.4 | 51.8 |

End to end (6 real requests, 2 threads, `--cache-ram 0`, AVX2 variant; `S-*-t2-haswell-*` rows):

| Model | `-ub 512` | `-ub 256` | Replies identical | Peak RSS |
| --- | ---: | ---: | --- | --- |
| 2B | 285.8 s | **245.9 s (-14%)** | 6/6 | 915 -> 901 MiB |
| 4B | 741.5 s | **645.7 s (-13%)** | 5/6 (one near-tie) | 1,881 -> 1,819 MiB |

**Ship `-ub 256`** (128 is as good or better in llama-bench but was not run end to end): 13-14% faster documents on
AVX2-only machines, neutral within noise on AVX-512 machines (2B 92.3 vs 95.1, 4B 35.8 vs 34.5 tok/s at 4 threads),
and it slightly lowers peak memory. Note also that the AVX2 variant's server uses much less memory than the AVX-512
one (2B 0.9 vs 1.4 GB, 4B 1.9 vs 2.95 GB with `--cache-ram 0`): the AVX-512 variants repack weights into extra
buffers. At the app default of 2 threads on the AVX2 path, these 1.2k-2.1k-token documents take ~41 s (2B) and ~108 s (4B)
of model time each with `-ub 256`, vs ~28 s and ~65 s for the same requests on the AVX-512 variant (1.5x / 1.7x).

## Reproduction

Scripts are committed in the worktree `/home/user/intern/.claude/worktrees/agent-a24ebaeab6a00721a`
(branch `worktree-agent-a24ebaeab6a00721a`, not pushed) under `scripts/bonsai-eval/`, plus
`scripts/run-internbench-candidate.sh`, `scripts/build-llama-cpu-linux.sh` and a lab-only opt-in in
`crates/intern-bench/src/live.rs` (`INTERN_BENCH_TOKEN_CONFIDENCE=1`). No production code changed.

```sh
export S=/path/to/scratch          # holds cache/, build/, runtime/, results/bonsai/
# corpus, worker, bench (benchmark-transport for --bench-profile)
npm ci && node bench/generate.mjs  # verify: 77/77 files match bench/manifest.json (Node 22 here, reproduced exactly)
cargo build --release --locked -p intern-worker --features windows-native
cargo build --release --locked -p intern-bench --features benchmark-transport --bins
# runtime: Prism release archive (pinned above), unpacked to $S/build/prebuilt-prism-b10826
# forced CPU variants for laptop emulation
scripts/bonsai-eval/variant-dir.sh $S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc haswell $S/build/prebuilt-prism-b10826-only-haswell
# requests for the server benchmark (from the committed Windows captures)
python3 -I scripts/bonsai-eval/make_requests.py bench/reports/bonsai-native-windows-2026-10-18/bonsai2-synthetic-raw.jsonl $S/work/req2
# everything (resumable; takes the bench lock once per block): InternBench subsets, server bench, matrix, sweeps
scripts/bonsai-eval/pipeline4.sh            # blocks: subsets server matrix noprofile sweep
scripts/bonsai-eval/pipeline4.sh run haswell  # AVX2 micro-batch check (phase2c.sh)
# (pipeline.sh / pipeline2.sh / pipeline3.sh are the earlier stages that ran the full 77-document runs)
# one full live InternBench run, e.g. 2B at 4 threads with token confidence:
LOCK=$S/bench.lock MODEL=$S/cache/bonsai/Ternary-Bonsai-2-2B-PQ2_0.gguf \
  LLAMA_SERVER=$S/build/prebuilt-prism-b10826/llama-prism-b10826-c9c34bc/llama-server \
  INTERN_BENCH=target/release/intern-bench INTERN_RUNTIME_DIR=$S/runtime THREADS=4 SAMPLE_RSS=1 \
  BENCH_PROFILE=bench/reports/bonsai-native-windows-2026-10-18/plans/profile.json INTERN_BENCH_TOKEN_CONFIDENCE=1 \
  scripts/bonsai-eval/internbench-chunked.sh $S/results/bonsai/internbench/2b-t4-app
# analysis
python3 -I scripts/bonsai-eval/analyze-routing.py $S/results/bonsai/routing --b2 .../2b-t4-app/report.json \
  --b4 .../4b-t4-app/report.json --ref qwen_live=.../qwen-t4-app/report.json --swap-s 3
python3 -I scripts/bonsai-eval/routing-readiness-rules.py $S/results/bonsai bench/gold.json
python3 -I scripts/bonsai-eval/compare-runs.py A/report.json B/report.json
python3 -I scripts/bonsai-eval/summarize-server.py $S/results/bonsai/microbench/server.jsonl S-2B-t4-real12=S-2B-t4-real12-cram0
```

Raw data in `$S/results/bonsai/`: `internbench/<run>/{report.json,report.md,recording.json,chunks/*/(wire/,rss.json,llama-server.log,startup.txt,server-command.txt)}`,
`microbench/*.jsonl` (+ per-run stderr and server logs), `routing/`, `runtime/` (commit log review, SHA-256
verification), `host/lscpu-*.txt`, `logs/`.
