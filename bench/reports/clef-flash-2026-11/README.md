# Clef-Flash decision screen (Nov 2026)

**Verdict: drop it.** Clef-Flash does not clear the kill line (+10 points of relation
accuracy over the 9B's own reading). It is also about 100 s per document on CPU, on top of
the 9B.

## What was tested

- Model: `ggml-org/Clef-Flash-GGUF` Q4_K_M (6.73 GB, revision c3d92545), served by llama.cpp
  `d35dc4d9` through `/v1/systemone`. Flags: `-nr -c 8192 -b 8192 -ub 8192 -t 4`, on 4-vCPU
  GitHub runners. Weight repacking must be off (`-nr`), or the load aborts in
  `ggml_backend_sched_split_graph`. The most likely cause is the decision head reading rows
  of a quantized embedding that the repack buffer cannot serve.
- Questions per document (one joint request): `date` over every date found in the text,
  `sender` / `recipient` over the 9B's parties plus organisation-looking lines, and
  `relation` over between/from/to/for. Built by `prepare.py` from the recorded 9B readings.
- Sets: InternBench plus real1 to real6 (412 documents). real7 and real8 were not touched.
  410 were answered. Two documents were over the 8,192-token batch and failed.
- Scoring: `score.py` ports intern-bench's party and relation rules and agrees with the
  bench on all 1,190 checks. "Slot" means the 9B's relation word with Clef picking the
  parties.

## Results (same 410 documents, paired)

| | 9B | Clef | Slot (9B word, Clef parties) |
|---|---|---|---|
| Date | 342/395 | 344/395 | – |
| Parties | 312 | 293 | 294 |
| **Relation** | **309 (75.4%)** | **314 (76.6%)** | 303 |
| Full name (parties + relation) | 277 | 271 | 271 |

Real sets only (real1 to real6, 334 docs): relation 233 → 251 (+5.4 pts), name 201 → 213,
dates 269 → 270. InternBench: name 76 → 58. Clef often says `to` where the gold says `from`
(43 cases), and `for` where the gold says `between` (22 cases).

Flips on full name: Clef right and 9B wrong in 51 documents, the reverse in 57.

Latency: median 98 s and p90 276 s per document (median 1,791 input tokens, max 8,144).
The worst case took 11.6 minutes.

## Why not keep pushing

- Even the best slice (real sets only, +5.4 pts relation) is half the bar. The gain on real
  sets is offset by losses on InternBench, so it is not a clean win anywhere.
- A Clef server answers decisions only. It would be a second 6.7 GB model next to the 9B on
  an 8 GB laptop, adding about 1.5 minutes per document.

## Reproduce

The one-off runner workflow (`.github/workflows/clef-flash-screen.yml`) lived on branch
`claude/project-thread-uushfr` at commit `700790a0` and was removed before review, because it
commits to whatever branch it runs on. Restore it on a scratch branch and push `run.txt` to
rerun. Answers land in `answers/`. Then run:

    cat answers/answers-*.jsonl > all.jsonl
    python3 -I score.py requests.jsonl all.jsonl > scored.json
