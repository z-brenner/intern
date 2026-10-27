# InternBench: replay run

- **Run:** 2026-10-27T09:32:01Z · commit `unknown`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.15 GB · sha256 `d406b1e912c0`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-27T08:32:19Z at commit `unknown` · sha256 `5fb5bf3c31`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 47/77 (61.0%) |
| Document type | 71/77 (92.2%) |
| Date | 71/77 (92.2%) |
| Date role (when the reviewed date was chosen) | 49/68 (72.1%) |
| Parties | 70/77 (90.9%) |
| Relation word (when parties were named) | 62/77 (80.5%) |
| Party in the right role (when parties were named) | 60/70 (85.7%) |
| Ready / review routing | 48/57 (84.2%) |
| Description has every fact | 20/77 (26.0%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 77/77 (100.0%) |
| Description fact coverage | 53.7% (mean of 77) |
| Description specificity | 98.3% (mean of 77) |
| Evidence recall (model's quotes) | 68.2% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 11.7% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 58.3% | 12 |
| Complex-document filename accuracy | 58.9% | 56 |
| Description completeness | 53.7% | 77 |
| Unsupported-fact rate (documents) | 7.8% | 77 |
| Review rate | 11.7% | 77 |
| Evidence recall | 68.2% | 77 |
| Total latency p50 | 37.94 s | 77 |
| Total latency p95 | 70.31 s | 77 |
| Generation latency p50 | 10.70 s | 77 |
| Generation latency p95 | 13.68 s | 77 |
| Generated tokens p50 | 87 | 77 |
| Generated tokens p95 | 104 | 77 |
| Prompt tokens p50 | 914 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 24 of 77 |
| Trap date chosen | 5 of 77 |
| Forbidden party named | 3 of 77 |
| Spurious parties named | 7 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 303 (0.0%) |
| Right name sent to review anyway | 2 of 53 |
| Sent to review for an unsupported fact | 6 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 1/1 | 0.0% | 48.50 s | 50.85 s |
| annual_report | 2 | 0/2 | 1/2 | 1/2 | 1/2 | 50.0% | 37.81 s | 57.84 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 44.88 s | 44.88 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 37.94 s | 37.94 s |
| claim | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 35.47 s | 35.47 s |
| condition_report | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 42.29 s | 42.29 s |
| contract | 16 | 14/16 | 15/16 | 15/16 | 12/12 | 0.0% | 48.76 s | 127.62 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 31.42 s | 31.42 s |
| form | 5 | 2/5 | 4/5 | 5/5 | 4/4 | 0.0% | 35.24 s | 42.41 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 27.29 s | 27.29 s |
| insurance | 2 | 1/2 | 1/2 | 2/2 | 2/2 | 0.0% | 49.56 s | 66.56 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 4/5 | 28.6% | 26.45 s | 32.63 s |
| letter | 5 | 2/5 | 5/5 | 5/5 | 3/4 | 20.0% | 35.16 s | 45.09 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 74.79 s | 74.79 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 47.47 s | 47.47 s |
| minutes | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 56.51 s | 56.51 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 38.93 s | 38.93 s |
| notice | 10 | 7/10 | 10/10 | 8/10 | 8/8 | 10.0% | 28.81 s | 47.69 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 35.88 s | 35.88 s |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 36.94 s | 54.87 s |
| price_list | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 42.57 s | 42.57 s |
| purchase_order | 2 | 1/2 | 2/2 | 1/2 | 1/1 | 0.0% | 28.19 s | 39.80 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 33.04 s | 33.04 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 34.53 s | 34.53 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 26.96 s | 26.96 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 30.33 s | 30.33 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 59.75 s | 61.85 s |
| resolution | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 47.77 s | 47.77 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 45.38 s | 45.38 s |
| statement | 3 | 3/3 | 3/3 | 3/3 | 2/3 | 0.0% | 36.36 s | 44.40 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 31.42 s | 31.42 s |
| mixed | 3 | 2/3 | 2/3 | 3/3 | – | 0.0% | 45.36 s | 50.85 s |
| native | 47 | 31/47 | 43/47 | 41/47 | 40/44 | 8.5% | 41.72 s | 66.56 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 26.07 s | 26.07 s |
| office | 5 | 1/5 | 5/5 | 5/5 | 4/5 | 20.0% | 47.77 s | 54.87 s |
| scan | 16 | 9/16 | 15/16 | 16/16 | 1/4 | 12.5% | 31.20 s | 127.62 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 2/2 | 0.0% | 61.85 s | 74.79 s |
| text | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 33.07 s | 33.07 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 22/37 | 36/37 | 35/37 | 21/26 | 16.2% | 31.20 s | 49.74 s |
| 2-4 | 24 | 17/24 | 23/24 | 21/24 | 14/17 | 8.3% | 39.07 s | 56.51 s |
| 5-9 | 4 | 1/4 | 3/4 | 3/4 | 4/4 | 0.0% | 36.94 s | 45.38 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 0.0% | 48.48 s | 75.28 s |
| 25-49 | 4 | 3/4 | 3/4 | 4/4 | 3/3 | 0.0% | 54.34 s | 127.62 s |
| 50-99 | 2 | 0/2 | 1/2 | 1/2 | 2/2 | 0.0% | 66.56 s | 67.39 s |
| 100+ | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 57.84 s | 59.75 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 7/12 | 9/12 | 11/12 | 9/10 | 4 of 12 | 57.84 s | 127.62 s | 10.17 s | 14.26 s |
| complex | 56 | 33/56 | 51/56 | 50/56 | 41/46 | 19 of 56 | 38.93 s | 67.39 s | 10.66 s | 13.68 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 1 |
| competing_dates | 67 | 39/67 | 61/67 | 60/67 | 8 |
| complex_pdf | 5 | 3/5 | 5/5 | 4/5 | 1 |
| contract | 20 | 16/20 | 17/20 | 19/20 | 3 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 4/7 | 5/7 | 6/7 | 3 |
| docx | 3 | 1/3 | 3/3 | 3/3 | 0 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 5/10 | 8/10 | 8/10 | 2 |
| form | 7 | 2/7 | 6/7 | 7/7 | 1 |
| healthcare | 3 | 2/3 | 2/3 | 3/3 | 1 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 9/16 | 15/16 | 16/16 | 1 |
| image_region | 1 | 0/1 | 0/1 | 1/1 | 1 |
| information_dense | 10 | 6/10 | 7/10 | 9/10 | 2 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 8/17 | 14/17 | 15/17 | 3 |
| key_value | 11 | 6/11 | 11/11 | 11/11 | 0 |
| layout_parties | 11 | 7/11 | 11/11 | 11/11 | 0 |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 1/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 2/6 | 3/6 | 5/6 | 2 |
| mixed_scan | 3 | 2/3 | 2/3 | 3/3 | 1 |
| multi_column | 7 | 6/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 7/10 | 10/10 | 8/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 1/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 1/2 | 2/2 | 0 |
| pages_25 | 4 | 3/4 | 3/4 | 4/4 | 1 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 0/2 | 1/2 | 1/2 | 1 |
| png | 5 | 3/5 | 4/5 | 5/5 | 1 |
| pptx | 2 | 0/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 1/2 | 1 |
| referenced_agreement | 20 | 12/20 | 18/20 | 17/20 | 2 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 25/41 | 38/41 | 36/41 | 4 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 2/4 | 4/4 | 3/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 9.25 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 2.44 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.47 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 3.61 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.29 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 2.14 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 2.84 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.05 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 42.57 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 95.15 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 4.82 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 3.80 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 4.86 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 3.41 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 3.20 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 2.05 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.18 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 4.60 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 4.25 s |
| **All scanned pages** |  | 55 | 1.8% | 1.8% | 1.9% | 100.0% | 100.0% | 100.0% | 98 | – |

Error rates are edit distances over the drawn text's length, pooled over pages; Dates, Names and IDs are the fraction of those drawn on the read pages that survive OCR. A page the reader does not return (a TIFF frame it does not read, shown as unread) counts as read empty, as does every page of a scan whose extraction failed: every character and value on it missed.

## Structure

| Document | Route class | Reading order (snippets) | Table rows | Table cells | Key-values | Routes (pages) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice-date-in-table | layout | – | 7/7 (100.0%) | 39/39 (100.0%) | 7/7 (100.0%) | 1/1 (100.0%) |
| invoice-layout-only | layout | – | 5/5 (100.0%) | 25/25 (100.0%) | 7/7 (100.0%) | 1/1 (100.0%) |
| purchase-order | layout | – | 8/8 (100.0%) | 48/48 (100.0%) | 3/3 (100.0%) | 1/1 (100.0%) |
| lease-two-column | layout | 8/8 (100.0%) | 10/10 (100.0%) | 20/20 (100.0%) | – | 1/1 (100.0%) |
| declarations-interleaved | layout | 8/8 (100.0%) | 6/6 (100.0%) | 12/12 (100.0%) | 6/6 (100.0%) | 2/2 (100.0%) |
| vendor-registration-form | layout | – | – | – | 6/6 (100.0%) | 1/1 (100.0%) |
| change-order-form | layout | – | 6/6 (100.0%) | 18/18 (100.0%) | 6/6 (100.0%) | 1/1 (100.0%) |
| scan-patient-intake-form | ocr | – | – | – | 6/6 (100.0%) | 1/1 (100.0%) |
| newsletter-three-column | layout | 9/9 (100.0%) | – | – | – | 1/1 (100.0%) |
| agreement-two-column-footnotes | layout | 10/10 (100.0%) | – | – | – | 2/2 (100.0%) |
| meeting-notice-columns | layout | 11/11 (100.0%) | – | – | – | 1/1 (100.0%) |
| meeting-notice-interleaved | layout | 11/11 (100.0%) | – | – | – | 1/1 (100.0%) |
| meeting-notice-reversed | layout | 11/11 (100.0%) | – | – | – | 1/1 (100.0%) |
| rate-confirmation-rotated | layout | – | 4/4 (100.0%) | 28/28 (100.0%) | 5/5 (100.0%) | 2/2 (100.0%) |
| inspection-log-ruled-2p | layout | – | 45/45 (100.0%) | 315/315 (100.0%) | 5/5 (100.0%) | 2/2 (100.0%) |
| price-list-unruled | layout | – | 21/21 (100.0%) | 126/126 (100.0%) | – | 1/1 (100.0%) |
| invoice-label-above | layout | – | – | – | 8/8 (100.0%) | 1/1 (100.0%) |
| invoice-right-aligned | layout | – | 7/7 (100.0%) | 28/28 (100.0%) | 8/8 (100.0%) | 1/1 (100.0%) |
| invoice-boxed-grid | layout | – | 5/5 (100.0%) | 20/20 (100.0%) | 11/11 (100.0%) | 1/1 (100.0%) |
| benefits-change-checkbox-form | layout | – | 17/17 (100.0%) | 22/22 (100.0%) | 7/7 (100.0%) | 1/1 (100.0%) |
| loss-notice-boxed-fields | layout | – | – | – | 17/17 (100.0%) | 1/1 (100.0%) |
| scan-rotated-page-in-pdf | ocr | – | 6/6 (100.0%) | 36/36 (100.0%) | 5/5 (100.0%) | 2/2 (100.0%) |
| scan-cancellation-notice-150dpi | ocr | – | – | – | 7/7 (100.0%) | 1/1 (100.0%) |
| scan-remittance-advice-120dpi | ocr | – | 6/7 (85.7%) | 34/35 (97.1%) | 7/7 (100.0%) | 1/1 (100.0%) |
| scan-mixed-middle-page | ocr | – | 6/6 (100.0%) | 24/24 (100.0%) | – | 3/3 (100.0%) |
| mixed-signature-region | ocr_regions | 5/5 (100.0%) | – | – | – | 2/2 (100.0%) |
| scan-certificate-of-insurance | ocr | – | 3/5 (60.0%) | 28/30 (93.3%) | 4/5 (80.0%) | 1/1 (100.0%) |
| scan-bill-of-lading | ocr | – | 5/5 (100.0%) | 31/31 (100.0%) | 10/10 (100.0%) | 1/1 (100.0%) |
| msa-effective-date-in-definitions-12p | layout | 3/5 (60.0%) | – | – | 1/1 (100.0%) | – |
| industrial-lease-dated-in-schedule-25p | layout | 5/7 (71.4%) | – | – | 3/3 (100.0%) | – |
| term-loan-parties-apart-40p | layout | 1/8 (12.5%) | – | – | 1/1 (100.0%) | – |
| property-policy-declarations-mid-60p | layout | 1/6 (16.7%) | – | – | 2/2 (100.0%) | – |
| watershed-monitoring-report-100p | layout | 1/8 (12.5%) | – | – | – | – |
| **All** |  | 84/107 (78.5%) | 167/170 (98.2%) | 854/857 (99.6%) | 142/143 (99.3%) | 36/36 (100.0%) |

Measured over the page text the engine receives. Reading order: the most gold snippets found in the gold's order (a longest increasing run), of all of them. Table rows: rows whose cells are all on one line of their table, in order, with no other row between them and an empty check box left empty. Table cells: cells found in their table's lines. Key-values: values after their label on its line (before the next label), alone on the next line, or in the cell under it in a linearised table. Routes: pages whose layout took the expected route, judged only when the worker sends layouts. See `docs/internbench.md` for the exact rules.

## Routes

- **Pages:** fast 121 · layout 392 · ocr 55 · ocr_regions 1
- **Documents by route class:** fast 11 · layout 46 · ocr 19 · ocr_regions 1
- **Pages the gold gives a route for:** 36, 36 judged

| Expected, then taken | fast | layout | ocr | ocr_regions |
| --- | ---: | ---: | ---: | ---: |
| fast | 5 | · | · | · |
| layout | · | 22 | · | · |
| ocr | · | · | 8 | · |
| ocr_regions | · | · | · | 1 |

Pages the gold gives a route for, by the route they should take and the one they took (`none`: the page came without a layout).

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 77 | 37.94 s | 70.31 s | 127.62 s |
| `extraction_wall_ms` | 77 | 8.9 ms | 5.05 s | 95.35 s |
| `worker_total_ms` | 77 | 7.8 ms | 5.05 s | 95.35 s |
| `worker_snapshot_ms` | 77 | 0.26 ms | 0.90 ms | 1.7 ms |
| `worker_parse_ms` | 77 | 3.5 ms | 89.4 ms | 195.9 ms |
| `worker_analysis_ms` | 77 | 0.73 ms | 49.1 ms | 181.4 ms |
| `worker_render_ms` | 77 | 0.00 ms | 346.4 ms | 3.83 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 108.9 ms | 162.5 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 4.86 s | 95.15 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 415.8 ms | 7.38 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 4.35 s | 87.52 s |
| `worker_vision_ms` | 77 | 0.00 ms | 18.7 ms | 40.7 ms |
| `analyze_wall_ms` | 77 | 36.09 s | 66.37 s | 74.79 s |
| `distill_ms` | 77 | 4.7 ms | 129.7 ms | 664.9 ms |
| `prompt_ms` | 77 | 0.05 ms | 0.10 ms | 0.19 ms |
| `inference_ms` | 77 | 36.05 s | 65.68 s | 74.68 s |
| `prefill_ms` | 77 | 25.41 s | 52.35 s | 62.65 s |
| `generation_ms` | 77 | 10.70 s | 13.68 s | 14.80 s |
| `validation_ms` | 77 | 28.6 ms | 226.6 ms | 1.01 s |
| `naming_ms` | 77 | 0.98 ms | 17.3 ms | 36.8 ms |
| `index_ms` | 77 | 4.5 ms | 167.5 ms | 621.8 ms |
| `retrieval_ms` | 77 | 0.05 ms | 8.3 ms | 13.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 36.1 tok/s | 39.2 tok/s | 40.2 tok/s |
| `generation_tok_per_s` | 77 | 8.0 tok/s | 9.0 tok/s | 9.1 tok/s |
| `prompt_tokens` | 77 | 914 | 1861 | 2080 |
| `cached_tokens` | 77 | 477 | 489 | 1104 |
| `generated_tokens` | 77 | 87 | 104 | 111 |
| `estimated_prompt_tokens` | 77 | 1464 | 2506 | 2612 |
| `model_requests` | 77 | 1 | 1 | 1 |
| `redistillations` | 77 | 0 | 0 | 0 |
| `source_characters` | 77 | 2460 | 112672 | 211186 |
| `digest_characters` | 77 | 2496 | 5387 | 6343 |
| `prompt_characters` | 77 | 4545 | 7442 | 8398 |
| `worker_ocr_pages` | 77 | 0 | 2 | 25 |
| `worker_ocr_passes` | 77 | 0 | 2 | 25 |
| `worker_orientation_passes` | 77 | 0 | 2 | 25 |
| `worker_rendered_pixels` | 77 | 0 | 16830000 | 210375000 |
| `index_units` | 77 | 28 | 1339 | 3806 |
| `context_units` | 77 | 25 | 59 | 68 |
| `context_tokens` | 77 | 876 | 1915 | 2022 |

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 11 MB | 688 MB | scan-lease-25p |
| Model server | 3795 MB | 4967 MB | benefits-change-checkbox-form |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (30)

- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order from Juniper Okonkwo.pdf` · trap: party Juniper Okonkwo (buyer contact) · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 Capital Call Notice No. 7 - Ravensmoor Growth GP III, LLC.pdf` · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter from Halloran Ostrowski Cpas LLP.pdf` · **filed without review**
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 invoice from Briarport Coffee Roasters LLC.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2026-03-12 2025 Annual Report for Casimir Ellingsen.pdf` · trap: party Casimir Ellingsen (chief executive who signs the letter) · **filed without review**
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Vendor Registration Form for Hartwell County.pdf` · **filed without review**
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Document - Beckett Halloran.pdf` · trap: party Beckett Halloran (board president) · review: TYPE_UNSUPPORTED
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report for Anselm Greaves.pdf` · **filed without review**
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and The Lenders Party Hereto.pdf` · **filed without review**
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `Annual Report from Tamsin Valley Farmers Cooperative.pdf` · review: DATE_UNSUPPORTED
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Demand for Payment - Highmeadow Orchard Supply Co. - Past-Due Invoices Totaling.docx` · review: DESCRIPTION_UNSUPPORTED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Resolution from Summerhill Robotics, Inc.docx` · **filed without review**
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review - Q3 2026 for Pemberly Falls Distribution Co.pptx` · **filed without review**
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document - Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing to THE CLERK OF THE.txt` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quotation from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · **filed without review**
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-01-01 New Patient Intake Form for Calder Way Family Medicine.png` · trap: date 2026-01-01 (insurance coverage effective date) · **filed without review**
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter of the Saltmarsh Point Homeowners Association from Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Spring Wholesale Availability and Price List from Thistledown Wholesale Nursery.pdf` · **filed without review**
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf` · **filed without review**
- **scan-rotated-page-in-pdf**: expected `2026-04-27 Notice of Freight Claim from Hollin Brook Furniture Gallery.pdf`, got `2026-04-27 Notice of Freight Claim - Claims Department Redgate Interstate Freight Inc.pdf` · **filed without review**
- **scan-cancellation-notice-150dpi**: expected `2026-08-12 Notice of Cancellation for Brannock Tool & Die Inc.pdf`, got `2026-08-12 Notice of Cancellation for Nonpayment of Premium - Brannock Tool & Die Inc.pdf` · review: DATE_AMBIGUOUS
- **mixed-signature-region**: expected `2026-08-21 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf`, got `2026-08-18 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf` · trap: date 2026-08-18 (earlier of the two signatures, not the last) · **filed without review**
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-06-23 Certificate of Liability Insurance Date (MMDDYYYY) for Hartsfield & Quail Insurance Brokers.pdf` · **filed without review**
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading - Short Form from Corriveau Millwork Supply Co.pdf` · **filed without review**
- **industrial-lease-dated-in-schedule-25p**: expected `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`, got `2026-04-20 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf` · trap: date 2026-04-20 (landlord signature date) · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-06-24 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · trap: date 2026-06-24 (countersignature date) · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
