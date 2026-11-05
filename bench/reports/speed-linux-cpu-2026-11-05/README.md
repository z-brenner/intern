# The 9B's threads and memory floor: what was measured, November 2026

Status: measured on 2026-11-05 on GitHub-hosted runners and on one 4-core
Linux box. The questions were: which `llama-server` settings make the 9B
faster on a hyper-threaded machine, and how little memory it can be given
before it stops reading at its speed. Two settings change from this:
the model's threads, and the memory the 9B needs free to load.

**Model and runtime.** Qwen3.5-9B Q4_K_M on upstream llama.cpp `b11946`,
the accurate tier as alpha.13 ships it, with the app's flags
(`--parallel 1 --ctx-size 8192 --cache-ram 0 -ub 256 --prio -1 --poll 0`).
The requests are the 26 recorded under
[`../speed-linux-cpu-2026-10-29/requests`](../speed-linux-cpu-2026-10-29/requests),
every second one (13), each slot warmed first, as
`.github/workflows/speed-battery.yml` runs them.

## Server settings, one against the app's flags per runner

Each row is one job on one runner: the app's flags (two threads, the
runner's rule), the variant, the app's again, the variant again. The two
baselines and two variants are averaged; the ratio within a row is the
measurement, because every job had a different CPU (two cores, four
threads, on each). Raw summaries: `battery.json`.

| Variant | Runner CPU | Time per request | Prompt processing, p50 | Decode, p50 | Replies identical to the baseline |
| --- | --- | ---: | ---: | ---: | ---: |
| `--threads-batch 4` | AMD EPYC 7763 | 97.8 s → 92.2 s (−5.7%) | 64.4 s → 60.0 s | 4.33 → 4.35 tok/s | 13/13 |
| `--threads 3 --threads-batch 4` | AMD EPYC 9V45 | 43.7 s → 36.7 s (−16.0%) | 24.7 s → 19.7 s | 6.50 → 7.08 tok/s | 8/13 |
| **`--threads 4 --threads-batch 4`** | Intel Xeon Platinum 8573C | **82.3 s → 67.8 s (−17.6%)** | 41.3 s → 37.4 s | **2.81 → 4.03 tok/s** | 6/13 |
| `-ub 512` | AMD EPYC 9V45 | 42.1 s → 41.7 s (−0.8%) | 24.1 s → 24.1 s | 6.92 → 7.11 tok/s | 6/13 |
| `--flash-attn off` | AMD EPYC 9V74 | 60.3 s → 60.5 s (+0.2%) | 36.7 s → 37.6 s | 5.66 → 5.88 tok/s | 6/13 |
| `-ctk q8_0 -ctv q8_0` | AMD EPYC 7763 | 99.6 s → 104.5 s (+4.9%) | 64.6 s → 68.4 s | 4.00 → 4.14 tok/s | 1/13 |
| `-lm none` (no memory mapping) | AMD EPYC 7763 | 103.0 s → 103.3 s (+0.3%) | 64.8 s → 64.8 s | 3.51 → 3.51 tok/s | 13/13 |

What it means:

- **Every logical processor beats half of them.** On a two-core,
  four-thread runner, four threads against two cut the time per request by
  16-18%. Prompt processing, which is compute-bound, gains 9-19%; on the
  Intel runner decoding, which is memory-bound, gains 43% as well. Batch
  threads alone (`--threads-batch 4`, decoding still on two) give only the
  prompt-processing part, 6%. So the app's rule changes from half the
  logical count to all of it, at most eight (`server.rs` `default_threads`).
  The gain was measured on the SMT siblings of a few cores; a hybrid chip's
  efficiency cores were not measured, which is why eight is the ceiling.
- **The other flags stay.** `-ub 512` and flash attention off are within
  noise, as they were for the 2B, and so is loading the weights without
  memory mapping once they are in the page cache (`--no-mmap` is not a
  flag `b11946` takes; `-lm none` is). A q8_0 KV cache is 5% slower and
  changes 12 of 13 replies: rejected.
- **Replies drift with the thread count.** Four threads against two changed
  5-7 of 13 replies at temperature 0, as a different CPU does: the thread
  count moves the arithmetic. Batch threads alone changed none. The
  accuracy of the shipped configuration is measured by the live runs, not
  here.

## The memory floor

The 9B under a memory cgroup limit on one 4-core Xeon (AVX-512, no SMT),
four threads, five requests per limit, the page cache dropped before each.
The limit counts the server's anonymous memory and the mapped weights it
keeps resident; the kernel reclaims the mapped pages first, which is what a
laptop's "available" memory does under pressure. Raw summaries:
`memory-floor.json`.

| Limit | Time per request, p50 | Prompt processing, p50 | Decode | Weights resident |
| --- | ---: | ---: | ---: | ---: |
| none | 71.7 s | 33.3 s | 3.14 tok/s | 5.3 GB |
| 9 GB | 79.1 s | 46.9 s | 3.29 tok/s | 5.3 GB |
| 8 GB | 104.2 s | 43.5 s | 2.70 tok/s | 4.7 GB |
| 7 GB | 94.6 s | 42.1 s | 2.85 tok/s | 3.7 GB |
| 6 GB | 95.5 s | 38.3 s | 2.62 tok/s | 2.7 GB |
| 5 GB | 421.3 s, 5 of 13 requests failed | 38.7 s | 0.25 tok/s | 1.7 GB |

What it means:

- **The 9B's working set is about 6.2 GB**: 3.4 GB of anonymous memory
  (the repacked weights, the context, the compute buffers) and 2.7 GB of
  mapped weights it reads on every token. The rest of the 5.3 GB mapped
  file is the original of what was repacked, read once at load and never
  again; the system reclaims it without cost.
- **Below 6 GB it thrashes.** At 5 GB decoding fell to a quarter of a token
  a second and requests timed out. Between 6 and 8 GB it reads 20-30%
  slower than with no limit, as the pages it does need come and go.
- **The floor comes down from 10 GiB to 8 GiB** (`tiers.rs`
  `MIN_AVAILABLE_FOR_ACCURATE`): the working set, the worker's 0.7 GB, and
  1 GB of headroom. A 16 GB laptop with a browser open now reads with the
  9B where it read with the 4B before. The resident estimate stays at the
  measured high-water mark, 8.3 GB, because that is what unloading gives
  back; the floor is about what must stay, not what is touched.
- **Caveat.** The baseline and the 9 GB rows ran while the box was also
  building Rust, so their times are noisier than the rest; the ordering
  holds. The behaviour under a Windows memory squeeze, where the standby
  list plays the page cache's part, was not measured.

## Also measured

- **Q4_K_S against Q4_K_M** (the 9B, replayed with the current engine from
  the recordings in `candidate-bench.yml` run 41730932267): InternBench 67
  against 70 names right, 3 against 1 wrong names ready; the first real
  set 19 against 20, 3 against 2. On the same runner CPU it was no faster
  (p50 83.1 s against 80.1 s). Q4_K_M stays.
- **A newer llama.cpp.** Between `b11946` and `b12128` no x86 CPU kernel
  changed (`ggml/src/ggml-cpu/arch/x86/repack.cpp` is byte-identical); the
  AVX-VNNI kernels that would speed prompt processing by 45-85% are still
  open pull requests (ggml-org/llama.cpp#29397, #30036). Nothing to take.
