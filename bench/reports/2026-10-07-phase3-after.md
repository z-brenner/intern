# InternBench: replay run

- **Run:** 2026-10-07T18:03:20Z · commit `50bc017991`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-07T16:54:53Z at commit `902c7fe65c` · sha256 `bb64876b23`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 54/77 (70.1%) |
| Document type | 72/77 (93.5%) |
| Date | 75/77 (97.4%) |
| Date role (when the reviewed date was chosen) | 42/74 (56.8%) |
| Parties | 67/77 (87.0%) |
| Relation word (when parties were named) | 61/75 (81.3%) |
| Party in the right role (when parties were named) | 57/65 (87.7%) |
| Ready / review routing | 44/57 (77.2%) |
| Description has every fact | 21/77 (27.3%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 74/77 (96.1%) |
| Description fact coverage | 53.7% (mean of 77) |
| Description specificity | 93.9% (mean of 77) |
| Evidence recall (model's quotes) | 66.0% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 15.6% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 75.0% | 12 |
| Complex-document filename accuracy | 73.2% | 56 |
| Description completeness | 53.7% | 77 |
| Unsupported-fact rate (documents) | 11.7% | 77 |
| Review rate | 15.6% | 77 |
| Evidence recall | 66.0% | 77 |
| Total latency p50 | 19.21 s | 77 |
| Total latency p95 | 33.88 s | 77 |
| Generation latency p50 | 6.53 s | 77 |
| Generation latency p95 | 8.82 s | 77 |
| Generated tokens p50 | 87 | 77 |
| Generated tokens p95 | 116 | 77 |
| Prompt tokens p50 | 914 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 15 of 77 |
| Trap date chosen | 1 of 77 |
| Forbidden party named | 8 of 77 |
| Spurious parties named | 9 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 290 (0.0%) |
| Right name sent to review anyway | 2 of 53 |
| Sent to review for an unsupported fact | 9 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 2/3 | 1/1 | 0.0% | 22.91 s | 25.46 s |
| annual_report | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 15.67 s | 32.82 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 19.80 s | 19.80 s |
| certificate | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 26.72 s | 26.72 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 20.95 s | 20.95 s |
| condition_report | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 20.20 s | 20.20 s |
| contract | 16 | 15/16 | 16/16 | 16/16 | 12/12 | 0.0% | 25.14 s | 73.91 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 15.13 s | 15.13 s |
| form | 5 | 5/5 | 5/5 | 5/5 | 4/4 | 0.0% | 17.46 s | 22.18 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 13.26 s | 13.26 s |
| insurance | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 21.71 s | 35.25 s |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 4/5 | 28.6% | 13.85 s | 15.52 s |
| letter | 5 | 2/5 | 5/5 | 4/5 | 2/4 | 60.0% | 18.13 s | 19.76 s |
| log | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 32.98 s | 32.98 s |
| maintenance_record | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 21.89 s | 21.89 s |
| minutes | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 27.42 s | 27.42 s |
| newsletter | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 18.42 s | 18.42 s |
| notice | 10 | 8/10 | 10/10 | 10/10 | 8/8 | 10.0% | 16.42 s | 25.59 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 16.38 s | 16.38 s |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 1/2 | 50.0% | 16.77 s | 24.38 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 19.99 s | 19.99 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 15.74 s | 20.17 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 17.71 s | 17.71 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 18.12 s | 18.12 s |
| receipt | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 12.62 s | 12.62 s |
| remittance_advice | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 19.21 s | 19.21 s |
| report | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 28.53 s | 33.88 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 26.94 s | 26.94 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 21.92 s | 21.92 s |
| statement | 3 | 2/3 | 3/3 | 3/3 | 1/3 | 33.3% | 19.21 s | 21.16 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 15.13 s | 15.13 s |
| mixed | 3 | 2/3 | 2/3 | 2/3 | – | 0.0% | 18.96 s | 25.46 s |
| native | 47 | 35/47 | 46/47 | 41/47 | 38/44 | 12.8% | 19.82 s | 33.87 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 13.43 s | 13.43 s |
| office | 5 | 2/5 | 5/5 | 4/5 | 3/5 | 40.0% | 24.38 s | 26.94 s |
| scan | 16 | 12/16 | 16/16 | 15/16 | 0/4 | 12.5% | 17.71 s | 73.91 s |
| sheet | 3 | 2/3 | 3/3 | 2/3 | 2/2 | 0.0% | 28.53 s | 32.98 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 18.49 s | 18.49 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 24/37 | 37/37 | 32/37 | 17/26 | 24.3% | 16.42 s | 26.94 s |
| 2-4 | 24 | 18/24 | 23/24 | 19/24 | 15/17 | 4.2% | 19.76 s | 27.42 s |
| 5-9 | 4 | 3/4 | 3/4 | 4/4 | 2/4 | 50.0% | 16.77 s | 21.92 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 0.0% | 24.38 s | 43.82 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 29.70 s | 73.91 s |
| 50-99 | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 32.58 s | 35.25 s |
| 100+ | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 32.82 s | 33.88 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 9/12 | 12/12 | 12/12 | 10/10 | 3 of 12 | 32.58 s | 73.91 s | 6.53 s | 9.71 s |
| complex | 56 | 41/56 | 54/56 | 50/56 | 40/46 | 9 of 56 | 19.80 s | 33.88 s | 6.59 s | 9.00 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 2/3 | 2 |
| competing_dates | 67 | 46/67 | 65/67 | 57/67 | 9 |
| complex_pdf | 5 | 4/5 | 4/5 | 5/5 | 0 |
| contract | 20 | 18/20 | 19/20 | 19/20 | 2 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 5/7 | 7/7 | 6/7 | 0 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 1 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 7/10 | 9/10 | 9/10 | 1 |
| form | 7 | 6/7 | 7/7 | 6/7 | 1 |
| healthcare | 3 | 2/3 | 3/3 | 3/3 | 0 |
| hr | 4 | 4/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 12/16 | 16/16 | 15/16 | 1 |
| image_region | 1 | 0/1 | 0/1 | 0/1 | 2 |
| information_dense | 10 | 8/10 | 10/10 | 10/10 | 0 |
| invoice | 7 | 6/7 | 7/7 | 6/7 | 0 |
| irrelevant_names | 17 | 12/17 | 17/17 | 15/17 | 2 |
| key_value | 11 | 9/11 | 11/11 | 10/11 | 1 |
| layout_parties | 11 | 8/11 | 11/11 | 9/11 | 1 |
| letter | 5 | 2/5 | 5/5 | 4/5 | 1 |
| low_resolution_scan | 3 | 1/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 4/6 | 6/6 | 6/6 | 0 |
| mixed_scan | 3 | 2/3 | 2/3 | 2/3 | 2 |
| multi_column | 7 | 6/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 8/10 | 10/10 | 10/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 1/4 | 4/4 | 3/4 | 1 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| png | 5 | 4/5 | 5/5 | 5/5 | 0 |
| pptx | 2 | 1/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 14/20 | 19/20 | 18/20 | 3 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 2/3 | 3/3 | 2/3 | 1 |
| statement | 3 | 2/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 27/41 | 40/41 | 34/41 | 5 |
| tiff | 2 | 2/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 0/4 | 4/4 | 0/4 | 3 |
| xlsx | 2 | 1/2 | 2/2 | 1/2 | 1 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.44 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.47 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.75 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 2.20 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.41 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.25 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 2.04 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 2.26 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 27.65 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 57.05 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 3.35 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.65 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 3.50 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 2.19 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.30 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.55 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 941.3 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 3.31 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 2.99 s |
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
| `total_ms` | 77 | 19.21 s | 33.88 s | 73.91 s |
| `extraction_wall_ms` | 77 | 6.6 ms | 3.68 s | 57.26 s |
| `worker_total_ms` | 77 | 5.7 ms | 3.68 s | 57.25 s |
| `worker_snapshot_ms` | 77 | 0.21 ms | 0.57 ms | 2.4 ms |
| `worker_parse_ms` | 77 | 3.0 ms | 57.7 ms | 131.9 ms |
| `worker_analysis_ms` | 77 | 0.67 ms | 40.9 ms | 98.6 ms |
| `worker_render_ms` | 77 | 0.00 ms | 249.2 ms | 3.21 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 78.1 ms | 153.9 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 3.50 s | 57.05 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 332.6 ms | 5.69 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 3.05 s | 51.16 s |
| `worker_vision_ms` | 77 | 0.00 ms | 7.7 ms | 21.9 ms |
| `analyze_wall_ms` | 77 | 18.38 s | 32.98 s | 35.07 s |
| `distill_ms` | 77 | 4.9 ms | 118.5 ms | 638.0 ms |
| `prompt_ms` | 77 | 0.05 ms | 0.09 ms | 0.85 ms |
| `inference_ms` | 77 | 18.35 s | 32.32 s | 34.32 s |
| `prefill_ms` | 77 | 11.41 s | 24.14 s | 27.64 s |
| `generation_ms` | 77 | 6.53 s | 8.82 s | 9.71 s |
| `validation_ms` | 77 | 26.2 ms | 304.2 ms | 614.3 ms |
| `naming_ms` | 77 | 0.94 ms | 15.6 ms | 19.9 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 77.9 tok/s | 83.3 tok/s | 85.2 tok/s |
| `generation_tok_per_s` | 77 | 13.0 tok/s | 13.7 tok/s | 13.9 tok/s |
| `prompt_tokens` | 77 | 914 | 1861 | 2080 |
| `cached_tokens` | 77 | 477 | 477 | 1104 |
| `generated_tokens` | 77 | 87 | 116 | 122 |
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

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Model server | 4872 MB | 7232 MB | watershed-monitoring-report-100p |
| Benchmark (engine) | 22 MB | 40 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (23)

- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Invoice.pdf` · **filed without review**
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Meadowlark Health Plan from Paloma Achterberg.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Document - Joaquin Sandoval.pdf` · trap: party Joaquin Sandoval (client contact the letter is addressed to) · review: TYPE_UNSUPPORTED
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document - Briarport Coffee Roasters LLC.pdf` · review: TYPE_UNSUPPORTED
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report from Rookwood Precision Metals Corporation.pdf` · review: DATE_UNSUPPORTED
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes of the Regular Meeting of the Board of Directors for Beckett Halloran.pdf` · trap: party Beckett Halloran (board president) · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report for Delphine Moncrieff.pdf` · trap: party Delphine Moncrieff (conservator who examined the work) · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf` · trap: party Arlo Sinclair-Ray (inspector who signed the return to service) · **filed without review**
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Demand for Payment - Fernvale Cider Works LLC.docx` · **filed without review**
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors for Celeste Fontaine.docx` · trap: party Celeste Fontaine (director) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx` · **filed without review**
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 Harvest Log from Esme Varga.xlsx` · trap: party Esme Varga (winemaker) · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document from Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt` · **filed without review**
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **agreement-two-column-footnotes**: expected `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`, got `2026-03-02 Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Price List.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **scan-cancellation-notice-150dpi**: expected `2026-08-12 Notice of Cancellation for Brannock Tool & Die Inc.pdf`, got `2026-08-12 Notice of Cancellation for Nonpayment of Premium - Brannock Tool & Die Inc.pdf` · review: DATE_AMBIGUOUS
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-07-17 Remittance Advice - Sablewood Packaging Corp.png` · **filed without review**
- **mixed-signature-region**: expected `2026-08-21 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf`, got `2026-08-18 First Amendment to Software License Agreement between Rosalind Achterberg and Barnaby Quist.pdf` · trap: date 2026-08-18 (earlier of the two signatures, not the last); party Rosalind Achterberg (signatory); party Barnaby Quist (signatory) · **filed without review**
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `2026-09-08 Straight Bill of Lading - Odile Corriveau.pdf` · trap: party Odile Corriveau (shipper's signer) · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · **filed without review**
- **watershed-monitoring-report-100p**: expected `2026-06-19 Watershed Monitoring Report for Cobalt Basin Water Authority.pdf`, got `2026-06-19 Annual Watershed Monitoring Report to Cobalt Basin Water Authority.pdf` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
