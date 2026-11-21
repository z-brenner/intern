# InternBench: live run

- **Run:** 2026-11-21T04:49:40Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 895.48 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 41/77 (53.2%) |
| Document type | 58/77 (75.3%) |
| Date | 67/77 (87.0%) |
| Date role (when the reviewed date was chosen) | 39/66 (59.1%) |
| Parties | 67/77 (87.0%) |
| Relation word (when parties were named) | 66/76 (86.8%) |
| Party in the right role (when parties were named) | 59/71 (83.1%) |
| Ready / review routing | 4/57 (7.0%) |
| Description has every fact | 17/77 (22.1%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 75/77 (97.4%) |
| Description fact coverage | 48.3% (mean of 77) |
| Description specificity | 93.9% (mean of 77) |
| Evidence recall (model's quotes) | 24.7% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 100.0% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 58.3% | 12 |
| Complex-document filename accuracy | 51.8% | 56 |
| Description completeness | 48.3% | 77 |
| Unsupported-fact rate (documents) | 57.1% | 77 |
| Review rate | 100.0% | 77 |
| Evidence recall | 24.7% | 77 |
| Total latency p50 | 9.55 s | 77 |
| Total latency p95 | 22.68 s | 77 |
| Generation latency p50 | 3.08 s | 77 |
| Generation latency p95 | 4.13 s | 77 |
| Generated tokens p50 | 97 | 77 |
| Generated tokens p95 | 127 | 77 |
| Prompt tokens p50 | 978 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 77 |
| Trap date chosen | 5 of 77 |
| Forbidden party named | 4 of 77 |
| Spurious parties named | 9 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 289 (0.0%) |
| Right name sent to review anyway | 26 of 53 |
| Sent to review for an unsupported fact | 44 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0/1 | 100.0% | 11.27 s | 11.65 s |
| annual_report | 2 | 0/2 | 1/2 | 0/2 | 0/2 | 100.0% | 9.55 s | 18.64 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 11.05 s | 11.05 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 11.53 s | 11.53 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 10.16 s | 10.16 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 8.90 s | 8.90 s |
| contract | 16 | 13/16 | 15/16 | 15/16 | 0/12 | 100.0% | 13.90 s | 55.37 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 7.31 s | 7.31 s |
| form | 5 | 0/5 | 4/5 | 4/5 | 0/4 | 100.0% | 9.42 s | 9.95 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 17.56 s | 17.56 s |
| insurance | 2 | 0/2 | 2/2 | 1/2 | 0/2 | 100.0% | 10.23 s | 23.72 s |
| invoice | 7 | 4/7 | 5/7 | 7/7 | 0/5 | 100.0% | 6.96 s | 7.69 s |
| letter | 5 | 1/5 | 4/5 | 5/5 | 0/4 | 100.0% | 8.27 s | 9.35 s |
| log | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 15.93 s | 15.93 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 10.54 s | 10.54 s |
| minutes | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 11.99 s | 11.99 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 8.59 s | 8.59 s |
| notice | 10 | 9/10 | 10/10 | 10/10 | 0/8 | 100.0% | 7.67 s | 12.11 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 9.74 s | 9.74 s |
| presentation | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 9.18 s | 12.52 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 11.18 s | 11.18 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 100.0% | 7.76 s | 8.96 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 9.10 s | 9.10 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 7.46 s | 7.46 s |
| receipt | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 100.0% | 7.25 s | 7.25 s |
| remittance_advice | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 100.0% | 9.33 s | 9.33 s |
| report | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 14.74 s | 18.25 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 11.44 s | 11.44 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 11.68 s | 11.68 s |
| statement | 3 | 0/3 | 2/3 | 2/3 | 1/3 | 100.0% | 10.21 s | 17.68 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 7.31 s | 7.31 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 100.0% | 9.37 s | 11.65 s |
| native | 47 | 23/47 | 39/47 | 42/47 | 0/44 | 100.0% | 9.55 s | 20.49 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 7.32 s | 7.32 s |
| office | 5 | 2/5 | 5/5 | 3/5 | 0/5 | 100.0% | 11.44 s | 12.52 s |
| scan | 16 | 9/16 | 15/16 | 13/16 | 4/4 | 100.0% | 9.33 s | 55.37 s |
| sheet | 3 | 1/3 | 2/3 | 3/3 | 0/2 | 100.0% | 14.74 s | 15.93 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 8.62 s | 8.62 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 17/37 | 32/37 | 32/37 | 3/26 | 100.0% | 7.76 s | 11.53 s |
| 2-4 | 24 | 15/24 | 22/24 | 23/24 | 1/17 | 100.0% | 10.16 s | 17.68 s |
| 5-9 | 4 | 2/4 | 2/4 | 3/4 | 0/4 | 100.0% | 9.18 s | 11.68 s |
| 10-24 | 4 | 2/4 | 3/4 | 3/4 | 0/3 | 100.0% | 13.90 s | 30.26 s |
| 25-49 | 4 | 3/4 | 4/4 | 3/4 | 0/3 | 100.0% | 16.98 s | 55.37 s |
| 50-99 | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 20.49 s | 23.72 s |
| 100+ | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 18.25 s | 18.64 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 7/12 | 11/12 | 9/12 | 0/10 | 0 of 12 | 18.25 s | 55.37 s | 3.12 s | 3.58 s |
| complex | 56 | 29/56 | 49/56 | 48/56 | 0/46 | 0 of 56 | 9.59 s | 22.68 s | 3.08 s | 4.12 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 34/67 | 58/67 | 58/67 | 8 |
| complex_pdf | 5 | 2/5 | 4/5 | 3/5 | 2 |
| contract | 20 | 16/20 | 18/20 | 19/20 | 3 |
| csv | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 3/7 | 6/7 | 6/7 | 1 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 3/10 | 8/10 | 8/10 | 1 |
| form | 7 | 0/7 | 6/7 | 5/7 | 0 |
| healthcare | 3 | 1/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 2/4 | 4/4 | 3/4 | 0 |
| image_only_scan | 16 | 9/16 | 15/16 | 13/16 | 1 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 6/10 | 9/10 | 8/10 | 2 |
| invoice | 7 | 4/7 | 5/7 | 7/7 | 1 |
| irrelevant_names | 17 | 6/17 | 16/17 | 13/17 | 3 |
| key_value | 11 | 5/11 | 10/11 | 9/11 | 1 |
| layout_parties | 11 | 2/11 | 7/11 | 9/11 | 3 |
| letter | 5 | 1/5 | 4/5 | 5/5 | 1 |
| low_resolution_scan | 3 | 1/3 | 2/3 | 2/3 | 1 |
| middle_fact | 6 | 2/6 | 4/6 | 5/6 | 2 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 4/7 | 7/7 | 6/7 | 1 |
| noisy_scan | 3 | 1/3 | 2/3 | 2/3 | 1 |
| notice | 10 | 9/10 | 10/10 | 10/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 1/4 | 3/4 | 3/4 | 1 |
| pages_10 | 3 | 2/3 | 2/3 | 3/3 | 1 |
| pages_100 | 2 | 1/2 | 2/2 | 1/2 | 0 |
| pages_25 | 4 | 3/4 | 4/4 | 3/4 | 1 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| png | 5 | 2/5 | 4/5 | 4/5 | 1 |
| pptx | 2 | 1/2 | 2/2 | 1/2 | 1 |
| presentation | 2 | 1/2 | 2/2 | 1/2 | 1 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 15/20 | 18/20 | 20/20 | 2 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 4/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 1/3 | 2/3 | 3/3 | 0 |
| statement | 3 | 0/3 | 2/3 | 2/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 21/41 | 34/41 | 35/41 | 5 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 3/4 | 3/4 | 4/4 | 0 |
| xlsx | 2 | 1/2 | 1/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.54 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.21 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.33 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.84 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.84 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 982.1 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.59 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.71 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 41.82 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 88.70 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.67 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.04 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.19 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.98 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.05 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.20 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 719.6 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.81 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.38 s |
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
| `total_ms` | 77 | 9.55 s | 22.68 s | 55.37 s |
| `extraction_wall_ms` | 77 | 5.1 ms | 2.95 s | 45.22 s |
| `worker_total_ms` | 77 | 4.5 ms | 2.95 s | 45.22 s |
| `worker_snapshot_ms` | 77 | 0.23 ms | 0.40 ms | 1.2 ms |
| `worker_parse_ms` | 77 | 2.5 ms | 59.1 ms | 251.1 ms |
| `worker_analysis_ms` | 77 | 0.55 ms | 37.2 ms | 83.4 ms |
| `worker_render_ms` | 77 | 0.00 ms | 201.3 ms | 4.53 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 65.3 ms | 101.6 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 4.54 s | 88.70 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 275.6 ms | 6.12 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 3.87 s | 82.23 s |
| `worker_vision_ms` | 77 | 0.00 ms | 5.9 ms | 12.2 ms |
| `analyze_wall_ms` | 77 | 8.96 s | 19.57 s | 23.58 s |
| `distill_ms` | 77 | 2.3 ms | 81.4 ms | 257.2 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.08 ms | 0.09 ms |
| `inference_ms` | 77 | 8.95 s | 19.56 s | 23.38 s |
| `prefill_ms` | 77 | 5.83 s | 16.69 s | 19.91 s |
| `generation_ms` | 77 | 3.08 s | 4.13 s | 4.78 s |
| `validation_ms` | 77 | 5.3 ms | 40.0 ms | 69.0 ms |
| `naming_ms` | 77 | 0.26 ms | 5.9 ms | 9.5 ms |
| `index_ms` | 77 | 1.9 ms | 76.5 ms | 235.3 ms |
| `retrieval_ms` | 77 | 0.04 ms | 4.8 ms | 9.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 167.4 tok/s | 170.6 tok/s | 172.4 tok/s |
| `generation_tok_per_s` | 77 | 31.3 tok/s | 33.0 tok/s | 33.9 tok/s |
| `prompt_tokens` | 77 | 978 | 2685 | 3139 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 97 | 127 | 151 |
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
| fast | 11 | 2.2 ms | 5.1 ms | 1.5 ms | 4.7 ms | 3.8 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 4.6 ms | 147.8 ms | 4.1 ms | 131.5 ms | 75.8 ms | 41.1 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 812.1 ms | 812.1 ms | 811.7 ms | 811.7 ms | 2.6 ms | 0.31 ms | 63.4 ms | 719.6 ms |
| ocr | 19 | 2.09 s | 45.22 s | 2.09 s | 45.22 s | 28.0 ms | 1.7 ms | 4.53 s | 88.70 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 4.1 ms | 2.53 s | 3.4 ms | 2.52 s | 26.6 ms | 0.83 ms | 113.4 ms | 2.38 s |
| 2-4 | 24 | 4.7 ms | 2.83 s | 4.2 ms | 2.83 s | 4.0 ms | 1.3 ms | 201.3 ms | 4.54 s |
| 5-9 | 4 | 9.9 ms | 13.8 ms | 9.0 ms | 12.9 ms | 10.3 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 19.2 ms | 21.23 s | 16.8 ms | 21.23 s | 14.5 ms | 5.8 ms | 1.47 s | 41.82 s |
| 25-49 | 4 | 50.4 ms | 45.22 s | 46.3 ms | 45.22 s | 58.3 ms | 37.2 ms | 4.53 s | 88.70 s |
| 50-99 | 2 | 56.7 ms | 147.8 ms | 52.5 ms | 131.5 ms | 75.8 ms | 53.9 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 115.5 ms | 219.5 ms | 102.1 ms | 191.4 ms | 106.0 ms | 83.4 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 812.1 ms | 1.34 s | 811.7 ms | 1.34 s | 3.6 ms | 0.89 ms | 109.6 ms | 1.21 s |
| annual_report | 2 | 9.9 ms | 115.5 ms | 9.0 ms | 102.1 ms | 59.1 ms | 41.1 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.53 s | 2.53 s | 2.52 s | 2.52 s | 0.15 ms | 0.38 ms | 113.4 ms | 2.38 s |
| certificate | 1 | 2.95 s | 2.95 s | 2.95 s | 2.95 s | 0.13 ms | 0.35 ms | 103.9 ms | 2.81 s |
| claim | 1 | 2.83 s | 2.83 s | 2.83 s | 2.83 s | 0.26 ms | 0.32 ms | 201.3 ms | 5.19 s |
| condition_report | 1 | 3.5 ms | 3.5 ms | 3.1 ms | 3.1 ms | 2.0 ms | 0.65 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 22.8 ms | 45.22 s | 20.9 ms | 45.22 s | 58.3 ms | 37.2 ms | 4.53 s | 88.70 s |
| email | 1 | 1.1 ms | 1.1 ms | 0.48 ms | 0.48 ms | 0.15 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 5.0 ms | 2.16 s | 4.4 ms | 2.16 s | 24.8 ms | 0.71 ms | 0.00 ms | 2.04 s |
| inspection_report | 1 | 5.5 ms | 5.5 ms | 4.8 ms | 4.8 ms | 2.8 ms | 1.6 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 4.4 ms | 147.8 ms | 3.9 ms | 131.5 ms | 75.8 ms | 53.9 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 3.2 ms | 1.75 s | 2.8 ms | 1.75 s | 20.6 ms | 0.59 ms | 112.5 ms | 1.62 s |
| letter | 5 | 4.2 ms | 1.77 s | 3.8 ms | 1.77 s | 22.4 ms | 0.65 ms | 0.00 ms | 1.71 s |
| log | 1 | 1.8 ms | 1.8 ms | 1.2 ms | 1.2 ms | 0.95 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 4.5 ms | 4.5 ms | 4.1 ms | 4.1 ms | 2.9 ms | 0.73 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 5.1 ms | 5.1 ms | 4.7 ms | 4.7 ms | 3.8 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 4.2 ms | 4.2 ms | 3.9 ms | 3.9 ms | 2.6 ms | 0.85 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 4.2 ms | 2.11 s | 3.8 ms | 2.11 s | 251.1 ms | 0.66 ms | 111.7 ms | 1.98 s |
| payroll | 1 | 2.3 ms | 2.3 ms | 1.5 ms | 1.5 ms | 1.2 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 2.6 ms | 2.7 ms | 2.1 ms | 2.1 ms | 1.8 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 4.1 ms | 4.1 ms | 2.7 ms | 2.7 ms | 1.6 ms | 0.75 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 3.6 ms | 1.96 s | 3.1 ms | 1.96 s | 2.0 ms | 0.72 ms | 98.1 ms | 1.84 s |
| quotation | 1 | 2.77 s | 2.77 s | 2.77 s | 2.77 s | 28.0 ms | 0.18 ms | 0.00 ms | 2.67 s |
| rate_confirmation | 1 | 4.2 ms | 4.2 ms | 3.7 ms | 3.7 ms | 2.7 ms | 0.51 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 997.4 ms | 997.4 ms | 996.9 ms | 996.9 ms | 11.1 ms | 0.10 ms | 0.00 ms | 982.1 ms |
| remittance_advice | 1 | 2.09 s | 2.09 s | 2.09 s | 2.09 s | 26.6 ms | 0.20 ms | 0.00 ms | 2.05 s |
| report | 2 | 1.8 ms | 219.5 ms | 0.80 ms | 191.4 ms | 106.0 ms | 83.4 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 2.2 ms | 2.2 ms | 1.7 ms | 1.7 ms | 1.4 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 11.2 ms | 11.2 ms | 10.3 ms | 10.3 ms | 6.6 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 5.1 ms | 1.74 s | 4.1 ms | 1.74 s | 2.5 ms | 1.2 ms | 125.3 ms | 1.59 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1280 MB | scan-lease-25p |
| Model server | 854 MB | 856 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (36)

- **invoice-layout-only**: expected `2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf`, got `Invoice from Ironvale Marine Coatings Co.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-03-17 Account Summary from Kingsfold Community Bank.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Purchase Order - Lamplighter Robotics Inc.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2026-06-03 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2026-06-03 (client signature date) · review: LOW_CONFIDENCE
- **offer-letter**: expected `2026-02-10 Offer Letter to Dalia Brennagh.pdf`, got `2026-02-10 Letter to Dalia Brennagh.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Meadowlark Health Plan from Meadowlark Health Plan.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 Notice from Ravensmoor Growth Partners III, L.P.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-03 Letter to Briarport Coffee Roasters LLC.pdf` · trap: date 2026-03-03 (date of the customer's earlier letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-06-18 Commercial Package Policy Declarations for Kingsfold Insurance Agency, Inc.pdf` · trap: party Kingsfold Insurance Agency, Inc. (insurance agency (producer)) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report for Casimir Ellingsen.pdf` · trap: party Casimir Ellingsen (chief executive who signs the letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Document from Hartwell County.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **change-order-form**: expected `2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf`, got `2026-05-14 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes from Cairnfield Cooperative Grocers.pdf` · review: LOW_CONFIDENCE
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `2026-09-18 Annual Report for Kingsfold Farm Credit Bank.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Letter to Fernvale Cider Works LLC.docx` · review: TYPE_MISSING, LOW_CONFIDENCE
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors for Board of Directors.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review for Signe Holmqvist.pptx` · trap: party Signe Holmqvist (customer attendee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Document - Lowmarsh Farm Equipment Co.csv` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-08-16 Summary for Rowanbrae Vineyards.xlsx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Store 3 REG 2 Cashier Hollis.png` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement from 08312026.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quotation from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-08-27 Document - Ione Kowalczyk.png` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Tidewater Ledger - Saltmarsh Point Homeowners Association.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **agreement-two-column-footnotes**: expected `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`, got `2026-03-02 Document between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **invoice-label-above**: expected `2026-05-11 Invoice from Quarrystone Signs & Graphics.pdf`, got `2026-05-06 Invoice from Quarrystone Signs & Graphics.pdf` · trap: date 2026-05-06 (installation date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **invoice-boxed-grid**: expected `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf`, got `2026-07-09 Document from Whitlock & Sons Plumbing LLC.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-29 Medical Plan for Human Resources.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Document from Harrowmere Mutual Insurance Company.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-05-28 Invoice from Tolliver Grain & Feed Cooperative.png` · trap: date 2026-05-28 (date of SPC-30417) · review: LOW_CONFIDENCE
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-06-23 Certificate of Liability Insurance for Calloway Crossing Shopping Center LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_MISSING, LOW_CONFIDENCE
- **msa-effective-date-in-definitions-12p**: expected `2026-03-02 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf`, got `2026-03-11 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf` · trap: date 2026-03-11 (client signature date) · review: LOW_CONFIDENCE
- **industrial-lease-dated-in-schedule-25p**: expected `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`, got `2026-04-14 Industrial Lease between Moss & Lanyard Distribution Inc and Moss & Lanyard Holdings LLC.pdf` · trap: party Moss & Lanyard Holdings LLC (guarantor, not a party to the lease) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
