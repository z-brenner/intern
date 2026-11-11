# The 9B without repacked weights: what was measured, November 2026

Status: measured on 2026-11-11 on GitHub-hosted runners. The question was
whether the accurate model should run with llama.cpp's `--no-repack`. It
does from alpha.17 (`tiers::tier_server_flags`); the fast model was not
measured with it and keeps the default.

**Model and runtime.** Qwen3.5-9B Q4_K_M on upstream llama.cpp `b11946`,
the accurate tier as alpha.16 ships it, four threads, the app's flags
(`--parallel 1 --ctx-size 8192 --cache-ram 0 -ub 256 --prio -1 --poll 0
--no-mmproj`), with `--no-repack` added for the variant.

## What the flag does

On x86, llama.cpp copies every Q4_K weight matrix into an interleaved
layout at load, for its older 8x8 GEMM (the "repack" buffer). For the 9B
that copy is about 2.6 GiB of the server's private memory, on top of the
mapped file. With `--no-repack` the weights are read where the file maps
them, and prompt processing goes through the tiled k-quant kernel b11946
already contains (ggml-org/llama.cpp#27851), which skips repacked weights.
No x86 kernel changed between b11946 and b12283, so a newer runtime would
not change this.

## Speed: one job per runner, its own baseline

`speed-battery.yml`: the 13 recorded requests (every second one of
`speed-linux-cpu-2026-10-29/requests`), baseline and variant alternated
over two rounds on the same runner, each slot warmed first. Raw summaries:
`battery.json`.

| Run | Runner CPU | ggml backend | Time per request | Change | Replies identical |
| --- | --- | --- | ---: | ---: | ---: |
| 42975883453 | AMD EPYC 9V74 (Zen 4) | its own (AVX-512) | 62.2 s -> 45.8 s | **-26%** | 4/13 |
| 42975883453 | AMD EPYC 9V74 (Zen 4) | its own (AVX-512) | 63.1 s -> 46.0 s | **-27%** | 4/13 |
| 42975883453 | AMD EPYC 9V45 (Zen 5) | its own (AVX-512) | 38.7 s -> 30.1 s | **-22%** | 4/13 |
| 42975883453 | Intel Xeon Platinum 8573C | its own (AMX) | 58.1 s -> 58.0 s | 0% | 4/13 |
| 43048574452 | AMD EPYC 9V45 | haswell (AVX2), pinned | 61.2 s -> 51.9 s | **-15%** | 3/13 |
| 43048574452 | AMD EPYC 9V74 | haswell (AVX2), pinned | 78.9 s -> 70.7 s | **-10%** | 3/13 |
| 43048574452 | AMD EPYC 9V74 | haswell (AVX2), pinned | 80.1 s -> 71.8 s | **-10%** | 3/13 |

- The gain is in prompt processing (+51-55% tokens a second on AVX-512,
  +15-27% on AVX2); decoding moved -4% to +13%.
- AVX2 is the code path laptops without AVX-512 load. A job pinned to
  `alderlake` (AVX2 with AVX-VNNI, Intel 12th generation and later) could
  not start on its runner, whose CPU has no AVX-VNNI. b11946 has no
  AVX-VNNI kernel for Q4_K on either path, so those laptops are expected to
  behave as the AVX2 rows do; that is an inference, not a measurement.
- Replies differ from the repacked run's, as they do between two processors
  (the order of floating-point sums changes). What that does to names is
  measured below.

## Memory

The server's peak resident memory (`VmHWM`, which Linux reports in KiB)
over each whole live set, from `candidate-bench.yml` runs 42973633153
(repacked) and 42983156560 (`--no-repack`), both on InternBench and the
five real sets (`battery.json`, `server_peak_rss_kb`):

| | Repacked | `--no-repack` |
| --- | ---: | ---: |
| Peak resident memory, every set | 8,750-8,800 MiB | 6,133-6,184 MiB |

2,616 MiB (2.7 GB) less, about 30%. The model file is 5,417 MiB, so
without repacking all but about 0.7 GiB of the resident set is the mapped
file. The server now reads all of it but the token embeddings, which are
read a row at a time, on every token: on Windows (below) about 4,880 MiB of
the file stayed in its working set, the rest about the size of the
embeddings. `VmHWM` is a high-water mark of whatever was ever resident, so
it does not show what is in use.

On Windows (`runtime-speed.yml` with `probe: memory`, run 43063958237, a
16 GiB runner, four threads), the 9B's working set right after a document
was 5,490-5,491 MiB, and 5,229-5,491 MiB from load to a minute idle, against
8,107-8,110 MiB right after a document with repacked weights
(`bench/reports/windows-memory-2026-11-09`). The machine had 8,208 MiB
available with it loaded and nothing else running (5,410-5,570 before),
and 5,939 MiB with another process holding 2 GiB (3,535 before). With
that 2 GiB held, the tier plan's figure against the memory available
before each load came out 141 MiB under with the 9B right after a
document and 1,453 MiB over with the 4B a minute idle: a 1,594 MiB gap,
wider than the 1,024 MiB `tiers::UPGRADE_MARGIN` then asked of a loaded
4B, which would have swapped the models at every pause on a machine with
about 7,763 to 8,333 MiB available before any load. The margin is now
2,048 MiB; see its comment and the note in
`bench/reports/windows-memory-2026-11-09`.

The memory floors in `tiers.rs` are left as they are.
`MIN_AVAILABLE_FOR_ACCURATE` counts what the 9B must keep resident to read
at its speed, measured with repacked weights at a 6 GB memory limit: 6,043
MiB, 3,317 MiB of anonymous memory and 2,726 MiB of the file it kept
reading; the rest of the file was the original of the copy, read once
(`speed-linux-cpu-2026-11-05`, which gives it as 6.2 GB, the KiB count read
as GB). Without the copy it reads those originals in its place, all of the
file but the token embeddings. Its working set on Windows peaked at about
5,525 MiB, below that figure, and its peak resident set on Linux,
6,133-6,184 MiB, is about the same; but what it needs without the copy was
not measured under a limit, so the floors are kept rather than lowered, and
may now carry more slack than before. One bar did depend on the old
layout: on Linux, where `MemAvailable` counts mapped weights as
reclaimable, a second OCR page is read at once only with the weights the
9B keeps reading counted on top (`paddle::pages_at_once`). Those are now
counted as the whole 5.68 GB file, an upper bound, not 2.7 GB, and the bar
rises from 5 to 8 GiB; counting only the 4,880 MiB in its Windows working
set would give 8 GiB too. The fallback estimate of what unloading the 9B
gives back (`tiers::resident_estimate`, used only when the live server
cannot be read) is kept at its repacked figure; see its comment.

## Names

The same two live runs, replayed through the alpha.17 engine at the 0.7
gate, on the documents both completed, against two repacked runs on
different runners (2026-11-09 and 2026-11-11) for scale:

| Pair | Documents | Names right | Filed | Filed wrong | Same filename |
| --- | ---: | ---: | ---: | ---: | ---: |
| repacked 11-09 vs repacked 11-11 | 329 | 201 vs 201 | 57 vs 59 | 5 vs 5 (2 and 2 differ) | 288 |
| repacked 11-11 vs `--no-repack` | 335 | 202 vs 200 | 59 vs 60 | 5 vs 7 (0 and 2 differ) | 301 |

The no-repack run's filenames are closer to the 11-11 repacked run's than
the two repacked runs' are to each other. Its two extra wrong filings
(`property-policy-declarations-mid-60p`, `real3-notice-15`) are names whose
least probable token crossed the 0.7 gate, the kind of change any other
processor makes. Within that drift, no difference in accuracy was found.

## Not measured

- The fast model with `--no-repack`.
- Windows: the speed with `--no-repack`. `runtime-speed.yml` run
  43063962365, on a 4-logical-processor AMD EPYC 7763 with two server
  threads, timed the 9B at 88-90 s a request and the 4B at 50 s, but that
  workflow launches the server without `--no-repack`, so the run timed the
  repacked launch; the Windows speed with `--no-repack` was not measured.
- An Intel laptop with AVX-VNNI (above).
