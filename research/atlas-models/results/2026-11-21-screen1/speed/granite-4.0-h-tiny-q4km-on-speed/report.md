# InternBench: live run

- **Run:** 2026-11-21T07:33:40Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 1810.92 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 54/77 (70.1%) |
| Document type | 71/77 (92.2%) |
| Date | 68/77 (88.3%) |
| Date role (when the reviewed date was chosen) | 38/64 (59.4%) |
| Parties | 71/77 (92.2%) |
| Relation word (when parties were named) | 74/76 (97.4%) |
| Party in the right role (when parties were named) | 69/75 (92.0%) |
| Ready / review routing | 21/57 (36.8%) |
| Description has every fact | 26/77 (33.8%) |
| Description states nothing false | 74/77 (96.1%) |
| Description is specific | 76/77 (98.7%) |
| Description fact coverage | 55.0% (mean of 77) |
| Description specificity | 96.1% (mean of 77) |
| Evidence recall (model's quotes) | 46.8% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 72.7% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 58.3% | 12 |
| Complex-document filename accuracy | 67.9% | 56 |
| Description completeness | 55.0% | 77 |
| Unsupported-fact rate (documents) | 23.4% | 77 |
| Review rate | 72.7% | 77 |
| Evidence recall | 46.8% | 77 |
| Total latency p50 | 20.16 s | 77 |
| Total latency p95 | 47.09 s | 77 |
| Generation latency p50 | 4.43 s | 77 |
| Generation latency p95 | 5.45 s | 77 |
| Generated tokens p50 | 93 | 77 |
| Generated tokens p95 | 115 | 77 |
| Prompt tokens p50 | 889 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 4 of 77 |
| Trap date chosen | 8 of 77 |
| Forbidden party named | 2 of 77 |
| Spurious parties named | 5 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 3 of 316 (0.9%) |
| Right name sent to review anyway | 22 of 53 |
| Sent to review for an unsupported fact | 18 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0/1 | 100.0% | 24.98 s | 26.19 s |
| annual_report | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 20.64 s | 40.42 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 20.80 s | 20.80 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 19.77 s | 19.77 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 18.73 s | 18.73 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 20.90 s | 20.90 s |
| contract | 16 | 13/16 | 15/16 | 15/16 | 3/12 | 62.5% | 33.85 s | 71.03 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 16.02 s | 16.02 s |
| form | 5 | 3/5 | 3/5 | 5/5 | 1/4 | 80.0% | 17.21 s | 20.84 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 37.72 s | 37.72 s |
| insurance | 2 | 1/2 | 2/2 | 2/2 | 1/2 | 50.0% | 23.42 s | 49.14 s |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 1/5 | 71.4% | 13.18 s | 15.62 s |
| letter | 5 | 3/5 | 5/5 | 5/5 | 0/4 | 100.0% | 18.70 s | 22.89 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 33.18 s | 33.18 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 20.74 s | 20.74 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 28.06 s | 28.06 s |
| newsletter | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 18.59 s | 18.59 s |
| notice | 10 | 6/10 | 7/10 | 9/10 | 4/8 | 60.0% | 14.22 s | 24.64 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 18.31 s | 18.31 s |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 18.42 s | 27.21 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 20.11 s | 20.11 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 100.0% | 13.89 s | 20.12 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 17.27 s | 17.27 s |
| rate_confirmation | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 17.05 s | 17.05 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 13.64 s | 13.64 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 100.0% | 16.48 s | 16.48 s |
| report | 2 | 0/2 | 2/2 | 1/2 | 1/2 | 50.0% | 28.85 s | 40.10 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 25.48 s | 25.48 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 27.82 s | 27.82 s |
| statement | 3 | 2/3 | 2/3 | 2/3 | 2/3 | 66.7% | 20.03 s | 33.13 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 16.02 s | 16.02 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 66.7% | 20.16 s | 26.19 s |
| native | 47 | 29/47 | 38/47 | 42/47 | 14/44 | 70.2% | 20.74 s | 47.09 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 13.18 s | 13.18 s |
| office | 5 | 5/5 | 5/5 | 5/5 | 1/5 | 80.0% | 25.48 s | 27.21 s |
| scan | 16 | 13/16 | 16/16 | 16/16 | 4/4 | 81.2% | 16.48 s | 71.03 s |
| sheet | 3 | 1/3 | 3/3 | 2/3 | 1/2 | 66.7% | 28.85 s | 33.18 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 17.10 s | 17.10 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 25/37 | 30/37 | 34/37 | 10/26 | 78.4% | 15.62 s | 27.01 s |
| 2-4 | 24 | 19/24 | 23/24 | 23/24 | 8/17 | 62.5% | 20.92 s | 37.72 s |
| 5-9 | 4 | 3/4 | 4/4 | 3/4 | 1/4 | 75.0% | 20.46 s | 27.82 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 1/3 | 75.0% | 33.47 s | 42.47 s |
| 25-49 | 4 | 3/4 | 3/4 | 4/4 | 0/3 | 75.0% | 38.40 s | 71.03 s |
| 50-99 | 2 | 0/2 | 2/2 | 1/2 | 0/2 | 100.0% | 47.09 s | 49.14 s |
| 100+ | 2 | 1/2 | 2/2 | 2/2 | 1/2 | 50.0% | 40.10 s | 40.42 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 7/12 | 11/12 | 11/12 | 2/10 | 1 of 12 | 40.10 s | 71.03 s | 4.46 s | 5.10 s |
| complex | 56 | 38/56 | 48/56 | 51/56 | 15/46 | 4 of 56 | 20.49 s | 48.78 s | 4.45 s | 5.32 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 48/67 | 58/67 | 62/67 | 10 |
| complex_pdf | 5 | 1/5 | 2/5 | 4/5 | 4 |
| contract | 20 | 17/20 | 19/20 | 19/20 | 1 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_table | 7 | 4/7 | 5/7 | 7/7 | 1 |
| docx | 3 | 3/3 | 3/3 | 3/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 5/10 | 10/10 | 7/10 | 1 |
| form | 7 | 5/7 | 5/7 | 7/7 | 1 |
| healthcare | 3 | 2/3 | 2/3 | 2/3 | 1 |
| hr | 4 | 3/4 | 3/4 | 4/4 | 1 |
| image_only_scan | 16 | 13/16 | 16/16 | 16/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 5/10 | 9/10 | 9/10 | 1 |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 1 |
| irrelevant_names | 17 | 10/17 | 15/17 | 15/17 | 2 |
| key_value | 11 | 8/11 | 9/11 | 10/11 | 3 |
| layout_parties | 11 | 7/11 | 9/11 | 10/11 | 2 |
| letter | 5 | 3/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 2/6 | 6/6 | 5/6 | 0 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 3/7 | 3/7 | 7/7 | 4 |
| noisy_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 6/10 | 7/10 | 9/10 | 3 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 2/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 3/4 | 3/4 | 4/4 | 1 |
| pages_5 | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 0/2 | 2/2 | 1/2 | 0 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 2/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 16/20 | 20/20 | 18/20 | 0 |
| rotated_page | 1 | 0/1 | 0/1 | 1/1 | 1 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 1/3 | 3/3 | 2/3 | 0 |
| statement | 3 | 2/3 | 2/3 | 2/3 | 1 |
| stream_order | 3 | 0/3 | 0/3 | 3/3 | 3 |
| table | 41 | 29/41 | 37/41 | 36/41 | 4 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 3/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 1/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.58 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.21 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.31 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.90 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.86 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 976.2 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.61 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.73 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 41.73 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 91.38 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.62 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.11 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.13 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.99 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.06 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.20 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 704.0 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.83 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.42 s |
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
| `total_ms` | 77 | 20.16 s | 47.09 s | 71.03 s |
| `extraction_wall_ms` | 77 | 4.9 ms | 2.96 s | 46.64 s |
| `worker_total_ms` | 77 | 4.4 ms | 2.96 s | 46.64 s |
| `worker_snapshot_ms` | 77 | 0.23 ms | 0.37 ms | 1.4 ms |
| `worker_parse_ms` | 77 | 2.5 ms | 50.8 ms | 105.6 ms |
| `worker_analysis_ms` | 77 | 0.55 ms | 35.7 ms | 83.3 ms |
| `worker_render_ms` | 77 | 0.00 ms | 233.1 ms | 4.58 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 64.5 ms | 106.8 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 4.58 s | 91.38 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 351.6 ms | 5.83 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 3.91 s | 85.19 s |
| `worker_vision_ms` | 77 | 0.00 ms | 5.9 ms | 12.1 ms |
| `analyze_wall_ms` | 77 | 20.02 s | 40.31 s | 48.98 s |
| `distill_ms` | 77 | 2.1 ms | 81.3 ms | 250.0 ms |
| `prompt_ms` | 77 | 0.03 ms | 0.07 ms | 0.08 ms |
| `inference_ms` | 77 | 20.01 s | 40.17 s | 48.79 s |
| `prefill_ms` | 77 | 15.43 s | 36.97 s | 44.51 s |
| `generation_ms` | 77 | 4.43 s | 5.45 s | 6.51 s |
| `validation_ms` | 77 | 5.5 ms | 33.1 ms | 66.5 ms |
| `naming_ms` | 77 | 0.24 ms | 6.0 ms | 9.5 ms |
| `index_ms` | 77 | 1.9 ms | 76.7 ms | 235.6 ms |
| `retrieval_ms` | 77 | 0.03 ms | 4.8 ms | 10.2 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 57.8 tok/s | 59.1 tok/s | 59.8 tok/s |
| `generation_tok_per_s` | 77 | 21.1 tok/s | 22.0 tok/s | 22.3 tok/s |
| `prompt_tokens` | 77 | 889 | 2152 | 2612 |
| `cached_tokens` | 77 | 469 | 469 | 1045 |
| `generated_tokens` | 77 | 93 | 115 | 134 |
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
| fast | 11 | 2.3 ms | 4.4 ms | 1.9 ms | 3.8 ms | 3.0 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 4.6 ms | 114.0 ms | 4.0 ms | 100.9 ms | 57.8 ms | 41.2 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 810.3 ms | 810.3 ms | 809.9 ms | 809.9 ms | 2.1 ms | 0.32 ms | 73.3 ms | 704.0 ms |
| ocr | 19 | 2.11 s | 46.64 s | 2.11 s | 46.64 s | 34.0 ms | 1.8 ms | 4.58 s | 91.38 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 3.9 ms | 2.57 s | 3.4 ms | 2.57 s | 26.0 ms | 0.83 ms | 112.7 ms | 2.42 s |
| 2-4 | 24 | 4.9 ms | 2.88 s | 4.3 ms | 2.88 s | 3.9 ms | 1.3 ms | 233.1 ms | 4.58 s |
| 5-9 | 4 | 9.2 ms | 12.6 ms | 8.2 ms | 11.8 ms | 9.3 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 20.4 ms | 21.20 s | 17.2 ms | 21.20 s | 14.5 ms | 5.8 ms | 1.55 s | 41.73 s |
| 25-49 | 4 | 51.0 ms | 46.64 s | 46.9 ms | 46.64 s | 50.8 ms | 35.7 ms | 4.58 s | 91.38 s |
| 50-99 | 2 | 57.3 ms | 156.1 ms | 53.1 ms | 139.9 ms | 83.2 ms | 55.1 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 114.0 ms | 220.3 ms | 100.9 ms | 191.3 ms | 105.6 ms | 83.3 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 810.3 ms | 1.35 s | 809.9 ms | 1.35 s | 3.4 ms | 0.89 ms | 108.4 ms | 1.21 s |
| annual_report | 2 | 9.2 ms | 114.0 ms | 8.2 ms | 100.9 ms | 57.8 ms | 41.2 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.57 s | 2.57 s | 2.57 s | 2.57 s | 0.17 ms | 0.36 ms | 118.6 ms | 2.42 s |
| certificate | 1 | 2.96 s | 2.96 s | 2.96 s | 2.96 s | 0.17 ms | 0.37 ms | 102.6 ms | 2.83 s |
| claim | 1 | 2.88 s | 2.88 s | 2.88 s | 2.88 s | 0.17 ms | 0.29 ms | 253.1 ms | 5.13 s |
| condition_report | 1 | 3.6 ms | 3.6 ms | 3.3 ms | 3.3 ms | 2.1 ms | 0.65 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 22.7 ms | 46.64 s | 20.9 ms | 46.64 s | 50.8 ms | 35.7 ms | 4.58 s | 91.38 s |
| email | 1 | 0.80 ms | 0.80 ms | 0.47 ms | 0.47 ms | 0.14 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 4.5 ms | 2.23 s | 4.0 ms | 2.23 s | 24.4 ms | 0.74 ms | 0.00 ms | 2.11 s |
| inspection_report | 1 | 6.2 ms | 6.2 ms | 5.6 ms | 5.6 ms | 3.5 ms | 1.6 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 4.9 ms | 156.1 ms | 4.3 ms | 139.9 ms | 83.2 ms | 55.1 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 3.3 ms | 1.75 s | 2.9 ms | 1.75 s | 25.4 ms | 0.55 ms | 112.0 ms | 1.61 s |
| letter | 5 | 3.9 ms | 1.80 s | 3.5 ms | 1.80 s | 26.0 ms | 0.66 ms | 0.00 ms | 1.73 s |
| log | 1 | 2.0 ms | 2.0 ms | 1.4 ms | 1.4 ms | 1.0 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 3.9 ms | 3.9 ms | 3.4 ms | 3.4 ms | 2.2 ms | 0.73 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 4.4 ms | 4.4 ms | 3.8 ms | 3.8 ms | 3.0 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 4.2 ms | 4.2 ms | 3.6 ms | 3.6 ms | 2.4 ms | 0.83 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 4.4 ms | 2.13 s | 3.6 ms | 2.13 s | 22.3 ms | 0.66 ms | 110.9 ms | 1.99 s |
| payroll | 1 | 3.1 ms | 3.1 ms | 2.3 ms | 2.3 ms | 2.0 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 2.6 ms | 2.7 ms | 2.2 ms | 2.2 ms | 1.9 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 4.2 ms | 4.2 ms | 3.1 ms | 3.1 ms | 2.0 ms | 0.75 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 3.6 ms | 2.04 s | 3.2 ms | 2.04 s | 2.0 ms | 0.72 ms | 112.7 ms | 1.90 s |
| quotation | 1 | 2.72 s | 2.72 s | 2.72 s | 2.72 s | 34.0 ms | 0.19 ms | 0.00 ms | 2.62 s |
| rate_confirmation | 1 | 3.4 ms | 3.4 ms | 3.0 ms | 3.0 ms | 2.1 ms | 0.49 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 990.2 ms | 990.2 ms | 989.7 ms | 989.7 ms | 8.8 ms | 0.15 ms | 0.00 ms | 976.2 ms |
| remittance_advice | 1 | 2.11 s | 2.11 s | 2.11 s | 2.11 s | 32.8 ms | 0.23 ms | 0.00 ms | 2.06 s |
| report | 2 | 1.7 ms | 220.3 ms | 0.79 ms | 191.3 ms | 105.6 ms | 83.3 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 2.3 ms | 2.3 ms | 1.9 ms | 1.9 ms | 1.5 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 11.2 ms | 11.2 ms | 10.3 ms | 10.3 ms | 6.6 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 4.9 ms | 1.75 s | 4.1 ms | 1.75 s | 2.7 ms | 1.2 ms | 111.2 ms | 1.61 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1243 MB | scan-lease-25p |
| Model server | 4426 MB | 4430 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (23)

- **notice-of-default**: expected `2025-10-14 Notice of Default for Glasswing Ceramics LLC.pdf`, got `2025-10-14 Notice of Default and Reservation of Rights for Hollis Varga.pdf` · review: LOW_CONFIDENCE
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-14 Explanation of Benefits.pdf` · trap: date 2026-01-14 (end of the claims processing window) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Letter to Briarport Coffee Roasters LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2026-03-12 Annual Report from Casimir Ellingsen.pdf` · trap: party Casimir Ellingsen (chief executive who signs the letter) · review: LOW_CONFIDENCE
- **change-order-form**: expected `2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf`, got `2026-05-14 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf` · review: DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and The Lenders Party Hereto.pdf` · review: LOW_CONFIDENCE
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Accounts Payable Aging Summary for Alderwick Payroll Services Inc.csv` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 Report from Rowanbrae Vineyards.xlsx` · review: TYPE_INFERRED, LOW_CONFIDENCE
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · review: LOW_CONFIDENCE
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quote for propeller overhaul from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · review: LOW_CONFIDENCE
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-10-22 Tidewater Ledger from Saltmarsh Point Homeowners Association.pdf` · trap: date 2026-10-22 (date of the annual meeting) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **meeting-notice-columns**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting) · **filed without review**
- **meeting-notice-interleaved**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting) · **filed without review**
- **meeting-notice-reversed**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting) · **filed without review**
- **rate-confirmation-rotated**: expected `2026-06-08 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf`, got `2026-06-10 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf` · trap: date 2026-06-10 (pickup date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **invoice-boxed-grid**: expected `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf`, got `2026-07-09 Service Invoice from Amara Ellingsworth.pdf` · trap: party Dr. Amara Ellingsworth (customer contact) · review: LOW_CONFIDENCE
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-20 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf` · trap: date 2026-06-20 (date of the qualifying event (marriage)) · review: LOW_CONFIDENCE
- **msa-effective-date-in-definitions-12p**: expected `2026-03-02 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf`, got `2026-03-02 Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **industrial-lease-dated-in-schedule-25p**: expected `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`, got `2026-07-01 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf` · trap: date 2026-07-01 (commencement date) · review: LOW_CONFIDENCE
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · review: LOW_CONFIDENCE
- **watershed-monitoring-report-100p**: expected `2026-06-19 Watershed Monitoring Report for Cobalt Basin Water Authority.pdf`, got `2026-06-19 Annual Watershed Monitoring Report, Monitoring Year 2025 for Cobalt Basin Water Authority.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
