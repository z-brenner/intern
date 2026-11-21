# InternBench: live run

- **Run:** 2026-11-21T09:35:27Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `97291f77f59a`
- **Corpus:** 200 documents · 200 completed · gold `978c25f73e`
- **Wall time:** 1038.08 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 107/200 (53.5%) |
| Document type | 198/200 (99.0%) |
| Date | 159/200 (79.5%) |
| Date role (when the reviewed date was chosen) | 40/159 (25.2%) |
| Parties | 158/200 (79.0%) |
| Relation word (when parties were named) | 164/196 (83.7%) |
| Party in the right role (when parties were named) | 93/134 (69.4%) |
| Ready / review routing | 0/200 (0.0%) |
| Description states nothing false | 200/200 (100.0%) |
| Description is specific | 187/200 (93.5%) |
| Description specificity | 82.3% (mean of 200) |
| Evidence recall (model's quotes) | 24.4% (mean of 200) |
| Digest recall (facts reaching the digest) | 100.0% (mean of 200) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 200) |
| Sent to review | 100.0% of 200 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | – | 0 |
| Complex-document filename accuracy | 53.5% | 200 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 43.5% | 200 |
| Review rate | 100.0% | 200 |
| Evidence recall | 24.4% | 200 |
| Total latency p50 | 5.08 s | 200 |
| Total latency p95 | 6.18 s | 200 |
| Generation latency p50 | 2.69 s | 200 |
| Generation latency p95 | 3.39 s | 200 |
| Generated tokens p50 | 87 | 200 |
| Generated tokens p95 | 108 | 200 |
| Prompt tokens p50 | 368 | 200 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 200 |
| Trap date chosen | 11 of 200 |
| Forbidden party named | 17 of 200 |
| Spurious parties named | 37 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 675 (0.0%) |
| Right name sent to review anyway | 107 of 200 |
| Sent to review for an unsupported fact | 87 of 200 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 37 | 29/37 | 30/37 | 36/37 | 0/37 | 100.0% | 5.57 s | 6.41 s |
| letter | 79 | 41/79 | 58/79 | 72/79 | 0/79 | 100.0% | 5.36 s | 6.18 s |
| notice | 61 | 24/61 | 54/61 | 27/61 | 0/61 | 100.0% | 4.67 s | 5.37 s |
| receipt | 11 | 7/11 | 10/11 | 11/11 | 0/11 | 100.0% | 3.99 s | 4.12 s |
| statement | 12 | 6/12 | 7/12 | 12/12 | 0/12 | 100.0% | 4.61 s | 5.61 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 120 | 71/120 | 98/120 | 95/120 | 0/120 | 100.0% | 5.00 s | 6.16 s |
| office | 39 | 15/39 | 30/39 | 30/39 | 0/39 | 100.0% | 5.54 s | 6.32 s |
| text | 41 | 21/41 | 31/41 | 33/41 | 0/41 | 100.0% | 4.91 s | 5.48 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 200 | 107/200 | 159/200 | 158/200 | 0/200 | 100.0% | 5.08 s | 6.18 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| complex | 200 | 107/200 | 159/200 | 158/200 | 0/200 | 0 of 200 | 5.08 s | 6.18 s | 2.69 s | 3.39 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| competing_dates | 200 | 107/200 | 159/200 | 158/200 | 28 |
| date_in_table | 12 | 6/12 | 7/12 | 12/12 | 0 |
| docx | 39 | 15/39 | 30/39 | 30/39 | 6 |
| form | 48 | 11/48 | 42/48 | 16/48 | 11 |
| invoice | 37 | 29/37 | 30/37 | 36/37 | 1 |
| irrelevant_names | 79 | 41/79 | 58/79 | 72/79 | 16 |
| key_value | 85 | 40/85 | 72/85 | 52/85 | 12 |
| layout_parties | 24 | 20/24 | 22/24 | 22/24 | 0 |
| letter | 79 | 41/79 | 58/79 | 72/79 | 16 |
| notice | 61 | 24/61 | 54/61 | 27/61 | 11 |
| statement | 12 | 6/12 | 7/12 | 12/12 | 0 |
| table | 49 | 35/49 | 37/49 | 48/49 | 1 |

