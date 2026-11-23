# InternBench: live run

- **Run:** 2026-11-23T17:38:08Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 4240.47 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 74/77 (96.1%) |
| Document type | 75/77 (97.4%) |
| Date | 76/77 (98.7%) |
| Date role (when the reviewed date was chosen) | 57/74 (77.0%) |
| Parties | 77/77 (100.0%) |
| Relation word (when parties were named) | 77/77 (100.0%) |
| Party in the right role (when parties were named) | 77/77 (100.0%) |
| Ready / review routing | 30/57 (52.6%) |
| Description has every fact | 23/77 (29.9%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 76/77 (98.7%) |
| Description fact coverage | 55.6% (mean of 77) |
| Description specificity | 97.0% (mean of 77) |
| Evidence recall (model's quotes) | 92.6% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 50.6% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 91.7% | 12 |
| Complex-document filename accuracy | 96.4% | 56 |
| Description completeness | 55.6% | 77 |
| Unsupported-fact rate (documents) | 7.8% | 77 |
| Review rate | 50.6% | 77 |
| Evidence recall | 92.6% | 77 |
| Total latency p50 | 49.28 s | 77 |
| Total latency p95 | 97.89 s | 77 |
| Generation latency p50 | 21.82 s | 77 |
| Generation latency p95 | 25.72 s | 77 |
| Generated tokens p50 | 90 | 77 |
| Generated tokens p95 | 105 | 77 |
| Prompt tokens p50 | 978 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 77 |
| Trap date chosen | 1 of 77 |
| Forbidden party named | 0 of 77 |
| Spurious parties named | 0 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 296 (0.0%) |
| Right name sent to review anyway | 24 of 53 |
| Sent to review for an unsupported fact | 6 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 0.0% | 56.73 s | 60.20 s |
| annual_report | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 48.16 s | 83.22 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 49.52 s | 49.52 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 53.00 s | 53.00 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 43.49 s | 43.49 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 46.80 s | 46.80 s |
| contract | 16 | 16/16 | 16/16 | 16/16 | 9/12 | 18.8% | 70.88 s | 106.44 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 40.93 s | 40.93 s |
| form | 5 | 5/5 | 5/5 | 5/5 | 1/4 | 80.0% | 42.16 s | 56.98 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 86.49 s | 86.49 s |
| insurance | 2 | 1/2 | 1/2 | 2/2 | 0/2 | 100.0% | 57.19 s | 109.18 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 1/5 | 85.7% | 37.29 s | 44.72 s |
| letter | 5 | 4/5 | 5/5 | 5/5 | 0/4 | 100.0% | 47.05 s | 53.15 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 79.70 s | 79.70 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 50.19 s | 50.19 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 59.27 s | 59.27 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 45.49 s | 45.49 s |
| notice | 10 | 10/10 | 10/10 | 10/10 | 3/8 | 60.0% | 37.90 s | 58.84 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 48.48 s | 48.48 s |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 44.55 s | 63.49 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 50.83 s | 50.83 s |
| purchase_order | 2 | 2/2 | 2/2 | 2/2 | 0/1 | 100.0% | 34.67 s | 50.62 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 44.27 s | 44.27 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 46.53 s | 46.53 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 41.08 s | 41.08 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 44.38 s | 44.38 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 64.35 s | 89.58 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 57.91 s | 57.91 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 63.01 s | 63.01 s |
| statement | 3 | 3/3 | 3/3 | 3/3 | 2/3 | 66.7% | 54.83 s | 85.73 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 40.93 s | 40.93 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 0.0% | 46.85 s | 56.73 s |
| native | 47 | 45/47 | 46/47 | 47/47 | 23/44 | 48.9% | 50.83 s | 98.68 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 34.03 s | 34.03 s |
| office | 5 | 5/5 | 5/5 | 5/5 | 3/5 | 40.0% | 57.91 s | 63.49 s |
| scan | 16 | 15/16 | 16/16 | 16/16 | 3/4 | 62.5% | 42.16 s | 90.11 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 1/2 | 33.3% | 64.35 s | 79.70 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 43.85 s | 43.85 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 36/37 | 37/37 | 37/37 | 11/26 | 70.3% | 41.57 s | 58.32 s |
| 2-4 | 24 | 23/24 | 24/24 | 24/24 | 9/17 | 37.5% | 52.48 s | 86.49 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 3/4 | 25.0% | 48.16 s | 63.01 s |
| 10-24 | 4 | 4/4 | 4/4 | 4/4 | 2/3 | 25.0% | 67.17 s | 80.61 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 2/3 | 25.0% | 84.17 s | 106.44 s |
| 50-99 | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 97.89 s | 109.18 s |
| 100+ | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 83.22 s | 89.58 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 11/12 | 11/12 | 12/12 | 7/10 | 0 of 12 | 83.22 s | 109.18 s | 22.13 s | 25.72 s |
| complex | 56 | 54/56 | 55/56 | 56/56 | 23/46 | 0 of 56 | 50.44 s | 97.89 s | 21.77 s | 25.72 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 65/67 | 66/67 | 67/67 | 1 |
| complex_pdf | 5 | 5/5 | 5/5 | 5/5 | 0 |
| contract | 20 | 20/20 | 20/20 | 20/20 | 0 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 6/7 | 6/7 | 7/7 | 1 |
| docx | 3 | 3/3 | 3/3 | 3/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 8/10 | 9/10 | 10/10 | 1 |
| form | 7 | 7/7 | 7/7 | 7/7 | 0 |
| healthcare | 3 | 3/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 15/16 | 16/16 | 16/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 9/10 | 9/10 | 10/10 | 1 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 16/17 | 16/17 | 17/17 | 1 |
| key_value | 11 | 11/11 | 11/11 | 11/11 | 0 |
| layout_parties | 11 | 11/11 | 11/11 | 11/11 | 0 |
| letter | 5 | 4/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 5/6 | 5/6 | 6/6 | 1 |
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
| pages_50 | 2 | 1/2 | 1/2 | 2/2 | 1 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 2/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 2/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 19/20 | 20/20 | 20/20 | 0 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 4/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 40/41 | 40/41 | 41/41 | 1 |
| tiff | 2 | 2/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 4/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 3.62 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.07 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.18 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.55 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.54 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 918.4 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.41 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.44 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 32.99 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 69.22 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.20 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 1.80 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 4.46 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.65 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 1.70 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.09 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 663.0 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.28 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.06 s |
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
| `total_ms` | 77 | 49.28 s | 97.89 s | 109.18 s |
| `extraction_wall_ms` | 77 | 3.6 ms | 2.39 s | 35.26 s |
| `worker_total_ms` | 77 | 3.2 ms | 2.39 s | 35.26 s |
| `worker_snapshot_ms` | 77 | 0.18 ms | 0.32 ms | 1.1 ms |
| `worker_parse_ms` | 77 | 1.7 ms | 35.5 ms | 74.0 ms |
| `worker_analysis_ms` | 77 | 0.47 ms | 27.8 ms | 65.0 ms |
| `worker_render_ms` | 77 | 0.00 ms | 153.1 ms | 3.19 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 53.1 ms | 73.3 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 3.62 s | 69.22 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 229.5 ms | 4.94 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 2.97 s | 63.99 s |
| `worker_vision_ms` | 77 | 0.00 ms | 5.3 ms | 10.4 ms |
| `analyze_wall_ms` | 77 | 48.48 s | 97.85 s | 109.07 s |
| `distill_ms` | 77 | 1.9 ms | 65.0 ms | 196.6 ms |
| `prompt_ms` | 77 | 0.03 ms | 0.06 ms | 0.07 ms |
| `inference_ms` | 77 | 48.47 s | 97.76 s | 108.94 s |
| `prefill_ms` | 77 | 26.29 s | 72.73 s | 86.80 s |
| `generation_ms` | 77 | 21.82 s | 25.72 s | 29.67 s |
| `validation_ms` | 77 | 3.5 ms | 29.7 ms | 52.5 ms |
| `naming_ms` | 77 | 0.22 ms | 5.4 ms | 8.6 ms |
| `index_ms` | 77 | 1.6 ms | 61.4 ms | 188.5 ms |
| `retrieval_ms` | 77 | 0.03 ms | 4.4 ms | 9.0 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 36.8 tok/s | 37.5 tok/s | 37.8 tok/s |
| `generation_tok_per_s` | 77 | 4.1 tok/s | 4.3 tok/s | 4.3 tok/s |
| `prompt_tokens` | 77 | 978 | 2685 | 3139 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 90 | 105 | 123 |
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
| fast | 11 | 1.5 ms | 3.4 ms | 0.99 ms | 2.9 ms | 2.2 ms | 0.33 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 3.3 ms | 85.4 ms | 2.8 ms | 75.4 ms | 40.8 ms | 32.9 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 726.0 ms | 726.0 ms | 725.6 ms | 725.6 ms | 1.5 ms | 0.28 ms | 44.9 ms | 663.0 ms |
| ocr | 19 | 1.73 s | 35.26 s | 1.73 s | 35.26 s | 15.8 ms | 1.6 ms | 3.19 s | 69.22 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 2.9 ms | 2.15 s | 2.5 ms | 2.15 s | 13.7 ms | 0.70 ms | 78.8 ms | 2.06 s |
| 2-4 | 24 | 3.6 ms | 2.39 s | 3.1 ms | 2.39 s | 2.8 ms | 1.1 ms | 153.1 ms | 3.62 s |
| 5-9 | 4 | 6.8 ms | 9.8 ms | 6.1 ms | 9.0 ms | 6.7 ms | 2.5 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 13.9 ms | 16.70 s | 12.8 ms | 16.70 s | 10.4 ms | 4.7 ms | 1.13 s | 32.99 s |
| 25-49 | 4 | 37.7 ms | 35.26 s | 34.1 ms | 35.26 s | 35.5 ms | 27.8 ms | 3.19 s | 69.22 s |
| 50-99 | 2 | 41.3 ms | 109.7 ms | 38.1 ms | 96.3 ms | 53.1 ms | 41.6 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 85.4 ms | 162.7 ms | 75.4 ms | 140.9 ms | 74.0 ms | 65.0 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 726.0 ms | 1.16 s | 725.6 ms | 1.16 s | 2.2 ms | 0.78 ms | 75.2 ms | 1.07 s |
| annual_report | 2 | 6.8 ms | 85.4 ms | 6.1 ms | 75.4 ms | 40.8 ms | 32.9 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.15 s | 2.15 s | 2.15 s | 2.15 s | 0.18 ms | 0.34 ms | 78.0 ms | 2.06 s |
| certificate | 1 | 2.37 s | 2.37 s | 2.37 s | 2.37 s | 0.17 ms | 0.32 ms | 71.6 ms | 2.28 s |
| claim | 1 | 2.39 s | 2.39 s | 2.39 s | 2.39 s | 0.20 ms | 0.31 ms | 153.1 ms | 4.46 s |
| condition_report | 1 | 2.8 ms | 2.8 ms | 2.4 ms | 2.4 ms | 1.5 ms | 0.56 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 17.2 ms | 35.26 s | 15.7 ms | 35.26 s | 35.5 ms | 27.8 ms | 3.19 s | 69.22 s |
| email | 1 | 0.82 ms | 0.82 ms | 0.41 ms | 0.41 ms | 0.16 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 3.1 ms | 1.89 s | 2.6 ms | 1.88 s | 14.6 ms | 0.62 ms | 0.00 ms | 1.80 s |
| inspection_report | 1 | 4.1 ms | 4.1 ms | 3.6 ms | 3.6 ms | 2.1 ms | 1.3 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 3.3 ms | 109.7 ms | 2.9 ms | 96.3 ms | 53.1 ms | 41.6 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 2.5 ms | 1.51 s | 2.0 ms | 1.51 s | 9.7 ms | 0.47 ms | 78.8 ms | 1.42 s |
| letter | 5 | 3.3 ms | 1.49 s | 2.7 ms | 1.49 s | 12.7 ms | 0.56 ms | 0.00 ms | 1.44 s |
| log | 1 | 1.6 ms | 1.6 ms | 0.93 ms | 0.93 ms | 0.71 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 3.0 ms | 3.0 ms | 2.7 ms | 2.7 ms | 1.7 ms | 0.62 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 3.4 ms | 3.4 ms | 2.9 ms | 2.9 ms | 2.2 ms | 0.33 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 3.1 ms | 3.1 ms | 2.8 ms | 2.8 ms | 1.8 ms | 0.70 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 3.2 ms | 1.74 s | 2.5 ms | 1.74 s | 12.8 ms | 0.58 ms | 73.8 ms | 1.65 s |
| payroll | 1 | 1.8 ms | 1.8 ms | 1.2 ms | 1.2 ms | 0.92 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 1.5 ms | 2.1 ms | 1.1 ms | 1.5 ms | 1.3 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 2.7 ms | 2.7 ms | 2.3 ms | 2.3 ms | 1.4 ms | 0.62 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 2.9 ms | 1.64 s | 2.5 ms | 1.64 s | 1.5 ms | 0.61 ms | 78.0 ms | 1.55 s |
| quotation | 1 | 2.28 s | 2.28 s | 2.28 s | 2.28 s | 15.8 ms | 0.19 ms | 0.00 ms | 2.20 s |
| rate_confirmation | 1 | 2.7 ms | 2.7 ms | 2.3 ms | 2.3 ms | 1.5 ms | 0.45 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 928.4 ms | 928.4 ms | 928.0 ms | 928.0 ms | 6.4 ms | 0.11 ms | 0.00 ms | 918.4 ms |
| remittance_advice | 1 | 1.73 s | 1.73 s | 1.73 s | 1.73 s | 13.7 ms | 0.19 ms | 0.00 ms | 1.70 s |
| report | 2 | 0.99 ms | 162.7 ms | 0.45 ms | 140.9 ms | 74.0 ms | 65.0 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 1.5 ms | 1.5 ms | 0.94 ms | 0.94 ms | 0.71 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 8.6 ms | 8.6 ms | 7.8 ms | 7.8 ms | 4.7 ms | 2.5 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 3.6 ms | 1.50 s | 3.1 ms | 1.50 s | 1.8 ms | 1.0 ms | 82.8 ms | 1.41 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1329 MB | scan-lease-25p |
| Model server | 6083 MB | 6085 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (3)

- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_MISSING
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-06-24 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf` · trap: date 2026-06-24 (countersignature date) · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
