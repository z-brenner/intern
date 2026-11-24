# InternBench: live run

- **Run:** 2026-11-23T16:51:04Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 2602.00 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 70/77 (90.9%) |
| Document type | 73/77 (94.8%) |
| Date | 77/77 (100.0%) |
| Date role (when the reviewed date was chosen) | 49/76 (64.5%) |
| Parties | 76/77 (98.7%) |
| Relation word (when parties were named) | 76/77 (98.7%) |
| Party in the right role (when parties were named) | 75/76 (98.7%) |
| Ready / review routing | 19/57 (33.3%) |
| Description has every fact | 24/77 (31.2%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 77/77 (100.0%) |
| Description fact coverage | 53.5% (mean of 77) |
| Description specificity | 98.3% (mean of 77) |
| Evidence recall (model's quotes) | 93.5% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 63.6% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 100.0% | 12 |
| Complex-document filename accuracy | 92.9% | 56 |
| Description completeness | 53.5% | 77 |
| Unsupported-fact rate (documents) | 18.2% | 77 |
| Review rate | 63.6% | 77 |
| Evidence recall | 93.5% | 77 |
| Total latency p50 | 29.91 s | 77 |
| Total latency p95 | 58.36 s | 77 |
| Generation latency p50 | 13.03 s | 77 |
| Generation latency p95 | 15.45 s | 77 |
| Generated tokens p50 | 89 | 77 |
| Generated tokens p95 | 106 | 77 |
| Prompt tokens p50 | 978 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 77 |
| Trap date chosen | 0 of 77 |
| Forbidden party named | 0 of 77 |
| Spurious parties named | 1 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 299 (0.0%) |
| Right name sent to review anyway | 31 of 53 |
| Sent to review for an unsupported fact | 14 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0/1 | 100.0% | 35.19 s | 36.19 s |
| annual_report | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 29.51 s | 50.95 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 30.56 s | 30.56 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 31.10 s | 31.10 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 26.59 s | 26.59 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 29.63 s | 29.63 s |
| contract | 16 | 16/16 | 16/16 | 16/16 | 6/12 | 37.5% | 44.42 s | 67.23 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 24.86 s | 24.86 s |
| form | 5 | 4/5 | 5/5 | 4/5 | 1/4 | 80.0% | 26.60 s | 31.56 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 54.46 s | 54.46 s |
| insurance | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 37.83 s | 68.53 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 1/5 | 71.4% | 23.52 s | 24.73 s |
| letter | 5 | 2/5 | 5/5 | 5/5 | 1/4 | 60.0% | 27.38 s | 32.36 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 47.60 s | 47.60 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 31.50 s | 31.50 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 40.20 s | 40.20 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 28.23 s | 28.23 s |
| notice | 10 | 10/10 | 10/10 | 10/10 | 3/8 | 60.0% | 24.25 s | 36.32 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 30.66 s | 30.66 s |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 25.79 s | 39.60 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 34.46 s | 34.46 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 50.0% | 22.14 s | 32.12 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 27.26 s | 27.26 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 27.12 s | 27.12 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 21.42 s | 21.42 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 25.39 s | 25.39 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 39.12 s | 54.71 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 34.65 s | 34.65 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 37.72 s | 37.72 s |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0/3 | 66.7% | 31.87 s | 50.18 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 24.86 s | 24.86 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 66.7% | 28.48 s | 35.19 s |
| native | 47 | 43/47 | 47/47 | 46/47 | 15/44 | 68.1% | 31.56 s | 58.36 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 22.38 s | 22.38 s |
| office | 5 | 4/5 | 5/5 | 5/5 | 0/5 | 100.0% | 34.65 s | 39.60 s |
| scan | 16 | 14/16 | 16/16 | 16/16 | 2/4 | 43.8% | 25.39 s | 67.23 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 1/2 | 33.3% | 39.12 s | 47.60 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 27.38 s | 27.38 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 32/37 | 37/37 | 36/37 | 5/26 | 78.4% | 24.73 s | 35.01 s |
| 2-4 | 24 | 22/24 | 24/24 | 24/24 | 9/17 | 45.8% | 30.77 s | 54.46 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 0/4 | 100.0% | 29.51 s | 37.72 s |
| 10-24 | 4 | 4/4 | 4/4 | 4/4 | 0/3 | 75.0% | 44.42 s | 49.27 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 51.38 s | 67.23 s |
| 50-99 | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 57.89 s | 68.53 s |
| 100+ | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 50.95 s | 54.71 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 12/12 | 12/12 | 12/12 | 5/10 | 0 of 12 | 50.95 s | 68.53 s | 13.33 s | 15.11 s |
| complex | 56 | 52/56 | 56/56 | 55/56 | 15/46 | 0 of 56 | 30.66 s | 63.81 s | 13.27 s | 15.69 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 62/67 | 67/67 | 66/67 | 0 |
| complex_pdf | 5 | 5/5 | 5/5 | 5/5 | 0 |
| contract | 20 | 20/20 | 20/20 | 20/20 | 0 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 6/7 | 7/7 | 7/7 | 0 |
| docx | 3 | 2/3 | 3/3 | 3/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 9/10 | 10/10 | 10/10 | 0 |
| form | 7 | 6/7 | 7/7 | 6/7 | 0 |
| healthcare | 3 | 3/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 14/16 | 16/16 | 16/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 10/10 | 10/10 | 10/10 | 0 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 17/17 | 17/17 | 17/17 | 0 |
| key_value | 11 | 11/11 | 11/11 | 11/11 | 0 |
| layout_parties | 11 | 9/11 | 11/11 | 10/11 | 0 |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 6/6 | 6/6 | 6/6 | 0 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 7/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 10/10 | 10/10 | 10/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 2/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 18/20 | 20/20 | 20/20 | 0 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 4/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 39/41 | 41/41 | 41/41 | 0 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 4/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 3.69 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.07 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.18 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.55 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.56 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 910.3 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.41 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.46 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 31.32 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 68.15 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.21 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 1.81 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 4.18 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.65 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 1.81 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.13 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 678.5 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.32 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.08 s |
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
| `total_ms` | 77 | 29.91 s | 58.36 s | 68.53 s |
| `extraction_wall_ms` | 77 | 3.8 ms | 2.41 s | 34.73 s |
| `worker_total_ms` | 77 | 3.2 ms | 2.41 s | 34.73 s |
| `worker_snapshot_ms` | 77 | 0.18 ms | 0.37 ms | 1.1 ms |
| `worker_parse_ms` | 77 | 1.7 ms | 40.4 ms | 2.12 s |
| `worker_analysis_ms` | 77 | 0.47 ms | 27.7 ms | 65.5 ms |
| `worker_render_ms` | 77 | 0.00 ms | 161.0 ms | 3.20 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 52.9 ms | 78.0 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 3.69 s | 68.15 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 297.7 ms | 5.11 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 3.11 s | 62.79 s |
| `worker_vision_ms` | 77 | 0.00 ms | 5.4 ms | 10.4 ms |
| `analyze_wall_ms` | 77 | 29.37 s | 57.85 s | 68.42 s |
| `distill_ms` | 77 | 1.8 ms | 65.2 ms | 200.6 ms |
| `prompt_ms` | 77 | 0.03 ms | 0.06 ms | 0.07 ms |
| `inference_ms` | 77 | 29.36 s | 57.76 s | 68.26 s |
| `prefill_ms` | 77 | 16.09 s | 44.63 s | 54.02 s |
| `generation_ms` | 77 | 13.03 s | 15.45 s | 17.50 s |
| `validation_ms` | 77 | 3.5 ms | 31.1 ms | 57.3 ms |
| `naming_ms` | 77 | 0.23 ms | 5.4 ms | 8.7 ms |
| `index_ms` | 77 | 1.6 ms | 61.1 ms | 189.9 ms |
| `retrieval_ms` | 77 | 0.03 ms | 4.4 ms | 9.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 60.2 tok/s | 61.7 tok/s | 63.0 tok/s |
| `generation_tok_per_s` | 77 | 6.9 tok/s | 7.1 tok/s | 7.2 tok/s |
| `prompt_tokens` | 77 | 978 | 2685 | 3139 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 89 | 106 | 117 |
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
| fast | 11 | 1.5 ms | 3.4 ms | 1.2 ms | 3.1 ms | 2.3 ms | 0.32 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 3.4 ms | 108.7 ms | 2.9 ms | 95.8 ms | 52.7 ms | 32.8 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 745.3 ms | 745.3 ms | 744.9 ms | 744.9 ms | 1.7 ms | 0.32 ms | 47.9 ms | 678.5 ms |
| ocr | 19 | 1.74 s | 34.73 s | 1.74 s | 34.73 s | 16.0 ms | 1.6 ms | 3.20 s | 68.15 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 3.0 ms | 2.18 s | 2.5 ms | 2.18 s | 14.7 ms | 0.67 ms | 79.6 ms | 2.08 s |
| 2-4 | 24 | 3.6 ms | 2.31 s | 3.1 ms | 2.31 s | 2.9 ms | 1.1 ms | 161.0 ms | 3.69 s |
| 5-9 | 4 | 7.5 ms | 9.7 ms | 6.9 ms | 9.0 ms | 6.7 ms | 2.6 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 13.7 ms | 15.85 s | 12.5 ms | 15.84 s | 10.2 ms | 4.7 ms | 1.08 s | 31.32 s |
| 25-49 | 4 | 37.4 ms | 34.73 s | 33.9 ms | 34.73 s | 35.5 ms | 27.7 ms | 3.20 s | 68.15 s |
| 50-99 | 2 | 42.0 ms | 108.7 ms | 38.4 ms | 95.8 ms | 52.7 ms | 41.5 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 85.4 ms | 164.9 ms | 75.0 ms | 142.0 ms | 74.4 ms | 65.5 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 745.3 ms | 1.16 s | 744.9 ms | 1.16 s | 2.3 ms | 0.76 ms | 75.4 ms | 1.07 s |
| annual_report | 2 | 7.5 ms | 85.4 ms | 6.9 ms | 75.0 ms | 40.4 ms | 32.8 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.18 s | 2.18 s | 2.18 s | 2.18 s | 0.15 ms | 0.34 ms | 78.1 ms | 2.08 s |
| certificate | 1 | 2.41 s | 2.41 s | 2.41 s | 2.41 s | 0.16 ms | 0.33 ms | 71.7 ms | 2.32 s |
| claim | 1 | 2.31 s | 2.31 s | 2.31 s | 2.31 s | 0.15 ms | 0.29 ms | 161.0 ms | 4.18 s |
| condition_report | 1 | 2.8 ms | 2.8 ms | 2.4 ms | 2.4 ms | 1.5 ms | 0.57 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 17.1 ms | 34.73 s | 15.5 ms | 34.73 s | 35.5 ms | 27.7 ms | 3.20 s | 68.15 s |
| email | 1 | 0.68 ms | 0.68 ms | 0.37 ms | 0.37 ms | 0.13 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 3.2 ms | 1.91 s | 2.7 ms | 1.91 s | 12.9 ms | 0.62 ms | 0.00 ms | 1.81 s |
| inspection_report | 1 | 4.4 ms | 4.4 ms | 3.8 ms | 3.8 ms | 2.2 ms | 1.2 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 3.4 ms | 108.7 ms | 2.9 ms | 95.8 ms | 52.7 ms | 41.5 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 2.4 ms | 1.49 s | 2.1 ms | 1.49 s | 8.8 ms | 0.48 ms | 79.6 ms | 1.40 s |
| letter | 5 | 3.1 ms | 1.50 s | 2.7 ms | 1.50 s | 12.4 ms | 0.57 ms | 0.00 ms | 1.46 s |
| log | 1 | 1.5 ms | 1.5 ms | 0.90 ms | 0.90 ms | 0.67 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 3.0 ms | 3.0 ms | 2.6 ms | 2.6 ms | 1.7 ms | 0.61 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 3.4 ms | 3.4 ms | 3.1 ms | 3.1 ms | 2.3 ms | 0.32 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 3.3 ms | 3.3 ms | 2.9 ms | 2.9 ms | 1.8 ms | 0.74 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 3.1 ms | 2.12 s | 2.5 ms | 2.12 s | 2.12 s | 0.58 ms | 76.4 ms | 1.65 s |
| payroll | 1 | 1.8 ms | 1.8 ms | 1.2 ms | 1.2 ms | 0.95 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 1.9 ms | 1.9 ms | 1.5 ms | 1.5 ms | 1.3 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 2.8 ms | 2.8 ms | 2.2 ms | 2.2 ms | 1.3 ms | 0.64 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 2.9 ms | 1.64 s | 2.5 ms | 1.64 s | 1.5 ms | 0.61 ms | 78.9 ms | 1.55 s |
| quotation | 1 | 2.29 s | 2.29 s | 2.29 s | 2.29 s | 16.0 ms | 0.19 ms | 0.00 ms | 2.21 s |
| rate_confirmation | 1 | 2.8 ms | 2.8 ms | 2.4 ms | 2.4 ms | 1.7 ms | 0.45 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 920.4 ms | 920.4 ms | 920.0 ms | 920.0 ms | 6.3 ms | 0.10 ms | 0.00 ms | 910.3 ms |
| remittance_advice | 1 | 1.84 s | 1.84 s | 1.84 s | 1.84 s | 14.7 ms | 0.21 ms | 0.00 ms | 1.81 s |
| report | 2 | 1.0 ms | 164.9 ms | 0.57 ms | 142.0 ms | 74.4 ms | 65.5 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 1.5 ms | 1.5 ms | 1.2 ms | 1.2 ms | 0.94 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 8.7 ms | 8.7 ms | 7.8 ms | 7.8 ms | 4.8 ms | 2.6 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 3.6 ms | 1.51 s | 3.2 ms | 1.51 s | 1.8 ms | 1.0 ms | 88.9 ms | 1.41 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1264 MB | scan-lease-25p |
| Model server | 3260 MB | 3262 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (7)

- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order - Kestrel Instruments Ltd.pdf` · review: PARTY_UNSUPPORTED
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Letter to Briarport Coffee Roasters LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Vendor Registration Form for Emberglow Industrial Finishes.pdf` · review: LOW_CONFIDENCE
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Notice for Fernvale Cider Works LLC.docx` · review: TYPE_UNSUPPORTED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quote for propeller overhaul from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
