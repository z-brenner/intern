# The first document after a model load: what was measured, November 2026

Status: measured on 2026-11-09 on GitHub-hosted runners, the first
mechanism check also on one 4-core Linux box. The questions were: how long the first document after a model
load waits for the system turn, how much restoring the server's slot from a
file saves, and whether a restore can change a reply. The answer to the
last one changed the design: only a state the instructions were computed
evenly for is saved (see "Why only an even state is saved").

**Model and runtime.** Qwen3.5-9B Q4_K_M on upstream llama.cpp `b11946`,
the accurate tier as alpha.14 ships it, with the app's flags
(`--parallel 1 --ctx-size 8192 --cache-ram 0 -ub 256 --prio -1 --poll 0`),
four threads, an API key and `--slot-save-path`. On the runners every
server ran on the `haswell` CPU backend alone (`speed-battery.yml`
`cpu_variant`, a hard-linked copy of the release holding one
`libggml-cpu-*.so`): AVX2, the path of Intel 8th-11th generation laptops
without AVX-512 and of AMD Zen 2 and Zen 3. The requests are the recorded
ones under
[`../speed-linux-cpu-2026-10-29/requests`](../speed-linux-cpu-2026-10-29/requests),
whose system turn is 477 tokens.

The sequence (`scripts/bonsai-eval/cold-start.py`): a fresh server reads
one request and its slot is saved; then, twice, a fresh server reads request
`0015` unwarmed, and a fresh server restores the file and reads `0015`; in
the first round the unrestored server also reads `0000` and `0015` again,
the steady state every later document of a lifetime is in. Raw summaries
and replies: `cold-start.json`.

## On the runners

Run 42584078974 (two jobs: an AMD EPYC 9V45 and an AMD EPYC 9V74, 2 cores /
4 threads each, both AVX-512 CPUs held to the `haswell` backend, which the
server confirmed loading), slot saved as the app saves it: after request
`0016` (1,094 tokens, processed from nothing) and the instructions-only
request (477 tokens cached, 15 processed, 1.1-1.5 s):

| First request on a fresh server | `cache_n` | Prompt tokens processed | Prompt time, 9V45 / 9V74 | Whole request, 9V45 / 9V74 | Reply equals the steady state |
| --- | ---: | ---: | ---: | ---: | ---: |
| No restore | 0 | 930 | 39.3 s / 54.0 s | 50.6 s / 68.5 s | 2/2, 2/2 |
| Slot restored | 477 | 453 | 19.3 s / 26.5 s | 30.1 s / 41.1 s | **2/2, 2/2** |
| Steady state (third request of a lifetime) | 477 | 453 | 19.5 s / 26.5 s | - | - |

- **The first document after a load is 19.9 s and 27.4 s sooner**: prompt
  time without the restore, less prompt time with it, less the restore
  (44 ms and 59 ms). The whole first request falls by 40% on both
  (50.6 to 30.1 s, 68.5 to 41.1 s), and is then as fast as any later
  document.
- **The restored reply is the steady-state reply**, byte for byte, in all
  four restored servers, and so is the reply with nothing restored.
- The file is 174,210,604 bytes (492 tokens: the instructions and the
  instructions-only request, and their context checkpoints), saved in
  35-38 ms. Server start to healthy: 3.1 s and 4.2 s.

The first run, below, measured the same saving and found the flaw the
second run's procedure removes.


Run 42575519449 (two jobs, both AMD EPYC 7763, 2 cores / 4 threads), slot
saved after request `0000` (637 tokens) with no instructions-only request:

| First request on a fresh server | `cache_n` | Prompt tokens processed | Prompt time | Whole request | Reply equals the steady state |
| --- | ---: | ---: | ---: | ---: | ---: |
| No restore | 0 | 930 | 64.7 s, 64.7 s | 81.8 s, 80.1 s | 2/2, 2/2 |
| Slot restored | 477 | 453 | 31.8 s, 31.8 s | 49.2 s, 47.6 s | **0/2, 0/2** |
| Steady state (third request of a lifetime) | 477 | 453 | 31.9 s, 31.9 s | - | - |

- The restore saves 32.8 s of the first document's 80-82 s on this CPU:
  the prompt time halves, the restore itself takes 100-106 ms, and the
  server still takes 4.9-5.1 s from start to healthy.
- The file was 234,281,292 bytes (717 tokens and three context checkpoints
  of 50.3 MiB each), saved in 63-64 ms.
- But the restored replies were not the steady-state reply. The date role
  changed from `execution` to `issuance` and the subject from "Ripstop
  nylon and textile fabrics" to "purchase order for fabric and zippers".

## Why only an even state is saved

