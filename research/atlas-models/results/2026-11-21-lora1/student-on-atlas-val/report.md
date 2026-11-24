# InternBench: live run

- **Run:** 2026-11-21T09:35:24Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `99e3ad6efd92`
- **Corpus:** 200 documents · 200 completed · gold `978c25f73e`
- **Wall time:** 883.60 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 197/200 (98.5%) |
| Document type | 200/200 (100.0%) |
| Date | 200/200 (100.0%) |
| Date role (when the reviewed date was chosen) | 127/200 (63.5%) |
| Parties | 199/200 (99.5%) |
| Relation word (when parties were named) | 198/200 (99.0%) |
| Party in the right role (when parties were named) | 134/137 (97.8%) |
| Ready / review routing | 183/200 (91.5%) |
| Description states nothing false | 200/200 (100.0%) |
| Description is specific | 194/200 (97.0%) |
| Description specificity | 86.0% (mean of 200) |
| Evidence recall (model's quotes) | 99.2% (mean of 200) |
| Digest recall (facts reaching the digest) | 100.0% (mean of 200) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 200) |
| Sent to review | 8.5% of 200 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | – | 0 |
| Complex-document filename accuracy | 98.5% | 200 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 2.5% | 200 |
| Review rate | 8.5% | 200 |
| Evidence recall | 99.2% | 200 |
| Total latency p50 | 4.33 s | 200 |
| Total latency p95 | 5.40 s | 200 |
| Generation latency p50 | 1.87 s | 200 |
| Generation latency p95 | 2.68 s | 200 |
| Generated tokens p50 | 71 | 200 |
| Generated tokens p95 | 102 | 200 |
| Prompt tokens p50 | 368 | 200 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 200 |
| Trap date chosen | 0 of 200 |
| Forbidden party named | 1 of 200 |
| Spurious parties named | 1 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 704 (0.0%) |
| Right name sent to review anyway | 14 of 200 |
| Sent to review for an unsupported fact | 5 of 200 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 37 | 37/37 | 37/37 | 37/37 | 30/37 | 18.9% | 4.61 s | 5.34 s |
| letter | 79 | 78/79 | 79/79 | 78/79 | 74/79 | 6.3% | 4.87 s | 5.66 s |
| notice | 61 | 61/61 | 61/61 | 61/61 | 61/61 | 0.0% | 3.71 s | 4.12 s |
| receipt | 11 | 9/11 | 11/11 | 11/11 | 8/11 | 27.3% | 3.27 s | 3.41 s |
| statement | 12 | 12/12 | 12/12 | 12/12 | 10/12 | 16.7% | 3.80 s | 4.39 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 120 | 120/120 | 120/120 | 120/120 | 109/120 | 9.2% | 4.39 s | 5.46 s |
| office | 39 | 37/39 | 39/39 | 38/39 | 34/39 | 12.8% | 4.56 s | 5.30 s |
| text | 41 | 40/41 | 41/41 | 41/41 | 40/41 | 2.4% | 4.12 s | 5.35 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 200 | 197/200 | 200/200 | 199/200 | 183/200 | 8.5% | 4.33 s | 5.40 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| complex | 200 | 197/200 | 200/200 | 199/200 | 183/200 | 0 of 200 | 4.33 s | 5.40 s | 1.87 s | 2.68 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| competing_dates | 200 | 197/200 | 200/200 | 199/200 | 1 |
| date_in_table | 12 | 12/12 | 12/12 | 12/12 | 0 |
| docx | 39 | 37/39 | 39/39 | 38/39 | 1 |
| form | 48 | 46/48 | 48/48 | 48/48 | 0 |
| invoice | 37 | 37/37 | 37/37 | 37/37 | 0 |
| irrelevant_names | 79 | 78/79 | 79/79 | 78/79 | 1 |
| key_value | 85 | 83/85 | 85/85 | 85/85 | 0 |
| layout_parties | 24 | 24/24 | 24/24 | 24/24 | 0 |
| letter | 79 | 78/79 | 79/79 | 78/79 | 1 |
| notice | 61 | 61/61 | 61/61 | 61/61 | 0 |
| statement | 12 | 12/12 | 12/12 | 12/12 | 0 |
| table | 49 | 49/49 | 49/49 | 49/49 | 0 |

## Routes

