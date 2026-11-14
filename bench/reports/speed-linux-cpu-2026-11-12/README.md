# Prompt-lookup decoding and the 4B without repacked weights, November 2026

Status: measured on 2026-11-12 on GitHub-hosted runners. Two questions:
whether llama.cpp's draft-free speculative decoding makes either model
faster (it does not), and whether the fast model should run with
`--no-repack` as the accurate one does from alpha.17
(`bench/reports/speed-linux-cpu-2026-11-11`); it does from alpha.18.

**Method.** `speed-battery.yml`: the 13 recorded requests (every second one
of `speed-linux-cpu-2026-10-29/requests`), baseline and variant alternated
over two rounds on one runner, each slot warmed first, the app's flags
(`--parallel 1 --ctx-size 8192 --cache-ram 0 -ub 256 --prio -1 --poll 0
--no-mmproj`) on upstream llama.cpp `b11946`, with two threads where the
app would take four on these four-processor runners (every logical
processor, at most eight). For the 9B the baseline carries `--no-repack`
too (`base_args`), so each variant is timed against the 9B's flags as the
app launches it. Raw summaries: `battery.json`.

## Prompt-lookup speculative decoding: rejected

b11946's server can draft tokens without a second model, from n-grams of
the prompt and the reply so far (`--spec-type ngram-*`), and check the
drafts in one batch. A reply here copies names and dates from the prompt,
so it looked like a fit.

| Run | Model | Runner CPU (ggml variant) | Variant | Time per request | Change | Drafts accepted | Replies identical |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| 43257823167 | 9B | AMD EPYC 7763 (haswell) | `ngram-simple` | 92.8 s -> 94.1 s | +1.4% | 9 of 192 | 13/13 |
| 43257823167 | 9B | AMD EPYC 9V74 (haswell) | `ngram-map-k` | 90.8 s -> 91.2 s | +0.4% | 2 of 48 | 13/13 |
| 43257823167 | 9B | Intel Xeon Platinum 8370C (icelake) | `ngram-mod` | 73.6 s -> 77.5 s | +5.3% | 0 of 704 | 7/13 |
| 43257828551 | 4B | AMD EPYC 9V74 (zen4) | `ngram-simple` | 34.0 s -> 34.6 s | +1.9% | 3 of 144 | 13/13 |

Times are the means of the two rounds; drafts are counted over the first
variant round. Every variant was slower. The reply is a short JSON object
whose values are quoted with the ids of the lines they came from
(`"p1.b4.f2"`), and the drafter almost never guesses the next tokens: 0 to
5% of drafted tokens were accepted, so the batch that checks them is spent
for nothing. `ngram-mod`, which drafts from a rolling hash of the whole
context, also changed six of 13 replies, as batched verification can on a
near tie. Nothing here suggests a draft model would do better on replies
this short (about 90 tokens), and a draft model would need its own memory.

## The 4B with `--no-repack`: adopted

The fast model runs with it from alpha.18 (`tiers::tier_server_flags`), as
the accurate one has since alpha.17. On x86 llama.cpp otherwise copies every
Q4_K weight matrix into an interleaved layout at load; without the copy the
weights are read where the file maps them.

### Speed

| Run | Runner CPU (ggml variant) | Time per request | Change | Prefill | Decode | Replies identical |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 43257828551, Linux | AMD EPYC 7763 (haswell) | 58.1 s -> 53.0 s | **-8.8%** | 563 s -> 508 s | 6.05 -> 6.45 tokens/s | 9/13 |
| 43274013741, Windows | AMD EPYC 7763 | 53.9 s -> 51.7 s | **-4.2%** | 32.0 s -> 30.6 s (median) | 7.13 -> 7.08 tokens/s (median) | - |

The Linux row is `speed-battery.yml` as above; the Windows row is
`runtime-speed.yml` with `probe: no-repack-fast`, the 4B launched with the
app's flags but two server threads where the app would take four,
alternately without and with the flag, twice each, on every third recorded
request, on one runner with four logical processors
(`windows-fast-no-repack.json`). The four replies that changed on
Linux differ as replies differ between two processors: the line a date is
quoted from, the order of two parties, the case of a type, a subject's
wording; both variant rounds gave the same four.

### Memory

The server's peak resident memory (`VmHWM`) over each whole live set, from
`candidate-bench.yml` runs 43273805585 (repacked) and 43273798577
(`--no-repack`), InternBench and the real sets 1-5, four threads
(`fast-no-repack-live.json`):

| | Repacked | `--no-repack` |
| --- | ---: | ---: |
| Peak resident memory, every set | 4,558-4,613 MiB | 3,260-3,315 MiB |

About 1,300 MiB (1.4 GB) less, 28%. On Windows (`runtime-speed.yml` with
`probe: memory`, run 43273956181, both models launched as alpha.18 launches
them, `windows-memory-both-no-repack.json`), the 4B's working set right
after a document was 3,215 MiB, against 4,513-4,514 MiB with repacked
weights (`bench/reports/windows-memory-2026-11-09`), and the 9B's 5,491 as
before. This is the model a machine with less than 8 GiB available falls
back to, so the saving lands where memory is shortest.

The tier plan's arithmetic was checked against the new figures. With
another process holding 2 GiB, its figure against the memory available
before each load came out 11 MiB under with the 9B right after a document,
and 400 and 743 MiB over with the 4B right after a document and a minute
idle: the 4B's is the higher by 411 and 754 MiB, inside
`tiers::UPGRADE_MARGIN` (2,048 MiB) and inside the 1,024 MiB of alpha.16
too, so a loaded 4B does not give way to the 9B only to be stood down again
(the test `a_loaded_fast_model_makes_way_only_with_room_to_spare` carries
the pass). The margin is kept. The memory floors are kept: the 4B's floor
(`MIN_AVAILABLE_FOR_FAST`) decides nothing yet, and the fallback estimate of
what unloading the 4B gives back stands in only when the live server cannot
be read (see `tiers::resident_estimate`).

### Names

The two live runs and an earlier repacked 4B recording (InternBench and the
real sets 1-4), replayed through one engine (alpha.17's, which alpha.18's
flags do not change), on the documents both runs of a pair completed and
the gold names:

| Pair | Documents | Names right | Same filename |
| --- | ---: | ---: | ---: |
| repacked (earlier) vs repacked (11-12) | 291 | 165 vs 159 | 263 |
| repacked (11-12) vs `--no-repack` | 291 | 159 vs 165 | 253 |
| repacked (earlier) vs `--no-repack` | 287 | 165 vs 165 | 254 |
| real5: repacked (11-12) vs `--no-repack` | 53 | 13 vs 16 | 44 |

Two repacked runs on different runners differed by six right names; the
no-repack run gave as many right names as the better of them. Its filenames
differ from a repacked run's somewhat more often than two repacked runs'
do (253 and 254 of about 290 the same, against 263), so the flag moves
replies about as far as a change of processor does, or a little further,
and no loss of right names came with it. The fast tier's readings are held
for review whatever their confidence, so a name it gives is a suggestion a
person confirms.

## Not measured

- The 4B on AVX-512 runners with the battery (one AVX2 runner on Linux, one
  on Windows; the 9B gained most on AVX-512).
- A draft-model speculative setup.
- An Intel laptop with AVX-VNNI or a hybrid core layout.
