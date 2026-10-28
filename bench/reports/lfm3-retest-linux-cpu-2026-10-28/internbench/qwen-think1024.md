# InternBench: replay run

- **Run:** 2026-10-28T21:38:52Z · commit `1b2167e6bc`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T19:16:37Z at commit `1b2167e6bc` · sha256 `6f59b961bd`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 47/77 (61.0%) |
| Document type | 70/77 (90.9%) |
| Date | 72/77 (93.5%) |
| Date role (when the reviewed date was chosen) | 42/71 (59.2%) |
| Parties | 69/77 (89.6%) |
| Relation word (when parties were named) | 61/76 (80.3%) |
| Party in the right role (when parties were named) | 59/70 (84.3%) |
| Ready / review routing | 45/57 (79.0%) |
| Description has every fact | 21/77 (27.3%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 74/77 (96.1%) |
| Description fact coverage | 52.6% (mean of 77) |
| Description specificity | 94.4% (mean of 77) |
| Evidence recall (model's quotes) | 74.0% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 18.2% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 83.3% | 12 |
| Complex-document filename accuracy | 64.3% | 56 |
| Description completeness | 52.6% | 77 |
| Unsupported-fact rate (documents) | 11.7% | 77 |
| Review rate | 18.2% | 77 |
| Evidence recall | 74.0% | 77 |
| Total latency p50 | 92.76 s | 77 |
| Total latency p95 | 102.29 s | 77 |
| Generation latency p50 | 82.68 s | 77 |
| Generation latency p95 | 86.52 s | 77 |
| Generated tokens p50 | 1109 | 77 |
| Generated tokens p95 | 1137 | 77 |
| Prompt tokens p50 | 868 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 21 of 77 |
| Trap date chosen | 4 of 77 |
| Forbidden party named | 4 of 77 |
| Spurious parties named | 6 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 283 (0.0%) |
| Right name sent to review anyway | 2 of 53 |
| Sent to review for an unsupported fact | 9 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 0.0% | 93.20 s | 100.20 s |
| annual_report | 2 | 1/2 | 1/2 | 2/2 | 2/2 | 0.0% | 89.47 s | 101.32 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 96.18 s | 96.18 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 98.89 s | 98.89 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 91.58 s | 91.58 s |
| condition_report | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 90.96 s | 90.96 s |
| contract | 16 | 15/16 | 16/16 | 15/16 | 12/12 | 6.2% | 95.46 s | 173.57 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 88.77 s | 88.77 s |
| form | 5 | 4/5 | 4/5 | 5/5 | 4/4 | 0.0% | 91.82 s | 98.59 s |
| inspection_report | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 87.04 s | 87.04 s |
| insurance | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 98.12 s | 103.88 s |
| invoice | 7 | 5/7 | 5/7 | 7/7 | 2/5 | 57.1% | 89.89 s | 90.57 s |
| letter | 5 | 2/5 | 5/5 | 5/5 | 3/4 | 20.0% | 87.77 s | 95.22 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 101.90 s | 101.90 s |
| maintenance_record | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 94.89 s | 94.89 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 93.70 s | 93.70 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 93.84 s | 93.84 s |
| notice | 10 | 6/10 | 10/10 | 8/10 | 7/8 | 30.0% | 89.32 s | 95.47 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 87.70 s | 87.70 s |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 90.69 s | 98.67 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 90.09 s | 90.09 s |
| purchase_order | 2 | 0/2 | 2/2 | 1/2 | 1/1 | 0.0% | 91.91 s | 93.19 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 96.09 s | 96.09 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 88.11 s | 88.11 s |
| receipt | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 91.71 s | 91.71 s |
| remittance_advice | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 93.81 s | 93.81 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 98.73 s | 102.26 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 98.91 s | 98.91 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 92.70 s | 92.70 s |
| statement | 3 | 1/3 | 3/3 | 3/3 | 1/3 | 33.3% | 91.10 s | 95.26 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 88.77 s | 88.77 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 0.0% | 92.94 s | 100.20 s |
| native | 47 | 30/47 | 42/47 | 43/47 | 38/44 | 12.8% | 92.71 s | 102.26 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 89.98 s | 89.98 s |
| office | 5 | 0/5 | 5/5 | 3/5 | 4/5 | 20.0% | 93.77 s | 98.91 s |
| scan | 16 | 10/16 | 16/16 | 15/16 | 1/4 | 25.0% | 93.69 s | 173.57 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 2/2 | 0.0% | 98.73 s | 101.90 s |
| text | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 89.32 s | 89.32 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 17/37 | 34/37 | 29/37 | 17/26 | 29.7% | 90.57 s | 98.89 s |
| 2-4 | 24 | 19/24 | 24/24 | 24/24 | 14/17 | 8.3% | 93.56 s | 100.20 s |
| 5-9 | 4 | 1/4 | 2/4 | 4/4 | 4/4 | 0.0% | 90.69 s | 92.70 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 25.0% | 94.19 s | 122.54 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 97.99 s | 173.57 s |
| 50-99 | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 102.29 s | 103.88 s |
| 100+ | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 101.32 s | 102.26 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 10/12 | 12/12 | 12/12 | 10/10 | 2 of 12 | 100.24 s | 173.57 s | 82.50 s | 85.81 s |
| complex | 56 | 36/56 | 51/56 | 50/56 | 39/46 | 13 of 56 | 92.94 s | 102.29 s | 82.68 s | 86.42 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 39/67 | 62/67 | 59/67 | 8 |
| complex_pdf | 5 | 3/5 | 4/5 | 5/5 | 1 |
| contract | 20 | 18/20 | 19/20 | 19/20 | 1 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 4/7 | 7/7 | 6/7 | 1 |
| docx | 3 | 0/3 | 3/3 | 1/3 | 1 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 6/10 | 9/10 | 10/10 | 1 |
| form | 7 | 5/7 | 6/7 | 6/7 | 2 |
| healthcare | 3 | 2/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 2/4 | 3/4 | 3/4 | 1 |
| image_only_scan | 16 | 10/16 | 16/16 | 15/16 | 1 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 9/10 | 10/10 | 10/10 | 0 |
| invoice | 7 | 5/7 | 5/7 | 7/7 | 1 |
| irrelevant_names | 17 | 13/17 | 17/17 | 16/17 | 1 |
| key_value | 11 | 6/11 | 9/11 | 10/11 | 3 |
| layout_parties | 11 | 7/11 | 9/11 | 10/11 | 2 |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 1/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 4/6 | 5/6 | 6/6 | 1 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 6/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 1/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 6/10 | 10/10 | 8/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 1/4 | 4/4 | 3/4 | 1 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 0/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 0/2 | 2/2 | 1/2 | 1 |
| referenced_agreement | 20 | 12/20 | 19/20 | 17/20 | 1 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 1/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 25/41 | 39/41 | 36/41 | 5 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 1/4 | 4/4 | 2/4 | 1 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 5.96 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 3.12 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.23 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 3.88 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.92 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 2.07 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 3.47 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.22 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 34.33 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 82.44 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 5.24 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.01 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 6.18 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 4.12 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 4.48 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 2.74 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.85 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 5.42 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 4.97 s |
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
| `total_ms` | 77 | 92.76 s | 102.29 s | 173.57 s |
| `extraction_wall_ms` | 77 | 8.4 ms | 6.21 s | 82.70 s |
| `worker_total_ms` | 77 | 7.6 ms | 6.21 s | 82.69 s |
| `worker_snapshot_ms` | 77 | 0.24 ms | 0.82 ms | 2.5 ms |
| `worker_parse_ms` | 77 | 2.8 ms | 77.9 ms | 136.0 ms |
| `worker_analysis_ms` | 77 | 0.66 ms | 48.1 ms | 101.1 ms |
| `worker_render_ms` | 77 | 0.00 ms | 612.9 ms | 5.26 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 138.7 ms | 222.4 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 5.96 s | 82.44 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 546.5 ms | 6.63 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 5.10 s | 75.26 s |
| `worker_vision_ms` | 77 | 0.00 ms | 8.6 ms | 18.4 ms |
| `analyze_wall_ms` | 77 | 91.05 s | 101.90 s | 103.64 s |
| `distill_ms` | 77 | 5.5 ms | 130.5 ms | 615.6 ms |
| `prompt_ms` | 77 | 0.05 ms | 0.09 ms | 0.15 ms |
| `inference_ms` | 77 | 91.01 s | 100.68 s | 102.87 s |
| `prefill_ms` | 77 | 8.09 s | 16.26 s | 19.16 s |
| `generation_ms` | 77 | 82.68 s | 86.52 s | 87.57 s |
| `validation_ms` | 77 | 27.7 ms | 358.9 ms | 655.4 ms |
| `naming_ms` | 77 | 1.1 ms | 12.9 ms | 23.8 ms |
| `index_ms` | 77 | 4.1 ms | 114.3 ms | 524.4 ms |
| `retrieval_ms` | 77 | 0.05 ms | 6.0 ms | 12.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 111.8 tok/s | 118.5 tok/s | 122.4 tok/s |
| `generation_tok_per_s` | 77 | 13.4 tok/s | 14.1 tok/s | 14.2 tok/s |
| `prompt_tokens` | 77 | 868 | 1859 | 2078 |
| `cached_tokens` | 77 | 477 | 477 | 1102 |
| `generated_tokens` | 77 | 1109 | 1137 | 1142 |
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
| Parser worker | 12 MB | 762 MB | scan-agreement-10p |
| Model server | 2081 MB | 2082 MB | scan-clean-lease-2p |
| Benchmark (engine) | 10 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (30)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice from Tamarack Property Management Co.pdf` · **filed without review**
- **invoice-layout-only**: expected `2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf`, got `Invoice from Ironvale Marine Coatings Co.pdf` · review: DATE_UNSUPPORTED
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order from Juniper Okonkwo.pdf` · trap: party Juniper Okonkwo (buyer contact) · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Statement from Meadowlark Health Plan.pdf` · **filed without review**
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 Notice from Ravensmoor Growth Partners III, L.P.pdf` · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Trade Credit Application for Moonrake Paper & Packaging Co.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2026-05-21 2025 Annual Report from Rookwood Precision Metals Corporation.pdf` · trap: date 2026-05-21 (annual meeting date) · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report from Haverford Family Collection.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record from Arlo Sinclair-Ray.pdf` · trap: party Arlo Sinclair-Ray (inspector who signed the return to service) · **filed without review**
- **separation-agreement**: expected `2026-04-06 Separation Agreement and General Release between Mossgiel Biomedical Inc and Corinne Abernathy.docx`, got `2026-04-06 Separation Agreement and General Release with Mossgiel Biomedical Inc.docx` · **filed without review**
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Demand for Payment - Fernvale Cider Works LLC.docx` · **filed without review**
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors for Celeste Fontaine.docx` · trap: party Celeste Fontaine (director) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx` · **filed without review**
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document - Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing to THE CLERK OF THE.txt` · review: DESCRIPTION_UNSUPPORTED
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement of Account from Copper Flats Dental Group PLLC.pdf` · **filed without review**
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quotation from Kestrel Ridge Aero.tiff` · **filed without review**
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter of the Saltmarsh Point Homeowners Association from Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **inspection-log-ruled-2p**: expected `2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf`, got `2026-05-19 Fire Extinguisher Inspection Report from Corbel Street Lofts Condominium Association.pdf` · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Price List.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **invoice-label-above**: expected `2026-05-11 Invoice from Quarrystone Signs & Graphics.pdf`, got `2026-05-06 Invoice from Quarrystone Signs & Graphics.pdf` · trap: date 2026-05-06 (installation date) · review: DATE_AMBIGUOUS
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-20 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf` · trap: date 2026-06-20 (date of the qualifying event (marriage)) · **filed without review**
- **scan-cancellation-notice-150dpi**: expected `2026-08-12 Notice of Cancellation for Brannock Tool & Die Inc.pdf`, got `2026-08-12 Notice of Cancellation for Nonpayment of Premium - Brannock Tool & Die Inc.pdf` · review: DATE_AMBIGUOUS
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-07-17 Document from Tolliver Grain & Feed Cooperative.png` · review: TYPE_UNSUPPORTED
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading - Odile Corriveau.pdf` · trap: party Odile Corriveau (shipper's signer) · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