- **Pages:** fast 154 · layout 46
- **Documents by route class:** fast 154 · layout 46

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 200 | 4.33 s | 5.40 s | 5.87 s |
| `extraction_wall_ms` | 200 | 2.5 ms | 3.5 ms | 943.9 ms |
| `worker_total_ms` | 200 | 2.1 ms | 3.0 ms | 943.5 ms |
| `worker_snapshot_ms` | 200 | 0.20 ms | 0.25 ms | 0.28 ms |
| `worker_parse_ms` | 200 | 1.5 ms | 2.3 ms | 942.8 ms |
| `worker_analysis_ms` | 200 | 0.11 ms | 0.36 ms | 1.4 ms |
| `worker_render_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_image_decode_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_encode_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_engine_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_vision_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `analyze_wall_ms` | 200 | 4.33 s | 5.37 s | 5.72 s |
| `distill_ms` | 200 | 1.0 ms | 1.5 ms | 1.9 ms |
| `prompt_ms` | 200 | 0.02 ms | 0.03 ms | 0.05 ms |
| `inference_ms` | 200 | 4.32 s | 5.37 s | 5.72 s |
| `prefill_ms` | 200 | 2.27 s | 3.16 s | 3.55 s |
| `generation_ms` | 200 | 1.87 s | 2.68 s | 2.92 s |
| `validation_ms` | 200 | 3.0 ms | 4.1 ms | 6.0 ms |
| `naming_ms` | 200 | 0.14 ms | 0.21 ms | 0.28 ms |
| `index_ms` | 200 | 0.79 ms | 1.2 ms | 2.4 ms |
| `retrieval_ms` | 200 | 0.01 ms | 0.02 ms | 0.04 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 200 | 162.8 tok/s | 168.3 tok/s | 172.2 tok/s |
| `generation_tok_per_s` | 200 | 37.9 tok/s | 38.6 tok/s | 39.2 tok/s |
| `prompt_tokens` | 200 | 368 | 526 | 584 |
| `cached_tokens` | 200 | 477 | 486 | 486 |
| `generated_tokens` | 200 | 71 | 102 | 110 |
| `estimated_prompt_tokens` | 200 | 922 | 1060 | 1109 |
| `model_requests` | 200 | 1 | 1 | 1 |
| `redistillations` | 200 | 0 | 0 | 0 |
| `source_characters` | 200 | 795 | 1243 | 1320 |
| `digest_characters` | 200 | 900 | 1380 | 1495 |
| `prompt_characters` | 200 | 2949 | 3429 | 3544 |
| `worker_ocr_pages` | 200 | 0 | 0 | 0 |
| `worker_ocr_passes` | 200 | 0 | 0 | 0 |
| `worker_orientation_passes` | 200 | 0 | 0 | 0 |
| `worker_rendered_pixels` | 200 | 0 | 0 | 0 |
| `index_units` | 200 | 12 | 23 | 25 |
| `context_units` | 200 | 12 | 23 | 25 |
| `context_tokens` | 200 | 334 | 472 | 521 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 154 | 2.0 ms | 3.2 ms | 1.2 ms | 2.8 ms | 2.2 ms | 0.14 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 3.0 ms | 3.8 ms | 2.5 ms | 3.2 ms | 2.4 ms | 0.40 ms | 0.00 ms | 0.00 ms |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 200 | 2.5 ms | 3.5 ms | 2.1 ms | 3.0 ms | 2.3 ms | 0.36 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 37 | 2.7 ms | 3.6 ms | 2.3 ms | 3.1 ms | 2.3 ms | 0.40 ms | 0.00 ms | 0.00 ms |
| letter | 79 | 2.5 ms | 3.8 ms | 2.1 ms | 3.2 ms | 2.4 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| notice | 61 | 2.5 ms | 3.2 ms | 2.0 ms | 2.8 ms | 2.2 ms | 0.16 ms | 0.00 ms | 0.00 ms |
| receipt | 11 | 2.1 ms | 2.9 ms | 1.8 ms | 2.5 ms | 2.0 ms | 0.11 ms | 0.00 ms | 0.00 ms |
| statement | 12 | 1.6 ms | 3.7 ms | 0.80 ms | 2.5 ms | 2.0 ms | 0.24 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 14 MB | 14 MB | atlas-val-00221 |
| Model server | 810 MB | 843 MB | atlas-val-00222 |
| Benchmark (engine) | 18 MB | 23 MB | atlas-val-00222 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (3)

- **atlas-val-00056**: expected `2022-10-27 Payment Receipt from Sablewood L.P.docx`, got `2022-10-27 Payment Receipt from Mireille Jablonski.docx` · review: LOW_CONFIDENCE
- **atlas-val-00089**: expected `2023-11-18 Payment Receipt from Quarry & Cobalt Laboratories, Inc.txt`, got `2023-11-18 Payment Receipt from Quillon Valcourt.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00152**: expected `2022-12-28 Request Letter to Ostrava Food Safety Authority.docx`, got `2022-12-28 Request Letter to Orrinholm Nutrition Holdings, Inc.docx` · trap: party Orrinholm Nutrition Holdings, Inc. (other operator mentioned) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