## Routes

- **Pages:** fast 154 · layout 46
- **Documents by route class:** fast 154 · layout 46

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 200 | 5.08 s | 6.18 s | 7.96 s |
| `extraction_wall_ms` | 200 | 2.4 ms | 3.4 ms | 1.31 s |
| `worker_total_ms` | 200 | 2.0 ms | 2.9 ms | 1.31 s |
| `worker_snapshot_ms` | 200 | 0.21 ms | 0.27 ms | 0.29 ms |
| `worker_parse_ms` | 200 | 1.4 ms | 2.3 ms | 1.31 s |
| `worker_analysis_ms` | 200 | 0.11 ms | 0.36 ms | 0.47 ms |
| `worker_render_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_image_decode_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_encode_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_engine_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_vision_ms` | 200 | 0.00 ms | 0.00 ms | 0.00 ms |
| `analyze_wall_ms` | 200 | 5.06 s | 6.17 s | 7.96 s |
| `distill_ms` | 200 | 1.0 ms | 1.4 ms | 1.8 ms |
| `prompt_ms` | 200 | 0.02 ms | 0.03 ms | 0.06 ms |
| `inference_ms` | 200 | 5.05 s | 6.16 s | 7.95 s |
| `prefill_ms` | 200 | 2.26 s | 3.18 s | 3.54 s |
| `generation_ms` | 200 | 2.69 s | 3.39 s | 5.64 s |
| `validation_ms` | 200 | 3.2 ms | 4.8 ms | 6.9 ms |
| `naming_ms` | 200 | 0.14 ms | 0.19 ms | 0.26 ms |
| `index_ms` | 200 | 0.77 ms | 1.1 ms | 1.5 ms |
| `retrieval_ms` | 200 | 0.01 ms | 0.02 ms | 0.03 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 200 | 162.5 tok/s | 167.7 tok/s | 170.2 tok/s |
| `generation_tok_per_s` | 200 | 32.3 tok/s | 33.8 tok/s | 34.6 tok/s |
| `prompt_tokens` | 200 | 368 | 526 | 584 |
| `cached_tokens` | 200 | 477 | 486 | 486 |
| `generated_tokens` | 200 | 87 | 108 | 177 |
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
| fast | 154 | 1.7 ms | 3.2 ms | 1.3 ms | 2.8 ms | 2.2 ms | 0.15 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 3.0 ms | 3.8 ms | 2.6 ms | 3.2 ms | 2.4 ms | 0.41 ms | 0.00 ms | 0.00 ms |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 200 | 2.4 ms | 3.4 ms | 2.0 ms | 2.9 ms | 2.3 ms | 0.36 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 37 | 2.8 ms | 3.6 ms | 2.4 ms | 3.2 ms | 2.4 ms | 0.40 ms | 0.00 ms | 0.00 ms |
| letter | 79 | 2.6 ms | 3.7 ms | 2.1 ms | 3.0 ms | 2.3 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| notice | 61 | 2.2 ms | 3.3 ms | 1.9 ms | 2.9 ms | 2.3 ms | 0.13 ms | 0.00 ms | 0.00 ms |
| receipt | 11 | 2.1 ms | 3.0 ms | 1.9 ms | 2.6 ms | 2.0 ms | 0.10 ms | 0.00 ms | 0.00 ms |
| statement | 12 | 1.5 ms | 3.2 ms | 1.1 ms | 2.8 ms | 2.1 ms | 0.28 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 14 MB | 14 MB | atlas-val-00099 |
| Model server | 861 MB | 881 MB | atlas-val-00222 |
| Benchmark (engine) | 18 MB | 24 MB | atlas-val-00222 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (93)