The server logs show why. llama-server processes a prompt it starts from
nothing in micro-batches, and breaks the batch where the last user turn
starts and 4 and 4 + `-ub` (260) tokens before the end. The saving server
read a 637-token request, so it broke at 377, inside the 477-token system
turn, and processed the instructions as 0-376 and 377-476; the unrestored
servers read a 930-token request and processed them as 0-476 in one
piece. The two states differ in the last bits, which is enough to move a
greedy reply. A server keeps for its whole lifetime the instructions its
first request computed, so this was already true before slot files: the
replies of a lifetime depended on how long its first document was.

So the app (`SlotKeeper::save_once`) saves only when the document that
triggers the save was the lifetime's first request and was at least the
instructions plus 260 tokens long, and the restored state is then exactly
the one such a document computes. That is not the state the live accuracy
runs were recorded in: `intern-bench` starts every live run with its
warm-up document, the notice that is request `0000` here (637 tokens), so
every recorded reply of InternBench and the real-document sets came from
instructions computed in pieces. A restored app computes them in one
piece, as an app already did in any lifetime whose first document was
long; on the purchase order above the two states give different replies.
No live accuracy run had been made in the even state when this was measured;
one was made later the same day, with the longer warm-up (the candidate
benchmark's run 42625355879 at `a29e446`; docs/architecture.md, "The
instructions across a restart"). Most documents are that long
(InternBench's median prompt is 1,347 tokens). Before saving, it sends the
instructions-only request (the same turns, then "JSON only.", one token of
reply), so the file holds the instructions and none of the document.
`cold-start.py` now does the same and saves after request `0016`
(1,094 tokens).

## On a runner, through the app's own code

`live_the_first_document_after_a_restart_finds_the_instructions_restored`
(`src-tauri/src/commands.rs`) drives the real `RuntimeModel` and
`LlamaLauncher` through three lifetimes of the real server: the speed
battery's `live_slots` job, run 42613236166 at d24fc11, the pinned
Qwen3.5-4B and the pinned `b11946` server on an AMD EPYC 9V74 runner (4
logical processors, the `zen4` backend), four threads as the app chooses
them. What the test printed is in [`live-slots.txt`](live-slots.txt):

- The first launch reads the self-test's probes first: nothing is saved.
- The second launch's first request is a 928-token agreement, processed
  from nothing in 20.8 s; the instructions are saved: 174,210,604 bytes,
  492 tokens (the instructions-only request's 477 cached and 15
  processed), in 36 ms.
- The third launch restores the file in 60 ms (3.0 s from launch to
  ready) and reads the agreement with 477 tokens cached and 451 processed
  in 10.3 s; read again after another document, 477 cached, 10.4 s.
- The agreement's reply - proposal, file name and description - is the
  same in all three readings: first with nothing restored, first after the
  restore, and later in the lifetime. And the restored lifetime's first
  reading of the agreement found only the instructions cached, not the
  agreement the file was saved after.

An earlier run of the same test on the shared 4-core box gave the same
outcome (30.1 s against 11.6 s); its log was not kept, and review could
not reproduce it on that box under load from other work, so it is not
counted here.

The first measurement of the mechanism, with `cold-start.py` and the 2B on
this box's AMX build (the slot saved after `0000`, before the even-state
rule), gave `cache_n` 0 and 0 without the restore and 477 and 477 with it,
an 89,749,464-byte file restored in 68 ms, and replies equal to the steady
state either way: the 2B's reply did not move where the 9B's did.

## The battery on the laptop path

The same jobs ran the battery on `haswell` (the app's flags against
themselves, alternated over two rounds), replies identical to the first
baseline in every run:

| Run | Runner CPU | Requests | Per request | Prompt p50 | Decode p50 |
| --- | --- | ---: | ---: | ---: | ---: |
| 42575519449 | AMD EPYC 7763 | every 3rd (9) | 93.5-94.8 s | 60.0-61.8 s | 4.9-5.1 tokens/s |
| 42575519449 | AMD EPYC 7763 | every 3rd (9) | 92.8-93.1 s | 59.9-60.3 s | 5.4 tokens/s |
| 42584078974 | AMD EPYC 9V45 | every 9th (3) | 57.1-59.1 s | 34.7-35.5 s | 7.5-7.9 tokens/s |
| 42584078974 | AMD EPYC 9V74 | every 9th (3) | 78.0-78.3 s | 47.5 s | 5.8-6.0 tokens/s |

An EPYC 7763 is Zen 3, whose own best backend is `haswell` anyway; the
9V45 and 9V74 have AVX-512 and would load `zen4` without the pin, and the
recorded `load_backend` line shows `libggml-cpu-haswell.so` on both. These
rows used other request subsets than earlier reports and are not
comparable with them; a same-runner comparison of `haswell` against the
runner's own backend needs two jobs on one CPU model, one with
`cpu_variant` and one without.
