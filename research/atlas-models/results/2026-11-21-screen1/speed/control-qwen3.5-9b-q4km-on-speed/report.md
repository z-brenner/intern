# InternBench: live run

- **Run:** 2026-11-21T05:33:51Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Wall time:** 7153.03 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 72/77 (93.5%) |
| Document type | 75/77 (97.4%) |
| Date | 74/77 (96.1%) |
| Date role (when the reviewed date was chosen) | 55/73 (75.3%) |
| Parties | 77/77 (100.0%) |
| Relation word (when parties were named) | 77/77 (100.0%) |
| Party in the right role (when parties were named) | 77/77 (100.0%) |
| Ready / review routing | 30/57 (52.6%) |
| Description has every fact | 23/77 (29.9%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 77/77 (100.0%) |
| Description fact coverage | 56.9% (mean of 77) |
| Description specificity | 97.4% (mean of 77) |
| Evidence recall (model's quotes) | 89.8% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 48.0% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 83.3% | 12 |
| Complex-document filename accuracy | 92.9% | 56 |
| Description completeness | 56.9% | 77 |
| Unsupported-fact rate (documents) | 7.8% | 77 |
| Review rate | 48.0% | 77 |
| Evidence recall | 89.8% | 77 |
| Total latency p50 | 79.45 s | 77 |
| Total latency p95 | 187.12 s | 77 |
| Generation latency p50 | 18.59 s | 77 |
| Generation latency p95 | 23.15 s | 77 |
| Generated tokens p50 | 90 | 77 |
| Generated tokens p95 | 110 | 77 |
| Prompt tokens p50 | 978 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 77 |
| Trap date chosen | 3 of 77 |
| Forbidden party named | 0 of 77 |
| Spurious parties named | 0 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 295 (0.0%) |
| Right name sent to review anyway | 21 of 53 |
| Sent to review for an unsupported fact | 6 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 0.0% | 98.75 s | 98.90 s |
| annual_report | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 83.34 s | 166.37 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 75.83 s | 75.83 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 78.97 s | 78.97 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 67.88 s | 67.88 s |
| condition_report | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 78.94 s | 78.94 s |
| contract | 16 | 15/16 | 15/16 | 16/16 | 9/12 | 18.8% | 131.71 s | 216.08 s |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 66.36 s | 66.36 s |
| form | 5 | 5/5 | 5/5 | 5/5 | 2/4 | 60.0% | 63.76 s | 88.88 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 166.44 s | 166.44 s |
| insurance | 2 | 1/2 | 1/2 | 2/2 | 0/2 | 100.0% | 100.25 s | 215.95 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 1/5 | 85.7% | 54.81 s | 65.39 s |
| letter | 5 | 4/5 | 5/5 | 5/5 | 0/4 | 100.0% | 76.15 s | 89.61 s |
| log | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 148.51 s | 148.51 s |
| maintenance_record | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 85.97 s | 85.97 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 105.93 s | 105.93 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 74.86 s | 74.86 s |
| notice | 10 | 10/10 | 10/10 | 10/10 | 4/8 | 50.0% | 55.58 s | 101.26 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 83.81 s | 83.81 s |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 74.96 s | 113.65 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 89.73 s | 89.73 s |
| purchase_order | 2 | 2/2 | 2/2 | 2/2 | 0/1 | 100.0% | 48.76 s | 83.50 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 61.25 s | 61.25 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 73.15 s | 73.15 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 54.91 s | 54.91 s |
| remittance_advice | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 64.44 s | 64.44 s |
| report | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 50.0% | 122.36 s | 169.45 s |
| resolution | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 104.78 s | 104.78 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 111.52 s | 111.52 s |
| statement | 3 | 3/3 | 3/3 | 3/3 | 2/3 | 66.7% | 90.64 s | 159.10 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 66.36 s | 66.36 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 0.0% | 76.94 s | 98.75 s |
| native | 47 | 43/47 | 44/47 | 47/47 | 25/44 | 44.7% | 85.97 s | 188.72 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 47.76 s | 47.76 s |
| office | 5 | 5/5 | 5/5 | 5/5 | 2/5 | 60.0% | 104.43 s | 113.65 s |
| scan | 16 | 15/16 | 16/16 | 16/16 | 2/4 | 56.2% | 61.25 s | 142.37 s |
| sheet | 3 | 3/3 | 3/3 | 3/3 | 1/2 | 33.3% | 122.36 s | 148.51 s |
| text | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 69.80 s | 69.80 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 36/37 | 37/37 | 37/37 | 10/26 | 70.3% | 63.76 s | 104.78 s |
| 2-4 | 24 | 23/24 | 24/24 | 24/24 | 9/17 | 33.3% | 83.97 s | 166.44 s |
| 5-9 | 4 | 3/4 | 3/4 | 4/4 | 3/4 | 25.0% | 79.45 s | 111.52 s |
| 10-24 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 113.65 s | 151.82 s |
| 25-49 | 4 | 3/4 | 3/4 | 4/4 | 2/3 | 25.0% | 142.37 s | 216.08 s |
| 50-99 | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 187.12 s | 215.95 s |
| 100+ | 2 | 2/2 | 2/2 | 2/2 | 2/2 | 0.0% | 166.37 s | 169.45 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 10/12 | 10/12 | 12/12 | 8/10 | 0 of 12 | 151.82 s | 216.08 s | 18.99 s | 23.98 s |
| complex | 56 | 52/56 | 53/56 | 56/56 | 23/46 | 0 of 56 | 83.50 s | 187.12 s | 18.59 s | 23.62 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 63/67 | 64/67 | 67/67 | 3 |
| complex_pdf | 5 | 5/5 | 5/5 | 5/5 | 0 |
| contract | 20 | 18/20 | 18/20 | 20/20 | 2 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 5/7 | 5/7 | 7/7 | 2 |
| docx | 3 | 3/3 | 3/3 | 3/3 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 8/10 | 9/10 | 10/10 | 1 |
| form | 7 | 7/7 | 7/7 | 7/7 | 0 |
| healthcare | 3 | 3/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 15/16 | 16/16 | 16/16 | 0 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 8/10 | 8/10 | 10/10 | 2 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 15/17 | 15/17 | 17/17 | 2 |
| key_value | 11 | 11/11 | 11/11 | 11/11 | 0 |
| layout_parties | 11 | 11/11 | 11/11 | 11/11 | 0 |
| letter | 5 | 4/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 4/6 | 4/6 | 6/6 | 2 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 7/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 10/10 | 10/10 | 10/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 3/4 | 3/4 | 4/4 | 1 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 1/2 | 2/2 | 1 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 2/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 2/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 18/20 | 19/20 | 20/20 | 1 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 4/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 3/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 39/41 | 39/41 | 41/41 | 2 |
| tiff | 2 | 2/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 4/4 | 4/4 | 4/4 | 0 |
| xlsx | 2 | 2/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.76 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.22 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.34 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 1.86 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.88 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 971.5 ms |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.61 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.74 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 41.97 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 90.71 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.64 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.11 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 5.44 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 1.99 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.06 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.20 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 711.2 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 2.80 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.40 s |
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
| `total_ms` | 77 | 79.45 s | 187.12 s | 216.08 s |
| `extraction_wall_ms` | 77 | 5.1 ms | 2.95 s | 46.26 s |
| `worker_total_ms` | 77 | 4.6 ms | 2.95 s | 46.26 s |
| `worker_snapshot_ms` | 77 | 0.23 ms | 0.41 ms | 1.3 ms |
| `worker_parse_ms` | 77 | 2.5 ms | 51.1 ms | 106.8 ms |
| `worker_analysis_ms` | 77 | 0.53 ms | 35.6 ms | 83.8 ms |
| `worker_render_ms` | 77 | 0.00 ms | 224.1 ms | 4.54 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 66.7 ms | 90.7 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 4.76 s | 90.71 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 273.5 ms | 6.16 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 4.04 s | 84.20 s |
| `worker_vision_ms` | 77 | 0.00 ms | 6.0 ms | 12.2 ms |
| `analyze_wall_ms` | 77 | 79.44 s | 187.06 s | 216.04 s |
| `distill_ms` | 77 | 2.3 ms | 84.0 ms | 246.7 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.07 ms | 0.09 ms |
| `inference_ms` | 77 | 79.42 s | 186.96 s | 215.99 s |
| `prefill_ms` | 77 | 60.51 s | 167.43 s | 196.64 s |
| `generation_ms` | 77 | 18.59 s | 23.15 s | 25.50 s |
| `validation_ms` | 77 | 4.9 ms | 35.8 ms | 65.6 ms |
| `naming_ms` | 77 | 0.25 ms | 6.0 ms | 9.5 ms |
| `index_ms` | 77 | 1.9 ms | 76.7 ms | 236.4 ms |
| `retrieval_ms` | 77 | 0.03 ms | 4.8 ms | 9.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 16.1 tok/s | 16.2 tok/s | 16.2 tok/s |
| `generation_tok_per_s` | 77 | 4.9 tok/s | 5.0 tok/s | 5.0 tok/s |
| `prompt_tokens` | 77 | 978 | 2685 | 3139 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 90 | 110 | 123 |
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
| fast | 11 | 2.1 ms | 5.0 ms | 1.5 ms | 4.5 ms | 3.6 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 46 | 4.5 ms | 115.4 ms | 4.0 ms | 101.9 ms | 58.2 ms | 41.6 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 801.7 ms | 801.7 ms | 801.2 ms | 801.2 ms | 2.5 ms | 0.30 ms | 63.1 ms | 711.2 ms |
| ocr | 19 | 2.11 s | 46.26 s | 2.11 s | 46.26 s | 35.5 ms | 1.7 ms | 4.54 s | 90.71 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 4.0 ms | 2.55 s | 3.6 ms | 2.55 s | 27.7 ms | 0.82 ms | 112.7 ms | 2.40 s |
| 2-4 | 24 | 4.9 ms | 2.95 s | 4.4 ms | 2.95 s | 3.9 ms | 1.3 ms | 224.1 ms | 4.76 s |
| 5-9 | 4 | 9.3 ms | 13.6 ms | 8.4 ms | 12.7 ms | 10.2 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| 10-24 | 4 | 18.8 ms | 21.29 s | 17.2 ms | 21.29 s | 14.4 ms | 5.8 ms | 1.52 s | 41.97 s |
| 25-49 | 4 | 58.4 ms | 46.26 s | 53.6 ms | 46.26 s | 51.1 ms | 35.6 ms | 4.54 s | 90.71 s |
| 50-99 | 2 | 56.4 ms | 145.7 ms | 52.2 ms | 129.4 ms | 74.8 ms | 53.1 ms | 0.00 ms | 0.00 ms |
| 100+ | 2 | 115.4 ms | 221.4 ms | 101.9 ms | 192.9 ms | 106.8 ms | 83.8 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 801.7 ms | 1.36 s | 801.2 ms | 1.36 s | 3.0 ms | 0.93 ms | 109.8 ms | 1.22 s |
| annual_report | 2 | 9.3 ms | 115.4 ms | 8.4 ms | 101.9 ms | 58.2 ms | 41.6 ms | 0.00 ms | 0.00 ms |
| bill_of_lading | 1 | 2.55 s | 2.55 s | 2.55 s | 2.55 s | 0.18 ms | 0.36 ms | 112.7 ms | 2.40 s |
| certificate | 1 | 2.93 s | 2.93 s | 2.93 s | 2.93 s | 0.22 ms | 0.35 ms | 102.9 ms | 2.80 s |
| claim | 1 | 2.95 s | 2.95 s | 2.95 s | 2.95 s | 0.29 ms | 0.32 ms | 224.1 ms | 5.44 s |
| condition_report | 1 | 4.1 ms | 4.1 ms | 3.7 ms | 3.7 ms | 2.6 ms | 0.64 ms | 0.00 ms | 0.00 ms |
| contract | 16 | 22.8 ms | 46.26 s | 20.9 ms | 46.26 s | 51.1 ms | 35.6 ms | 4.54 s | 90.71 s |
| email | 1 | 1.1 ms | 1.1 ms | 0.41 ms | 0.41 ms | 0.13 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| form | 5 | 4.5 ms | 2.25 s | 4.0 ms | 2.24 s | 28.2 ms | 0.72 ms | 0.00 ms | 2.11 s |
| inspection_report | 1 | 5.5 ms | 5.5 ms | 4.8 ms | 4.8 ms | 2.8 ms | 1.6 ms | 0.00 ms | 0.00 ms |
| insurance | 2 | 4.8 ms | 145.7 ms | 4.4 ms | 129.4 ms | 74.8 ms | 53.1 ms | 0.00 ms | 0.00 ms |
| invoice | 7 | 3.2 ms | 1.78 s | 2.6 ms | 1.78 s | 26.6 ms | 0.58 ms | 113.4 ms | 1.65 s |
| letter | 5 | 4.2 ms | 1.82 s | 3.5 ms | 1.82 s | 27.7 ms | 0.64 ms | 0.00 ms | 1.74 s |
| log | 1 | 2.1 ms | 2.1 ms | 1.4 ms | 1.4 ms | 1.1 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| maintenance_record | 1 | 4.6 ms | 4.6 ms | 4.2 ms | 4.2 ms | 3.0 ms | 0.72 ms | 0.00 ms | 0.00 ms |
| minutes | 1 | 5.0 ms | 5.0 ms | 4.5 ms | 4.5 ms | 3.6 ms | 0.37 ms | 0.00 ms | 0.00 ms |
| newsletter | 1 | 4.0 ms | 4.0 ms | 3.6 ms | 3.6 ms | 2.4 ms | 0.82 ms | 0.00 ms | 0.00 ms |
| notice | 10 | 4.0 ms | 2.12 s | 3.6 ms | 2.12 s | 17.8 ms | 0.65 ms | 106.0 ms | 1.99 s |
| payroll | 1 | 2.4 ms | 2.4 ms | 1.7 ms | 1.7 ms | 1.3 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| presentation | 2 | 2.1 ms | 2.5 ms | 1.5 ms | 1.9 ms | 1.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| price_list | 1 | 4.0 ms | 4.0 ms | 3.5 ms | 3.5 ms | 2.4 ms | 0.74 ms | 0.00 ms | 0.00 ms |
| purchase_order | 2 | 3.6 ms | 1.99 s | 3.2 ms | 1.99 s | 2.0 ms | 0.73 ms | 111.7 ms | 1.86 s |
| quotation | 1 | 2.75 s | 2.75 s | 2.75 s | 2.75 s | 35.5 ms | 0.18 ms | 0.00 ms | 2.64 s |
| rate_confirmation | 1 | 3.6 ms | 3.6 ms | 3.0 ms | 3.0 ms | 2.1 ms | 0.51 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 983.9 ms | 983.9 ms | 983.2 ms | 983.2 ms | 7.4 ms | 0.10 ms | 0.00 ms | 971.5 ms |
| remittance_advice | 1 | 2.11 s | 2.11 s | 2.11 s | 2.11 s | 26.9 ms | 0.20 ms | 0.00 ms | 2.06 s |
| report | 2 | 1.4 ms | 221.4 ms | 0.82 ms | 192.9 ms | 106.8 ms | 83.8 ms | 0.00 ms | 0.00 ms |
| resolution | 1 | 2.2 ms | 2.2 ms | 1.7 ms | 1.7 ms | 1.4 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| sow | 1 | 11.9 ms | 11.9 ms | 10.8 ms | 10.8 ms | 7.2 ms | 3.1 ms | 0.00 ms | 0.00 ms |
| statement | 3 | 5.3 ms | 1.74 s | 4.8 ms | 1.74 s | 3.1 ms | 1.3 ms | 107.2 ms | 1.61 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 18 MB | 1310 MB | scan-lease-25p |
| Model server | 6082 MB | 6084 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 24 MB | 43 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (5)

- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2026-06-03 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2026-06-03 (client signature date) · review: LOW_CONFIDENCE
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter to Silverbirch Ceramics Inc.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED
- **industrial-lease-dated-in-schedule-25p**: expected `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`, got `2026-04-16 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf` · trap: date 2026-04-16 (tenant signature date) · review: LOW_CONFIDENCE
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-06-24 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf` · trap: date 2026-06-24 (countersignature date) · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
