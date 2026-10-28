# InternBench: replay run

- **Run:** 2026-10-28T17:51:16Z · commit `1b2167e6bc`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.10 GB · sha256 `3793e1e6e226`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T15:41:56Z at commit `0414b3e412-dirty` · sha256 `bd838ef305`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 50/77 (64.9%) |
| Document type | 73/77 (94.8%) |
| Date | 73/77 (94.8%) |
| Date role (when the reviewed date was chosen) | 48/71 (67.6%) |
| Parties | 68/77 (88.3%) |
| Relation word (when parties were named) | 65/76 (85.5%) |
| Party in the right role (when parties were named) | 61/73 (83.6%) |
| Ready / review routing | 41/57 (71.9%) |
| Description has every fact | 17/77 (22.1%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 75/77 (97.4%) |
| Description fact coverage | 48.7% (mean of 77) |
| Description specificity | 94.8% (mean of 77) |
| Evidence recall (model's quotes) | 85.3% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 24.7% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 12 |
| Complex-document filename accuracy | 67.9% | 56 |
| Description completeness | 48.7% | 77 |
| Unsupported-fact rate (documents) | 18.2% | 77 |
| Review rate | 24.7% | 77 |
| Evidence recall | 85.3% | 77 |
| Total latency p50 | 86.85 s | 77 |
| Total latency p95 | 99.51 s | 77 |
| Generation latency p50 | 72.75 s | 77 |
| Generation latency p95 | 78.00 s | 77 |
| Generated tokens p50 | 1116 | 77 |
| Generated tokens p95 | 1132 | 77 |
| Prompt tokens p50 | 1267 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 19 of 77 |
| Trap date chosen | 3 of 77 |
| Forbidden party named | 1 of 77 |
| Spurious parties named | 7 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 288 (0.0%) |
| Right name sent to review anyway | 8 of 53 |
| Sent to review for an unsupported fact | 14 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 2/3 | 0/1 | 33.3% | 90.65 s | 91.86 s |
| annual_report | 2 | 0/2 | 1/2 | 1/2 | 1/2 | 50.0% | 84.38 s | 93.54 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 87.46 s | 87.46 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 87.01 s | 87.01 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 92.18 s | 92.18 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 84.48 s | 84.48 s |
| contract | 16 | 15/16 | 16/16 | 15/16 | 11/12 | 6.2% | 90.12 s | 161.64 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 83.15 s | 83.15 s |
| form | 5 | 1/5 | 3/5 | 4/5 | 4/4 | 0.0% | 86.57 s | 90.66 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 80.16 s | 80.16 s |
| insurance | 2 | 1/2 | 1/2 | 2/2 | 2/2 | 0.0% | 90.57 s | 96.14 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 3/5 | 42.9% | 83.88 s | 110.07 s |
| letter | 5 | 2/5 | 5/5 | 5/5 | 1/4 | 60.0% | 83.00 s | 89.84 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 93.75 s | 93.75 s |
| maintenance_record | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 84.27 s | 84.27 s |
| minutes | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 91.47 s | 91.47 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 90.06 s | 90.06 s |
| notice | 10 | 8/10 | 10/10 | 8/10 | 6/8 | 30.0% | 83.36 s | 92.27 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 84.59 s | 84.59 s |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 85.82 s | 97.85 s |
| price_list | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 89.24 s | 89.24 s |
| purchase_order | 2 | 0/2 | 2/2 | 1/2 | 0/1 | 50.0% | 80.96 s | 93.52 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 88.92 s | 88.92 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 86.85 s | 86.85 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 79.54 s | 79.54 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 85.08 s | 85.08 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 90.33 s | 91.80 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 91.12 s | 91.12 s |
| sow | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 89.71 s | 89.71 s |
| statement | 3 | 2/3 | 3/3 | 3/3 | 1/3 | 33.3% | 85.98 s | 87.49 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 83.15 s | 83.15 s |
| mixed | 3 | 3/3 | 3/3 | 2/3 | – | 0.0% | 83.16 s | 90.65 s |
| native | 47 | 31/47 | 44/47 | 40/47 | 33/44 | 25.5% | 87.22 s | 96.14 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 81.50 s | 81.50 s |
| office | 5 | 2/5 | 5/5 | 5/5 | 3/5 | 40.0% | 86.07 s | 97.85 s |
| scan | 16 | 10/16 | 15/16 | 16/16 | 2/4 | 18.8% | 85.08 s | 161.64 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 2/2 | 0.0% | 90.33 s | 93.75 s |
| text | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 83.32 s | 83.32 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 20/37 | 35/37 | 32/37 | 16/26 | 37.8% | 85.08 s | 93.52 s |
| 2-4 | 24 | 21/24 | 24/24 | 22/24 | 13/17 | 12.5% | 87.23 s | 93.75 s |
| 5-9 | 4 | 1/4 | 3/4 | 4/4 | 2/4 | 50.0% | 85.82 s | 89.71 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 0.0% | 92.91 s | 121.68 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 88.30 s | 161.64 s |
| 50-99 | 2 | 0/2 | 1/2 | 1/2 | 2/2 | 0.0% | 94.56 s | 96.14 s |
| 100+ | 2 | 1/2 | 2/2 | 1/2 | 2/2 | 0.0% | 91.80 s | 93.54 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 8/12 | 11/12 | 10/12 | 10/10 | 4 of 12 | 93.54 s | 161.64 s | 72.66 s | 77.57 s |
| complex | 56 | 38/56 | 53/56 | 48/56 | 34/46 | 13 of 56 | 87.22 s | 99.51 s | 73.85 s | 78.82 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 2/3 | 0 |
| competing_dates | 67 | 43/67 | 63/67 | 58/67 | 4 |
| complex_pdf | 5 | 3/5 | 4/5 | 5/5 | 0 |
| contract | 20 | 18/20 | 20/20 | 18/20 | 0 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 5/7 | 6/7 | 6/7 | 2 |
| docx | 3 | 2/3 | 3/3 | 3/3 | 0 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 6/10 | 8/10 | 9/10 | 1 |
| form | 7 | 2/7 | 5/7 | 6/7 | 2 |
| healthcare | 3 | 2/3 | 2/3 | 3/3 | 1 |
| hr | 4 | 3/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 10/16 | 15/16 | 16/16 | 1 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 7/10 | 9/10 | 8/10 | 1 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 11/17 | 16/17 | 14/17 | 1 |
| key_value | 11 | 8/11 | 11/11 | 11/11 | 0 |
| layout_parties | 11 | 7/11 | 10/11 | 10/11 | 1 |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 2/6 | 5/6 | 4/6 | 1 |
| mixed_scan | 3 | 3/3 | 3/3 | 2/3 | 0 |
| multi_column | 7 | 6/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 8/10 | 10/10 | 8/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 3/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 2/2 | 1/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 0/2 | 1/2 | 1/2 | 1 |
| png | 5 | 3/5 | 4/5 | 5/5 | 1 |
| pptx | 2 | 0/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 0/2 | 2/2 | 1/2 | 1 |
| referenced_agreement | 20 | 13/20 | 20/20 | 16/20 | 0 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sow | 1 | 0/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 2/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 30/41 | 39/41 | 37/41 | 2 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 2/4 | 4/4 | 3/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 11.94 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 2.37 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.45 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 3.23 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.23 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.82 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 2.94 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.29 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 40.07 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 81.00 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 5.14 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.12 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.96 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 3.69 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 3.83 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 2.23 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.17 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 4.82 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 4.26 s |
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
| `total_ms` | 77 | 86.85 s | 99.51 s | 161.64 s |
| `extraction_wall_ms` | 77 | 9.7 ms | 6.18 s | 81.20 s |
| `worker_total_ms` | 77 | 9.0 ms | 6.18 s | 81.20 s |
| `worker_snapshot_ms` | 77 | 0.24 ms | 0.87 ms | 2.6 ms |
| `worker_parse_ms` | 77 | 3.3 ms | 67.7 ms | 122.5 ms |
| `worker_analysis_ms` | 77 | 0.76 ms | 60.9 ms | 99.2 ms |
| `worker_render_ms` | 77 | 0.00 ms | 262.7 ms | 5.08 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 90.0 ms | 207.7 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 5.96 s | 81.00 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 446.7 ms | 6.50 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 5.31 s | 74.29 s |
| `worker_vision_ms` | 77 | 0.00 ms | 13.2 ms | 18.0 ms |
| `analyze_wall_ms` | 77 | 85.98 s | 95.96 s | 109.99 s |
| `distill_ms` | 77 | 5.2 ms | 148.6 ms | 592.8 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.09 ms | 0.14 ms |
| `inference_ms` | 77 | 85.95 s | 95.32 s | 109.94 s |
| `prefill_ms` | 77 | 11.92 s | 20.24 s | 21.69 s |
| `generation_ms` | 77 | 72.75 s | 78.00 s | 98.88 s |
| `validation_ms` | 77 | 29.6 ms | 332.2 ms | 712.9 ms |
| `naming_ms` | 77 | 0.94 ms | 12.7 ms | 20.8 ms |
| `index_ms` | 77 | 4.7 ms | 139.1 ms | 585.5 ms |
| `retrieval_ms` | 77 | 0.05 ms | 6.8 ms | 13.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 105.4 tok/s | 115.1 tok/s | 115.7 tok/s |
| `generation_tok_per_s` | 77 | 15.4 tok/s | 16.1 tok/s | 16.2 tok/s |
| `prompt_tokens` | 77 | 1267 | 2262 | 2282 |
| `cached_tokens` | 77 | 0 | 0 | 810 |
| `generated_tokens` | 77 | 1116 | 1132 | 1143 |
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
| Parser worker | 11 MB | 761 MB | scan-agreement-10p |
| Model server | 1959 MB | 1962 MB | scan-bill-of-lading |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (27)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice of Rent Increase from Tamarack Property Management Co.pdf` · **filed without review**
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order from Juniper Okonkwo.pdf` · trap: party Juniper Okonkwo (buyer contact) · review: DESCRIPTION_UNSUPPORTED
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2026-05-26 Statement of Work No. 3 between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document from Moonrake Paper & Packaging Co.pdf` · review: TYPE_UNSUPPORTED
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2025 Annual Report from Rookwood Precision Metals Corporation.pdf` · review: DATE_UNSUPPORTED
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2014-11-03 Vendor Registration Form for Emberglow Industrial Finishes.pdf` · trap: date 2014-11-03 (date of incorporation) · **filed without review**
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes of the Regular Meeting of the Board of Directors for Board of Directors.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and The Lenders Party Hereto.pdf` · **filed without review**
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `2026-06-30 Annual Report for Board of Directors and Members.pdf` · **filed without review**
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Demand for Payment - Fernvale Cider Works LLC.docx` · review: DESCRIPTION_UNSUPPORTED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review - Q3 2026 for Pemberly Falls Distribution Co.pptx` · **filed without review**
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document from Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing to THE CLERK OF THE.txt` · **filed without review**
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement of Account from Copper Flats Dental Group PLLC.pdf` · **filed without review**
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quotation from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · **filed without review**
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-01-01 New Patient Intake Form for Calder Way Family Medicine.png` · trap: date 2026-01-01 (insurance coverage effective date) · **filed without review**
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter of the Saltmarsh Point Homeowners Association from Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Spring Wholesale Availability and Price List from Thistledown Wholesale Nursery.pdf` · **filed without review**
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-29 Benefits Enrollment Change Form for Westbrook Kettle Brewing Co.pdf` · **filed without review**
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf` · **filed without review**
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading - Short Form from Corriveau Millwork Supply Co.pdf` · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-06-24 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · trap: date 2026-06-24 (countersignature date) · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