- **atlas-val-00000**: expected `2020-03-01 Acknowledgment Letter to Jasper & Vesper Chemical, LLC.pdf`, got `2020-03-25 Letter to Jasper & Vesper Chemical, LLC.pdf` · trap: date 2020-03-25 (response deadline) · review: TYPE_MISSING, LOW_CONFIDENCE
- **atlas-val-00001**: expected `2021-01-28 Notice of Noncompliance for Ash & Yarrow Foods, LLC.pdf`, got `2021-01-28 Notice of Noncompliance from Ostrava Department of Consumer Protection.pdf` · review: LOW_CONFIDENCE
- **atlas-val-00003**: expected `2025-01-03 Notice of Suspension for Valewood Packaging Co.txt`, got `2025-01-03 Notice of Suspension from Calderon Gaming Commission.txt` · review: LOW_CONFIDENCE
- **atlas-val-00005**: expected `2021-03-14 Notice of Violation for Alderwright L.P.pdf`, got `2021-03-14 Notice of Violation from New Wexford Department of Agriculture.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00009**: expected `2027-08-17 Monthly Statement from Larch & Zephyr Water Systems, Inc.docx`, got `2027-07-29 Monthly Statement for Tobiah S. Pryce.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **atlas-val-00010**: expected `2021-11-05 Deficiency Letter to Etheridge Robotics Co.pdf`, got `2021-11-05 Deficiency Letter to Office of Trade Practices.pdf` · review: LOW_CONFIDENCE
- **atlas-val-00013**: expected `2019-10-12 Billing Statement from Pellholm LLC.docx`, got `Billing Statement from Pellholm LLC.docx` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00015**: expected `2026-06-01 Approval Letter to Flintcliff Diagnostics Inc.pdf`, got `2026-06-11 Approval Letter to Flintcliff Diagnostics Inc.pdf` · trap: date 2026-06-11 (response deadline) · review: LOW_CONFIDENCE
- **atlas-val-00019**: expected `2019-05-09 Closing Letter to Marrowridge Company.docx`, got `2019-05-09 Letter to New Wexford Department of Transportation Safety.docx` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00021**: expected `2020-10-18 Invoice from Juniside Aviation L.P.pdf`, got `Invoice from Juniside Aviation L.P.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00023**: expected `2027-05-14 Untitled Letter to Yarrowmark L.P.pdf`, got `Untitled Letter to Yarrowmark L.P.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00025**: expected `2024-01-07 Closing Letter to Mossholm Inc.pdf`, got `2024-01-07 Closing Letter to Wendel R. Okonkwo Chief Executive Officer Mossholm Inc.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00026**: expected `2019-01-12 Information Request Letter to Umber & Nettle Manufacturing, LLC.pdf`, got `2019-01-25 Letter to Umber & Nettle Manufacturing, LLC.pdf` · trap: date 2019-01-25 (response deadline) · review: TYPE_MISSING, LOW_CONFIDENCE
- **atlas-val-00032**: expected `2023-01-28 Warning Letter to Sablestead Incorporated.docx`, got `2023-01-28 Warning Letter to Harlan Department of Consumer Protection.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **atlas-val-00033**: expected `2019-10-02 Invoice from Coppermere Analytics Inc.pdf`, got `Invoice from Coppermere Analytics Inc.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00034**: expected `2019-10-18 Notice of Motion between Wendel Wexley and Flintwell Corp.pdf`, got `Notice of Motion between Wendel Wexley and Flintwell Corp.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00035**: expected `2019-06-18 Notice of Violation for Indigowood Packaging Ltd.pdf`, got `2019-06-18 Notice of Violation for Yorick Y. Pryce Enforcement Officer.pdf` · trap: party Yorick Y. Pryce (signs) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00037**: expected `2020-03-26 Notice of Probable Violation for Larkspur Robotics Group LLC.txt`, got `2020-03-26 Notice of Probable Violation for Harlan Medical Board.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00038**: expected `2019-05-23 Acknowledgment Letter to Wrenside Co.txt`, got `2019-05-23 Acknowledgment Letter to New Wexford Department of Health Services.txt` · review: LOW_CONFIDENCE
- **atlas-val-00040**: expected `2022-05-10 Determination Letter to Indigoline Mills Corp.docx`, got `Determination Letter to Indigoline Mills Corp.docx` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00042**: expected `2025-04-28 Untitled Letter to Indigo & Larch Energy, LLC.docx`, got `2025-04-28 Letter to Dear Piers B. Ishikawa.docx` · trap: party Piers B. Ishikawa (person the letter is addressed to) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00047**: expected `2021-10-12 Notice of Proposed Penalty for Cinderworth Packaging Company.pdf`, got `Notice of Proposed Penalty for Ulric Jablonski Enforcement Officer.pdf` · trap: party Ulric Jablonski (signs) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00054**: expected `2022-03-24 Notice of Settlement between Yusuf F. Castellane and Ember & Thorn Analytics, Inc.docx`, got `2022-03-24 Notice of Settlement for Ember & Thorn Analytics, Inc.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00055**: expected `2026-12-17 Tax Invoice from Yilmaz Aviation Group LLC.pdf`, got `Tax Invoice from Yilmaz Aviation Group LLC.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00056**: expected `2022-10-27 Payment Receipt from Sablewood L.P.docx`, got `2022-10-27 Payment Receipt from Mireille Jablonski.docx` · review: LOW_CONFIDENCE
- **atlas-val-00060**: expected `2020-12-09 Statement of Account from Quill & Saffron Manufacturing, LLC.docx`, got `Statement of Account from Quill & Saffron Manufacturing, LLC.docx` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00061**: expected `2023-03-21 Acknowledgment Letter to Slate & Wick Packaging, LLC.pdf`, got `2023-03-21 Letter.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **atlas-val-00064**: expected `2019-08-08 Notice of Suspension for Moss & Quill Cosmetics, LLC.txt`, got `2019-08-08 Notice of Suspension for Marek Everleigh.txt` · trap: party Marek Everleigh (registered agent) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00066**: expected `2021-10-06 Account Statement from Everleigh Water Systems Corp.pdf`, got `Statement from Everleigh Water Systems Corp.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00067**: expected `2024-11-08 Closing Letter to Northwright Biologics Inc.pdf`, got `2024-11-08 Closing Letter to Niamh Uffington Compliance Manager Northwright Biologics Inc.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00069**: expected `2019-08-12 Untitled Letter to Dovetside Marine Services LLC.pdf`, got `2019-08-12 Untitled Letter to Calderon Department of Environmental Quality.pdf` · review: LOW_CONFIDENCE
- **atlas-val-00071**: expected `2021-09-19 Notice of Appearance between Orrinlane Holdings, Inc and Brackcliff Robotics Co.pdf`, got `2021-09-19 Notice of Appearance.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **atlas-val-00072**: expected `2024-06-12 Notice of Intent to Revoke for Vesper & Copper Chemical, LLC.txt`, got `Notice of Intent to Revoke from New Wexford Department of Consumer Protection.txt` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00077**: expected `2019-02-09 Closing Letter to Cinderside Laboratories Holdings, Inc.pdf`, got `2019-03-07 Letter from Piers Yilmaz President Cinderside Laboratories Holdings, Inc.pdf` · trap: date 2019-03-07 (response deadline) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00078**: expected `2026-02-17 Payment Receipt from Fallowstead Holdings, Inc.pdf`, got `Receipt from Fallowstead Holdings, Inc.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00080**: expected `2021-10-22 Notice of Violation for Quillwell Dairy L.P.docx`, got `2021-10-22 Notice of Violation for East Tamar Department of Transportation Safety.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00081**: expected `2026-02-03 Approval Letter to Elmcrest Ltd.pdf`, got `Approval Letter to Elmcrest Ltd.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00084**: expected `2021-04-16 Notice of Apparent Liability for Rowangate Robotics LLC.txt`, got `2021-04-16 Notice of Apparent Liability from Ostrava Department of Health Services.txt` · review: LOW_CONFIDENCE
- **atlas-val-00085**: expected `2025-01-09 Notice of Suspension for Heronbay L.P.pdf`, got `2025-01-09 Notice of Suspension from Office of Pipeline Integrity.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **atlas-val-00086**: expected `2025-08-24 Acknowledgment Letter to Uffington Mills Ltd.docx`, got `2025-08-24 Acknowledgment Letter to Fenella Q. Jessop.docx` · trap: party Fenella Q. Jessop (person the letter is addressed to) · review: LOW_CONFIDENCE
- … and 53 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
