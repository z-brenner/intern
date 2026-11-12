# Prompt-lookup decoding and the 4B without repacked weights, November 2026

Status: measured on 2026-11-12 on GitHub-hosted runners. Two questions:
whether llama.cpp's draft-free speculative decoding makes either model
faster, and whether the fast model should run with `--no-repack` as the
accurate one does from alpha.17 (`bench/reports/speed-linux-cpu-2026-11-11`).

**Method.** `speed-battery.yml`: the 13 recorded requests (every second one
of `speed-linux-cpu-2026-10-29/requests`), baseline and variant alternated
over two rounds on one runner, each slot warmed first, the app's flags
(`--parallel 1 --ctx-size 8192 --cache-ram 0 -ub 256 --prio -1 --poll 0
--no-mmproj`, two threads) on upstream llama.cpp `b11946`. For the 9B the
baseline carries `--no-repack` too (`base_args`), so each variant is timed
against the 9B as the app launches it. Raw summaries: `battery.json`.

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

## The 4B with `--no-repack`

| Run | Runner CPU (ggml variant) | Time per request | Change | Prefill | Decode | Replies identical |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 43257828551 | AMD EPYC 7763 (haswell) | 58.1 s -> 53.0 s | **-8.8%** | 563 s -> 508 s | 6.05 -> 6.45 tokens/s | 9/13 |

The four replies that changed differ as replies differ between two
processors: the line a date is quoted from, the order of two parties, the
case of a type, a subject's wording. Both variant rounds gave the same four.
