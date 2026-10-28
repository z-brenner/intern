# InternBench: replay run

- **Run:** 2026-10-28T18:24:09Z · commit `1b2167e6bc`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.05 GB · sha256 `43a7d09a0e9c`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T17:51:38Z at commit `1b2167e6bc` · sha256 `f7118ee573`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 51/77 (66.2%) |
| Document type | 66/77 (85.7%) |
| Date | 70/77 (90.9%) |
| Date role (when the reviewed date was chosen) | 49/69 (71.0%) |
| Parties | 73/77 (94.8%) |
| Relation word (when parties were named) | 67/77 (87.0%) |
| Party in the right role (when parties were named) | 64/74 (86.5%) |
| Ready / review routing | 36/57 (63.2%) |
| Description has every fact | 26/77 (33.8%) |
| Description states nothing false | 76/77 (98.7%) |
| Description is specific | 77/77 (100.0%) |
| Description fact coverage | 56.5% (mean of 77) |
| Description specificity | 97.8% (mean of 77) |
| Evidence recall (model's quotes) | 68.4% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 31.2% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 75.0% | 12 |
| Complex-document filename accuracy | 67.9% | 56 |
| Description completeness | 56.5% | 77 |
| Unsupported-fact rate (documents) | 22.1% | 77 |
| Review rate | 31.2% | 77 |
| Evidence recall | 68.4% | 77 |
| Total latency p50 | 20.65 s | 77 |
| Total latency p95 | 29.22 s | 77 |
| Generation latency p50 | 5.80 s | 77 |
| Generation latency p95 | 7.46 s | 77 |
| Generated tokens p50 | 93 | 77 |
| Generated tokens p95 | 114 | 77 |
| Prompt tokens p50 | 1421 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 11 of 77 |
| Trap date chosen | 1 of 77 |
| Forbidden party named | 0 of 77 |
| Spurious parties named | 4 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 307 (0.3%) |
| Right name sent to review anyway | 6 of 53 |
| Sent to review for an unsupported fact | 17 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 0.0% | 22.57 s | 23.89 s |
| annual_report | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 18.61 s | 25.63 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 21.95 s | 21.95 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 24.89 s | 24.89 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 23.44 s | 23.44 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 20.75 s | 20.75 s |
| contract | 16 | 14/16 | 15/16 | 15/16 | 10/12 | 12.5% | 23.72 s | 106.89 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 16.77 s | 16.77 s |
| form | 5 | 1/5 | 2/5 | 4/5 | 2/4 | 60.0% | 20.01 s | 22.01 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 16.56 s | 16.56 s |
| insurance | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 23.18 s | 29.22 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 3/5 | 42.9% | 17.32 s | 18.61 s |
| letter | 5 | 1/5 | 5/5 | 5/5 | 0/4 | 80.0% | 18.94 s | 21.19 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 27.65 s | 27.65 s |
| maintenance_record | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 23.34 s | 23.34 s |
| minutes | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 24.30 s | 24.30 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 20.35 s | 20.35 s |
| notice | 10 | 8/10 | 10/10 | 9/10 | 8/8 | 10.0% | 18.53 s | 26.11 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 17.81 s | 17.81 s |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 18.39 s | 23.66 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 20.32 s | 20.32 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 19.65 s | 21.82 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 22.67 s | 22.67 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 19.64 s | 19.64 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 17.12 s | 17.12 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 20.62 s | 20.62 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 24.32 s | 26.73 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 23.73 s | 23.73 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 20.92 s | 20.92 s |
| statement | 3 | 3/3 | 3/3 | 3/3 | 1/3 | 33.3% | 19.13 s | 22.35 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 16.77 s | 16.77 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 0.0% | 21.67 s | 23.89 s |
| native | 47 | 31/47 | 41/47 | 43/47 | 30/44 | 34.0% | 20.65 s | 29.10 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 18.61 s | 18.61 s |
| office | 5 | 3/5 | 5/5 | 5/5 | 3/5 | 40.0% | 22.69 s | 23.73 s |
| scan | 16 | 11/16 | 15/16 | 16/16 | 1/4 | 18.8% | 20.62 s | 106.89 s |
| sheet | 3 | 2/3 | 3/3 | 3/3 | 1/2 | 33.3% | 24.32 s | 27.65 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 20.06 s | 20.06 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 24/37 | 34/37 | 35/37 | 13/26 | 46.0% | 18.94 s | 24.32 s |
| 2-4 | 24 | 17/24 | 24/24 | 23/24 | 13/17 | 12.5% | 21.20 s | 26.11 s |
| 5-9 | 4 | 1/4 | 2/4 | 4/4 | 2/4 | 50.0% | 18.61 s | 20.92 s |
| 10-24 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 23.66 s | 57.49 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 26.78 s | 106.89 s |
| 50-99 | 2 | 0/2 | 1/2 | 1/2 | 1/2 | 50.0% | 29.22 s | 31.18 s |
| 100+ | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 25.63 s | 26.73 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 9/12 | 10/12 | 11/12 | 8/10 | 1 of 12 | 26.73 s | 106.89 s | 5.68 s | 7.56 s |
| complex | 56 | 38/56 | 50/56 | 52/56 | 31/46 | 7 of 56 | 20.92 s | 29.22 s | 5.68 s | 7.46 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 44/67 | 60/67 | 63/67 | 1 |
| complex_pdf | 5 | 3/5 | 4/5 | 5/5 | 0 |
| contract | 20 | 17/20 | 18/20 | 19/20 | 1 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 6/7 | 7/7 | 7/7 | 0 |
| docx | 3 | 2/3 | 3/3 | 3/3 | 0 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 6/10 | 8/10 | 10/10 | 0 |
| form | 7 | 2/7 | 4/7 | 6/7 | 0 |
| healthcare | 3 | 2/3 | 2/3 | 3/3 | 0 |
| hr | 4 | 2/4 | 3/4 | 4/4 | 0 |
| image_only_scan | 16 | 11/16 | 15/16 | 16/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 7/10 | 8/10 | 9/10 | 1 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 11/17 | 15/17 | 15/17 | 1 |
| key_value | 11 | 8/11 | 10/11 | 11/11 | 0 |
| layout_parties | 11 | 7/11 | 10/11 | 10/11 | 0 |
| letter | 5 | 1/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 2/6 | 3/6 | 5/6 | 1 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 5/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 8/10 | 10/10 | 9/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 3/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 1/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 0 |
| pages_50 | 2 | 0/2 | 1/2 | 1/2 | 1 |
| png | 5 | 3/5 | 4/5 | 5/5 | 0 |
| pptx | 2 | 1/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 12/20 | 18/20 | 18/20 | 1 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 0 |
| spreadsheet | 3 | 2/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 31/41 | 38/41 | 40/41 | 1 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 2/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 1/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 7.21 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 2.48 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.52 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 3.77 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.23 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.85 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 3.21 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.09 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 39.07 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 87.69 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 4.45 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 3.61 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.86 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 3.34 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 3.70 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 2.16 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.38 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 5.31 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 4.09 s |
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

Measured over the page text the engine receives. Reading order: the most gold snippets found in the gold's order (a longest increasing run), of all of them. Table rows: rows whose cells are all on one line of their table, in order, with no other row between them and an empty check box left empty. Table cells: cells found in their table's lines. Key-values: values after their label on its line (before the next label), alone on the next line, or in the cell under it in a linearized table. Routes: pages whose layout took the expected route, judged only when the worker sends layouts. See `docs/internbench.md` for the exact rules.

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
| `total_ms` | 77 | 20.65 s | 29.22 s | 106.89 s |
| `extraction_wall_ms` | 77 | 9.0 ms | 6.05 s | 87.95 s |
| `worker_total_ms` | 77 | 7.9 ms | 6.05 s | 87.95 s |
| `worker_snapshot_ms` | 77 | 0.23 ms | 0.62 ms | 2.7 ms |
| `worker_parse_ms` | 77 | 3.1 ms | 64.7 ms | 116.5 ms |
| `worker_analysis_ms` | 77 | 0.68 ms | 49.0 ms | 94.5 ms |
| `worker_render_ms` | 77 | 0.00 ms | 293.1 ms | 4.52 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 94.6 ms | 172.8 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 5.86 s | 87.69 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 483.5 ms | 6.88 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 5.15 s | 80.56 s |
| `worker_vision_ms` | 77 | 0.00 ms | 7.3 ms | 18.7 ms |
| `analyze_wall_ms` | 77 | 19.44 s | 27.65 s | 31.09 s |
| `distill_ms` | 77 | 4.6 ms | 149.9 ms | 588.2 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.08 ms | 0.10 ms |
| `inference_ms` | 77 | 19.42 s | 27.58 s | 30.53 s |
| `prefill_ms` | 77 | 13.38 s | 21.77 s | 22.96 s |
| `generation_ms` | 77 | 5.80 s | 7.46 s | 8.06 s |
| `validation_ms` | 77 | 28.3 ms | 348.3 ms | 817.8 ms |
| `naming_ms` | 77 | 1.1 ms | 17.0 ms | 25.3 ms |
| `index_ms` | 77 | 4.1 ms | 117.5 ms | 658.7 ms |
| `retrieval_ms` | 77 | 0.05 ms | 6.5 ms | 14.0 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 109.1 tok/s | 113.8 tok/s | 117.1 tok/s |
| `generation_tok_per_s` | 77 | 15.9 tok/s | 16.9 tok/s | 17.2 tok/s |
| `prompt_tokens` | 77 | 1421 | 2416 | 2436 |
| `cached_tokens` | 77 | 0 | 0 | 964 |
| `generated_tokens` | 77 | 93 | 114 | 123 |
| `estimated_prompt_tokens` | 77 | 1606 | 2648 | 2754 |
| `model_requests` | 77 | 1 | 1 | 1 |
| `redistillations` | 77 | 0 | 0 | 0 |
| `source_characters` | 77 | 2460 | 112672 | 211186 |
| `digest_characters` | 77 | 2496 | 5387 | 6343 |
| `prompt_characters` | 77 | 4975 | 7872 | 8828 |
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
| Parser worker | 12 MB | 767 MB | scan-agreement-10p |
| Model server | 2016 MB | 2019 MB | scan-agreement-10p |
| Benchmark (engine) | 10 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (26)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice of Rent Increase from Tamarack Property Management Co.pdf` · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · review: DATE_UNSUPPORTED
- **offer-letter**: expected `2026-02-10 Offer Letter to Dalia Brennagh.pdf`, got `2026-02-10 LETTER to Dalia Brennagh.pdf` · review: TYPE_MISSING
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 LETTER from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document from Moonrake Paper & Packaging Co.pdf` · review: TYPE_UNSUPPORTED
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-06-18 Commercial General Liability Coverage Part Declarations for Wexcombe Bicycle Cooperative.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report from Rookwood Precision Metals Corporation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `Vendor Registration Form for Emberglow Industrial Finishes.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 MINUTES for Board of Directors.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record from Highmeadow Air Charter LLC.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `2026-09-30 Credit Agreement between Marrowfield Packaging Holdings, Inc and The Lenders Party Hereto.pdf` · trap: date 2026-09-30 (first financial covenant test date and amortization date) · review: DESCRIPTION_UNSUPPORTED
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `Annual Report from Tamsin Valley Farmers Cooperative.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 LETTER - Fernvale Cider Works LLC.docx` · review: TYPE_MISSING, DESCRIPTION_UNSUPPORTED
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · **filed without review**
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 REPORT for Rowanbrae Vineyards.xlsx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document from Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt` · **filed without review**
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 QUOTATION from Kestrel Ridge Aero Services Hangar 6, Kestrel Ridge Municipal Airport, UT 84047.tiff` · **filed without review**
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `New Patient Intake Form for Calder Way Family Medicine.png` · review: DATE_UNSUPPORTED
- **agreement-two-column-footnotes**: expected `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`, got `2026-03-02 Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` · review: TYPE_MISSING
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `Benefits Enrollment Change Form for Westbrook Kettle Brewing Co.pdf` · review: DATE_UNSUPPORTED
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf` · **filed without review**
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-06-23 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf` · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Policy Declaration to Hollowmere Craft Brewing Cooperative.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
