# InternBench: replay run

- **Run:** 2026-10-07T18:03:15Z · commit `50bc017991`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 77 documents · 76 completed · 1 model_failed · gold `0d03c75723`
- **Recording:** made 2026-10-07T05:45:28Z at commit `bd48e987d5` · sha256 `5efea19aea`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 26/77 (33.8%) |
| Document type | 61/77 (79.2%) |
| Date | 65/77 (84.4%) |
| Date role (when the reviewed date was chosen) | 43/63 (68.2%) |
| Parties | 53/77 (68.8%) |
| Relation word (when parties were named) | 35/76 (46.1%) |
| Party in the right role (when parties were named) | 33/46 (71.7%) |
| Ready / review routing | 37/57 (64.9%) |
| Description has every fact | 11/77 (14.3%) |
| Description states nothing false | 75/76 (98.7%) |
| Description is specific | 76/77 (98.7%) |
| Description fact coverage | 53.5% (mean of 77) |
| Description specificity | 98.3% (mean of 77) |
| Evidence recall (model's quotes) | 51.7% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 99.4% (mean of 77) |
| Sent to review | 32.9% of 76 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 12 |
| Complex-document filename accuracy | 33.9% | 56 |
| Description completeness | 53.5% | 77 |
| Unsupported-fact rate (documents) | 27.3% | 77 |
| Review rate | 32.9% | 76 |
| Evidence recall | 51.7% | 77 |
| Total latency p50 | 38.94 s | 76 |
| Total latency p95 | 96.03 s | 76 |
| Generation latency p50 | 10.69 s | 76 |
| Generation latency p95 | 16.18 s | 76 |
| Generated tokens p50 | 141 | 76 |
| Generated tokens p95 | 204 | 76 |
| Prompt tokens p50 | 2144 | 76 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 30 of 77 |
| Trap date chosen | 11 of 77 |
| Forbidden party named | 9 of 77 |
| Spurious parties named | 25 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 264 (0.4%) |
| Right name sent to review anyway | 3 of 53 |
| Sent to review for an unsupported fact | 21 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 1/3 | 2/3 | 2/3 | 1/1 | 0.0% | 38.94 s | 47.04 s |
| annual_report | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 65.09 s | 102.78 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 42.12 s | 42.12 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 43.43 s | 43.43 s |
| claim | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 23.70 s | 23.70 s |
| condition_report | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 32.98 s | 32.98 s |
| contract | 16 | 15/16 | 16/16 | 16/16 | 9/12 | 18.8% | 63.28 s | 142.34 s |
| email | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 37.39 s | 37.39 s |
| form | 5 | 1/5 | 4/5 | 3/5 | 4/4 | 20.0% | 32.55 s | 36.36 s |
| inspection_report | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 63.24 s | 63.24 s |
| insurance | 2 | 0/2 | 2/2 | 1/2 | 2/2 | 0.0% | 37.08 s | 103.37 s |
| invoice | 7 | 5/7 | 7/7 | 5/7 | 3/5 | 42.9% | 31.90 s | 35.85 s |
| letter | 5 | 0/5 | 4/5 | 4/5 | 2/4 | 60.0% | 35.82 s | 40.48 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 53.87 s | 53.87 s |
| maintenance_record | 1 | 0/1 | 0/1 | 1/1 | – | 0.0% | 41.14 s | 41.14 s |
| minutes | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 40.40 s | 40.40 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 41.03 s | 41.03 s |
| notice | 10 | 0/10 | 7/10 | 4/10 | 1/8 | 90.0% | 32.88 s | 43.09 s |
| payroll | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 72.22 s | 72.22 s |
| presentation | 2 | 1/2 | 1/2 | 1/2 | 2/2 | 0.0% | 33.86 s | 42.99 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 39.47 s | 39.47 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 23.98 s | 39.44 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 22.15 s | 22.15 s |
| rate_confirmation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 41.32 s | 41.32 s |
| receipt | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 100.0% | 30.26 s | 30.26 s |
| remittance_advice | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 0.0% | 28.32 s | 28.32 s |
| report | 2 | 0/2 | 1/2 | 0/2 | 0/2 | 100.0% | 44.27 s | 44.27 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 44.99 s | 44.99 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 67.28 s | 67.28 s |
| statement | 3 | 1/3 | 2/3 | 2/3 | 3/3 | 33.3% | 43.72 s | 56.55 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 37.39 s | 37.39 s |
| mixed | 3 | 2/3 | 2/3 | 2/3 | – | 0.0% | 47.04 s | 49.53 s |
| native | 47 | 15/47 | 38/47 | 32/47 | 30/44 | 28.3% | 39.44 s | 96.03 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 31.90 s | 31.90 s |
| office | 5 | 2/5 | 4/5 | 3/5 | 3/5 | 40.0% | 42.04 s | 44.99 s |
| scan | 16 | 6/16 | 15/16 | 12/16 | 2/4 | 43.8% | 28.32 s | 142.34 s |
| sheet | 3 | 0/3 | 3/3 | 2/3 | 1/2 | 33.3% | 53.87 s | 72.22 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 43.09 s | 43.09 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 9/37 | 31/37 | 22/37 | 14/26 | 48.6% | 33.25 s | 44.27 s |
| 2-4 | 24 | 8/24 | 21/24 | 18/24 | 13/17 | 16.7% | 38.94 s | 63.28 s |
| 5-9 | 4 | 1/4 | 2/4 | 3/4 | 4/4 | 0.0% | 59.47 s | 67.28 s |
| 10-24 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 70.23 s | 90.10 s |
| 25-49 | 4 | 3/4 | 4/4 | 4/4 | 1/3 | 50.0% | 89.52 s | 142.34 s |
| 50-99 | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 50.0% | 72.61 s | 103.37 s |
| 100+ | 2 | 0/2 | 1/2 | 1/2 | 1/2 | 0.0% | 102.78 s | 102.78 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 8/12 | 11/12 | 10/12 | 6/10 | 2 of 12 | 89.52 s | 142.34 s | 12.82 s | 24.08 s |
| complex | 56 | 19/56 | 48/56 | 37/56 | 30/46 | 21 of 56 | 39.97 s | 102.78 s | 11.19 s | 16.78 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 1/3 | 2/3 | 2/3 | 2 |
| competing_dates | 67 | 22/67 | 55/67 | 46/67 | 20 |
| complex_pdf | 5 | 0/5 | 3/5 | 3/5 | 4 |
| contract | 20 | 16/20 | 18/20 | 19/20 | 3 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_table | 7 | 3/7 | 7/7 | 4/7 | 2 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 1 | 0/1 | 1/1 | 0/1 | 0 |
| financial | 10 | 2/10 | 9/10 | 6/10 | 2 |
| form | 7 | 1/7 | 6/7 | 4/7 | 2 |
| healthcare | 3 | 0/3 | 3/3 | 2/3 | 0 |
| hr | 4 | 1/4 | 2/4 | 3/4 | 2 |
| image_only_scan | 16 | 6/16 | 15/16 | 12/16 | 2 |
| image_region | 1 | 0/1 | 0/1 | 0/1 | 2 |
| information_dense | 10 | 6/10 | 9/10 | 8/10 | 1 |
| invoice | 7 | 5/7 | 7/7 | 5/7 | 2 |
| irrelevant_names | 17 | 6/17 | 15/17 | 13/17 | 2 |
| key_value | 11 | 2/11 | 10/11 | 5/11 | 3 |
| layout_parties | 11 | 3/11 | 11/11 | 6/11 | 3 |
| letter | 5 | 0/5 | 4/5 | 4/5 | 1 |
| low_resolution_scan | 3 | 0/3 | 2/3 | 1/3 | 1 |
| middle_fact | 6 | 2/6 | 4/6 | 4/6 | 2 |
| mixed_scan | 3 | 2/3 | 2/3 | 2/3 | 2 |
| multi_column | 7 | 2/7 | 4/7 | 4/7 | 6 |
| noisy_scan | 3 | 1/3 | 2/3 | 3/3 | 1 |
| notice | 10 | 0/10 | 7/10 | 4/10 | 6 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 0/4 | 3/4 | 2/4 | 1 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 0/2 | 1/2 | 1/2 | 0 |
| pages_25 | 4 | 3/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 2/2 | 1/2 | 1 |
| png | 5 | 1/5 | 4/5 | 4/5 | 1 |
| pptx | 2 | 1/2 | 1/2 | 1/2 | 1 |
| presentation | 2 | 1/2 | 1/2 | 1/2 | 1 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 9/20 | 18/20 | 16/20 | 3 |
| rotated_page | 1 | 0/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 1/4 | 4/4 | 3/4 | 1 |
| simple_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 0/3 | 3/3 | 2/3 | 0 |
| statement | 3 | 1/3 | 2/3 | 2/3 | 1 |
| stream_order | 3 | 0/3 | 0/3 | 0/3 | 6 |
| table | 41 | 13/41 | 35/41 | 28/41 | 8 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 0/4 | 3/4 | 3/4 | 1 |
| xlsx | 2 | 0/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 4.26 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.35 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.58 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 2.16 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.11 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.26 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 1.91 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 1.92 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 25.31 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 57.95 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 2.76 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 2.15 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 3.82 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 2.04 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 2.33 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 1.32 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 747.9 ms |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 3.27 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 3.19 s |
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
| watershed-monitoring-report-100p | model_failed | 1/8 (12.5%) | – | – | – | – |
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
| `total_ms` | 76 | 38.94 s | 96.03 s | 142.34 s |
| `extraction_wall_ms` | 76 | 7.3 ms | 3.99 s | 58.19 s |
| `worker_total_ms` | 76 | 5.2 ms | 3.99 s | 58.19 s |
| `worker_snapshot_ms` | 76 | 0.21 ms | 0.54 ms | 1.3 ms |
| `worker_parse_ms` | 76 | 2.2 ms | 38.4 ms | 76.1 ms |
| `worker_analysis_ms` | 76 | 0.63 ms | 15.8 ms | 58.4 ms |
| `worker_render_ms` | 76 | 0.00 ms | 242.2 ms | 3.61 s |
| `worker_image_decode_ms` | 76 | 0.00 ms | 68.4 ms | 101.0 ms |
| `worker_ocr_ms` | 76 | 0.00 ms | 3.82 s | 57.95 s |
| `worker_ocr_encode_ms` | 76 | 0.00 ms | 358.8 ms | 5.80 s |
| `worker_ocr_engine_ms` | 76 | 0.00 ms | 3.31 s | 51.97 s |
| `worker_vision_ms` | 76 | 0.00 ms | 9.1 ms | 14.5 ms |
| `analyze_wall_ms` | 76 | 38.79 s | 89.43 s | 103.21 s |
| `distill_ms` | 76 | 0.88 ms | 29.6 ms | 153.1 ms |
| `prompt_ms` | 76 | 0.00 ms | 0.00 ms | 0.00 ms |
| `inference_ms` | 76 | 38.78 s | 89.35 s | 102.99 s |
| `prefill_ms` | 76 | 25.65 s | 78.53 s | 92.86 s |
| `generation_ms` | 76 | 10.69 s | 16.18 s | 24.08 s |
| `validation_ms` | 76 | 12.5 ms | 52.0 ms | 74.7 ms |
| `naming_ms` | 76 | 1.0 ms | 7.7 ms | 11.5 ms |

Over the 76 documents that completed. The other 1 (failed, or not scored) are left out: a failed document's time is how long it took to fail.

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 76 | 81.0 tok/s | 89.0 tok/s | 91.6 tok/s |
| `generation_tok_per_s` | 76 | 13.2 tok/s | 14.3 tok/s | 14.6 tok/s |
| `prompt_tokens` | 76 | 2144 | 5991 | 7188 |
| `cached_tokens` | 76 | 46 | 1199 | 1881 |
| `generated_tokens` | 76 | 141 | 204 | 302 |
| `estimated_prompt_tokens` | 76 | 2531 | 6358 | 6926 |
| `model_requests` | 76 | 1 | 1 | 1 |
| `redistillations` | 76 | 0 | 0 | 1 |
| `source_characters` | 76 | 2459 | 111127 | 171445 |
| `digest_characters` | 76 | 3028 | 14185 | 14388 |
| `prompt_characters` | 76 | 8037 | 19339 | 19542 |
| `worker_ocr_pages` | 76 | 0 | 2 | 25 |
| `worker_ocr_passes` | 76 | 0 | 2 | 25 |
| `worker_orientation_passes` | 76 | 0 | 2 | 25 |
| `worker_rendered_pixels` | 76 | 0 | 16830000 | 210375000 |

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Model server | 3481 MB | 5361 MB | mixed-signature-region |
| Benchmark (engine) | 18 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (51)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice of Rent Increase for Cresthaven Court Holdings LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Invoice from Quillon Ridge Bakery, Inc.pdf` · trap: party Quillon Ridge Bakery, Inc. (bill-to customer) · **filed without review**
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-03-01 Statement - Kingsfold Community Bank.pdf` · trap: date 2026-03-01 (start of the statement period) · **filed without review**
- **second-amendment**: expected `2025-09-29 Second Amendment to Software License and Support Agreement with Umberlee Imaging Software Inc.pdf`, got `2025-09-29 Second Amendment to Software License and Support with Umberlee Imaging Software Inc.pdf` · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **notice-of-default**: expected `2025-10-14 Notice of Default for Glasswing Ceramics LLC.pdf`, got `2025-10-14 Notice of Default and Reservation of Rights - Basalt Commercial Credit Corp.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **offer-letter**: expected `2026-02-10 Offer Letter to Dalia Brennagh.pdf`, got `2026-03-16 Offer of Employment - Corvid Data Systems Inc.pdf` · trap: date 2026-03-16 (proposed start date) · **filed without review**
- **prior-authorization**: expected `2026-03-09 Notice of Prior Authorization Approval for Florian Okonkwo.pdf`, got `2026-03-09 Notice of Prior Authorization Approval - Silverlode Health Partners.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Statement of Benefits - Paloma Achterberg.pdf` · **filed without review**
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 CAPITAL CALL NOTICE No for Ravensmoor Growth Partners III, L.P.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter between Silverbirch Ceramics Inc and Halloran Ostrowski CPAs LLP.pdf` · **filed without review**
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document - Moonrake Paper & Packaging Co.pdf` · review: TYPE_UNSUPPORTED
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-06-18 Commercial Package Policy Declarations - Northfell Mutual Insurance Company.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2025-12-31 2025 Annual Report - Rookwood Precision Metals Corporation.pdf` · **filed without review**
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Vendor Registration Form - Emberglow Coatings Ltd.pdf` · trap: party Highmeadow Casualty Company (insurance carrier) · **filed without review**
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes - Cairnfield Cooperative Grocers.pdf` · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report - Haverford Family Collection.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-10 Maintenance Record - Highmeadow Air Charter LLC.pdf` · trap: date 2026-08-10 (first maintenance entry) · **filed without review**
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `2026-09-18 Annual Report - Tamsin Valley Farmers Cooperative.pdf` · **filed without review**
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 via Email and Certified Mail, Return Receipt Requested - Fernvale Cider Works LLC.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Approval of Minutes - Summerhill Robotics, Inc.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-04-24 Fieldnote Product Launch Plan - Quarrystone Audio Labs Inc.pptx` · trap: date 2026-04-24 (design freeze) · **filed without review**
- **payroll-register**: expected `2026-07-15 Payroll Register for Wrenfield Bakehouse LLC.xlsx`, got `2026-07-15 Payroll Register - Wrenfield Bakehouse LLC.xlsx` · **filed without review**
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Document - Lowmarsh Farm Equipment Co.csv` · review: TYPE_UNSUPPORTED
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 Harvest Log - Rowanbrae Vineyards.xlsx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document between Pemberly Falls Distribution Co and Basalt Telemetry Inc.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing - Gilchrist Harbor Marine Supply, LLC.txt` · **filed without review**
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-skewed-notice**: expected `2026-08-03 Notice of Nonrenewal of Lease for Wendell Okafor.tiff`, got `2026-08-03 Notice of Nonrenewal of Lease for Harrowgate Apartments LLC.tiff` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document - Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED
- **scan-faint-letter**: expected `2026-03-02 Letter of Resignation from Mireille Saltonstall.png`, got `2026-03-02 Document from Mireille Saltonstall.png` · review: TYPE_UNSUPPORTED
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-08-27 Document - Calder Way Family Medicine.png` · review: TYPE_UNSUPPORTED
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter - Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **meeting-notice-columns**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting); party Ironbridge Community Credit Union (merger partner) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **meeting-notice-interleaved**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting); party Ironbridge Community Credit Union (merger partner) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **meeting-notice-reversed**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting); party Ironbridge Community Credit Union (merger partner) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **rate-confirmation-rotated**: expected `2026-06-08 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf`, got `2026-06-08 Invoice between Copperline Freight Brokerage LLC and Halden Ridge Trucking Inc.pdf` · **filed without review**
- **inspection-log-ruled-2p**: expected `2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf`, got `2026-05-19 Fire Extinguisher Inspection Report - Emberwatch Fire Protection Inc.pdf` · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Price List.pdf` · **filed without review**
- **invoice-label-above**: expected `2026-05-11 Invoice from Quarrystone Signs & Graphics.pdf`, got `2026-05-11 Invoice from Bellhaven Physical Therapy PLLC.pdf` · trap: party Bellhaven Physical Therapy PLLC (bill-to customer) · review: DATE_AMBIGUOUS
- … and 11 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
