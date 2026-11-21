# InternBench: live run

- **Run:** 2026-11-21T05:04:45Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 1700.68 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 67/77 (87.0%) |
| Document type | 74/77 (96.1%) |
| Date | 75/77 (97.4%) |
| Date role (when the reviewed date was chosen) | 42/72 (58.3%) |
| Parties | 74/77 (96.1%) |
| Relation word (when parties were named) | 71/74 (96.0%) |
| Party in the right role (when parties were named) | 71/72 (98.6%) |
| Ready / review routing | 18/57 (31.6%) |
| Description has every fact | 20/77 (26.0%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 75/77 (97.4%) |
| Description fact coverage | 52.8% (mean of 77) |
| Description specificity | 93.9% (mean of 77) |
| Evidence recall (model's quotes) | 62.8% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 63.6% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 91.7% | 12 |
| Complex-document filename accuracy | 87.5% | 56 |
| Description completeness | 52.8% | 77 |
| Unsupported-fact rate (documents) | 13.0% | 77 |
| Review rate | 63.6% | 77 |
| Evidence recall | 62.8% | 77 |
| Total latency p50 | 18.25 s | 77 |
| Total latency p95 | 42.76 s | 77 |
| Generation latency p50 | 4.75 s | 77 |
| Generation latency p95 | 6.27 s | 77 |
| Generated tokens p50 | 84 | 77 |
| Generated tokens p95 | 111 | 77 |
| Prompt tokens p50 | 978 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 77 |
| Trap date chosen | 1 of 77 |
| Forbidden party named | 0 of 77 |
| Spurious parties named | 0 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 278 (0.0%) |
| Right name sent to review anyway | 30 of 53 |
| Sent to review for an unsupported fact | 10 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 0.0% | 21.86 s | 22.99 s |
| annual_report | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 18.49 s | 40.25 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 19.67 s | 19.67 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 20.90 s | 20.90 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 19.11 s | 19.11 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 17.82 s | 17.82 s |
| contract | 16 | 15/16 | 15/16 | 16/16 | 8/12 | 25.0% | 30.20 s | 67.23 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 14.70 s | 14.70 s |
| form | 5 | 5/5 | 5/5 | 5/5 | 1/4 | 80.0% | 15.19 s | 19.76 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 35.92 s | 35.92 s |
| insurance | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 20.94 s | 48.49 s |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 0/5 | 100.0% | 12.75 s | 13.82 s |
| letter | 5 | 2/5 | 4/5 | 5/5 | 0/4 | 80.0% | 17.24 s | 20.39 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 32.18 s | 32.18 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 20.86 s | 20.86 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 22.94 s | 22.94 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 17.15 s | 17.15 s |
| notice | 10 | 10/10 | 10/10 | 10/10 | 1/8 | 80.0% | 13.92 s | 23.92 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 18.72 s | 18.72 s |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 16.23 s | 24.84 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 19.31 s | 19.31 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 100.0% | 13.48 s | 18.07 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 15.51 s | 15.51 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 16.09 s | 16.09 s |
| receipt | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 12.35 s | 12.35 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 16.87 s | 16.87 s |
| report | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 26.29 s | 38.22 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 23.25 s | 23.25 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 24.78 s | 24.78 s |
| statement | 3 | 1/3 | 3/3 | 3/3 | 0/3 | 66.7% | 22.52 s | 37.54 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 14.70 s | 14.70 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 0.0% | 18.25 s | 22.99 s |
| native | 47 | 41/47 | 45/47 | 46/47 | 13/44 | 70.2% | 19.52 s | 42.76 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 12.75 s | 12.75 s |
| office | 5 | 4/5 | 5/5 | 5/5 | 2/5 | 60.0% | 23.25 s | 24.84 s |
| scan | 16 | 14/16 | 16/16 | 15/16 | 3/4 | 50.0% | 15.19 s | 67.23 s |
| sheet | 3 | 2/3 | 3/3 | 2/3 | 0/2 | 66.7% | 26.29 s | 32.18 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 16.85 s | 16.85 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 29/37 | 36/37 | 34/37 | 4/26 | 81.1% | 14.07 s | 24.05 s |
| 2-4 | 24 | 23/24 | 24/24 | 24/24 | 8/17 | 45.8% | 19.11 s | 37.54 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 2/4 | 50.0% | 17.77 s | 24.78 s |
| 10-24 | 4 | 3/4 | 3/4 | 4/4 | 2/3 | 25.0% | 29.53 s | 40.28 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 2/3 | 25.0% | 35.35 s | 67.23 s |
| 50-99 | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 42.76 s | 48.49 s |
| 100+ | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 38.22 s | 40.25 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 11/12 | 11/12 | 12/12 | 4/10 | 0 of 12 | 38.22 s | 67.23 s | 4.88 s | 7.17 s |
| complex | 56 | 49/56 | 54/56 | 53/56 | 14/46 | 0 of 56 | 19.11 s | 47.04 s | 4.85 s | 6.07 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 59/67 | 66/67 | 65/67 | 1 |
| complex_pdf | 5 | 5/5 | 5/5 | 5/5 | 0 |
| contract | 20 | 19/20 | 19/20 | 20/20 | 1 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_table | 7 | 5/7 | 7/7 | 6/7 | 0 |
| docx | 3 | 2/3 | 3/3 | 3/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 8/10 | 10/10 | 9/10 | 0 |
| form | 7 | 6/7 | 7/7 | 6/7 | 0 |
| healthcare | 3 | 2/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 14/16 | 16/16 | 15/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 9/10 | 9/10 | 10/10 | 1 |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 0 |
| irrelevant_names | 17 | 15/17 | 16/17 | 16/17 | 1 |
| key_value | 11 | 10/11 | 11/11 | 10/11 | 0 |
| layout_parties | 11 | 8/11 | 10/11 | 9/11 | 0 |
| letter | 5 | 2/5 | 4/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 5/6 | 5/6 | 6/6 | 1 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 7/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 10/10 | 10/10 | 10/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 3/4 | 4/4 | 3/4 | 0 |
| pages_10 | 3 | 2/3 | 2/3 | 3/3 | 1 |
| pages_100 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| png | 5 | 5/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 2/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 18/20 | 19/20 | 20/20 | 0 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 4/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 2/3 | 3/3 | 2/3 | 0 |
| statement | 3 | 1/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 34/41 | 40/41 | 38/41 | 1 |
| tiff | 2 | 2/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 4/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.62 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.20 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.28 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.83 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.87 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 992.6 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.60 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.71 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 41.87 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 90.02 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.61 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.04 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.27 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.96 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.04 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.21 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 709.4 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.80 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.43 s |
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
| `total_ms` | 77 | 18.25 s | 42.76 s | 67.23 s |
| `extraction_wall_ms` | 77 | 5.1 ms | 2.93 s | 45.91 s |
| `worker_total_ms` | 77 | 4.5 ms | 2.93 s | 45.91 s |
| `worker_snapshot_ms` | 77 | 0.22 ms | 0.38 ms | 1.2 ms |
| `worker_parse_ms` | 77 | 2.4 ms | 50.5 ms | 106.5 ms |
| `worker_analysis_ms` | 77 | 0.53 ms | 35.5 ms | 83.9 ms |
| `worker_render_ms` | 77 | 0.00 ms | 205.2 ms | 4.36 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 66.2 ms | 90.8 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 4.62 s | 90.02 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 276.3 ms | 5.87 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 3.96 s | 83.79 s |
| `worker_vision_ms` | 77 | 0.00 ms | 6.0 ms | 12.1 ms |
| `analyze_wall_ms` | 77 | 17.81 s | 42.45 s | 48.34 s |
| `distill_ms` | 77 | 2.3 ms | 82.6 ms | 245.7 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.07 ms | 0.08 ms |
| `inference_ms` | 77 | 17.80 s | 42.43 s | 48.14 s |
| `prefill_ms` | 77 | 13.05 s | 36.67 s | 43.25 s |
| `generation_ms` | 77 | 4.75 s | 6.27 s | 7.33 s |
| `validation_ms` | 77 | 5.0 ms | 37.9 ms | 68.9 ms |
| `naming_ms` | 77 | 0.26 ms | 6.0 ms | 9.5 ms |
| `index_ms` | 77 | 1.9 ms | 76.8 ms | 235.3 ms |
| `retrieval_ms` | 77 | 0.03 ms | 4.8 ms | 9.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 74.3 tok/s | 75.3 tok/s | 75.6 tok/s |
| `generation_tok_per_s` | 77 | 17.9 tok/s | 18.5 tok/s | 18.7 tok/s |
| `prompt_tokens` | 77 | 978 | 2685 | 3139 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 84 | 111 | 125 |
| `estimated_prompt_tokens` | 77 | 1545 | 3216 | 3600 |
| `model_requests` | 77 | 1 | 1 | 1 |
| `redistillations` | 77 | 0 | 0 | 0 |
| `source_characters` | 77 | 2460 | 112672 | 211186 |
| `digest_characters` | 77 | 2673 | 7639 | 8560 |
| `prompt_characters` | 77 | 4722 | 9694 | 10615 |
| `worker_ocr_pages` | 77 | 0 | 2 | 25 |
| `worker_ocr_passes` | 77 | 0 | 2 | 25 |
| `worker_orientation_passes` | 77 | 0 | 2 | 25 |
| `worker_rendered_pixels` | 77 | 0 | 16830000 | 210375000 |
| `index_units` | 77 | 28 | 1339 | 3806 |
| `context_units` | 77 | 27 | 69 | 90 |
| `context_tokens` | 77 | 958 | 2625 | 3010 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 11 | 2.3 ms | 5.1 ms | 1.6 ms | 4.7 ms | 3.8 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 4.5 ms | 114.2 ms | 4.0 ms | 101.0 ms | 57.9 ms | 41.3 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 815.5 ms | 815.5 ms | 815.1 ms | 815.1 ms | 2.0 ms | 0.32 ms | 72.9 ms | 709.4 ms |
| ocr | 19 | 2.09 s | 45.91 s | 2.09 s | 45.91 s | 31.2 ms | 1.7 ms | 4.36 s | 90.02 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 3.9 ms | 2.57 s | 3.5 ms | 2.57 s | 23.7 ms | 0.85 ms | 113.3 ms | 2.43 s |
| 2-4 | 24 | 4.8 ms | 2.87 s | 4.3 ms | 2.87 s | 4.6 ms | 1.3 ms | 205.2 ms | 4.62 s |
| 5-9 | 4 | 9.4 ms | 13.1 ms | 8.2 ms | 12.2 ms | 9.6 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 19.2 ms | 21.25 s | 17.5 ms | 21.25 s | 14.4 ms | 5.8 ms | 1.47 s | 41.87 s |
| 25-49 | 4 | 50.4 ms | 45.91 s | 46.2 ms | 45.91 s | 50.5 ms | 35.5 ms | 4.36 s | 90.02 s |
| 50-99 | 2 | 56.5 ms | 146.5 ms | 52.4 ms | 131.2 ms | 75.9 ms | 53.6 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 114.2 ms | 219.0 ms | 101.0 ms | 192.5 ms | 106.5 ms | 83.9 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 815.5 ms | 1.33 s | 815.1 ms | 1.33 s | 3.7 ms | 0.96 ms | 108.2 ms | 1.20 s |
| annual_report | 2 | 9.4 ms | 114.2 ms | 8.2 ms | 101.0 ms | 57.9 ms | 41.3 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.57 s | 2.57 s | 2.57 s | 2.57 s | 0.25 ms | 0.34 ms | 113.4 ms | 2.43 s |
| certificate | 1 | 2.93 s | 2.93 s | 2.93 s | 2.93 s | 0.13 ms | 0.35 ms | 102.6 ms | 2.80 s |
| claim | 1 | 2.87 s | 2.87 s | 2.87 s | 2.87 s | 0.16 ms | 0.33 ms | 205.2 ms | 5.27 s |
| condition_report | 1 | 4.3 ms | 4.3 ms | 3.8 ms | 3.8 ms | 2.7 ms | 0.68 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 22.8 ms | 45.91 s | 20.8 ms | 45.91 s | 50.5 ms | 35.5 ms | 4.36 s | 90.02 s |
| email | 1 | 0.83 ms | 0.83 ms | 0.42 ms | 0.42 ms | 0.14 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 4.2 ms | 2.18 s | 3.4 ms | 2.17 s | 23.7 ms | 0.74 ms | 0.00 ms | 2.04 s |
| inspection_report | 1 | 6.2 ms | 6.2 ms | 5.5 ms | 5.5 ms | 3.4 ms | 1.6 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 4.8 ms | 146.5 ms | 4.3 ms | 131.2 ms | 75.9 ms | 53.6 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 3.6 ms | 1.77 s | 3.2 ms | 1.77 s | 18.3 ms | 0.57 ms | 112.1 ms | 1.63 s |
| letter | 5 | 4.1 ms | 1.78 s | 3.7 ms | 1.78 s | 22.7 ms | 0.63 ms | 0.00 ms | 1.71 s |
| log | 1 | 2.5 ms | 2.5 ms | 1.9 ms | 1.9 ms | 1.5 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 4.0 ms | 4.0 ms | 3.6 ms | 3.6 ms | 2.4 ms | 0.75 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 5.1 ms | 5.1 ms | 4.7 ms | 4.7 ms | 3.8 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 4.1 ms | 4.1 ms | 3.6 ms | 3.6 ms | 2.4 ms | 0.85 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 4.1 ms | 2.09 s | 3.6 ms | 2.09 s | 21.6 ms | 0.67 ms | 105.7 ms | 1.96 s |
| payroll | 1 | 2.4 ms | 2.4 ms | 1.6 ms | 1.6 ms | 1.3 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 2.0 ms | 3.1 ms | 1.6 ms | 2.7 ms | 2.3 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 3.3 ms | 3.3 ms | 2.8 ms | 2.8 ms | 1.7 ms | 0.78 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 3.7 ms | 1.97 s | 3.1 ms | 1.97 s | 2.0 ms | 0.72 ms | 111.6 ms | 1.83 s |
| quotation | 1 | 2.71 s | 2.71 s | 2.71 s | 2.71 s | 26.3 ms | 0.18 ms | 0.00 ms | 2.61 s |
| rate_confirmation | 1 | 3.5 ms | 3.5 ms | 3.0 ms | 3.0 ms | 2.1 ms | 0.50 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.01 s | 1.01 s | 1.01 s | 1.01 s | 7.5 ms | 0.10 ms | 0.00 ms | 992.6 ms |
| remittance_advice | 1 | 2.09 s | 2.09 s | 2.09 s | 2.09 s | 31.2 ms | 0.20 ms | 0.00 ms | 2.04 s |
| report | 2 | 1.4 ms | 219.0 ms | 0.78 ms | 192.5 ms | 106.5 ms | 83.9 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 2.3 ms | 2.3 ms | 1.7 ms | 1.7 ms | 1.4 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 11.2 ms | 11.2 ms | 10.2 ms | 10.2 ms | 6.6 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 4.8 ms | 1.73 s | 4.1 ms | 1.73 s | 2.4 ms | 1.2 ms | 113.3 ms | 1.60 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1272 MB | scan-lease-25p |
| Model server | 1576 MB | 1579 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (10)

- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Invoice.pdf` · review: PARTY_MISSING, LOW_CONFIDENCE
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order - Lamplighter Robotics Inc.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Statement from Meadowlark Health Plan.pdf` · review: LOW_CONFIDENCE
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `Trade Credit Application for Briarport Coffee Roasters LLC.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Demand for Payment - Fernvale Cider Works LLC.docx` · review: PARTY_UNSUPPORTED
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Accounts Payable Aging Summary.csv` · review: LOW_CONFIDENCE
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement of Account from Copper Flats Dental Group PLLC.pdf` · **filed without review**
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading.pdf` · review: PARTY_MISSING, LOW_CONFIDENCE
- **msa-effective-date-in-definitions-12p**: expected `2026-03-02 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf`, got `2026-03-11 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf` · trap: date 2026-03-11 (client signature date) · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
