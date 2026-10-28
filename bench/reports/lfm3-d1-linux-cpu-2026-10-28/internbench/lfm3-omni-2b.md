# InternBench: replay run

- **Run:** 2026-10-28T09:13:52Z · commit `77a5df487d`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.10 GB · sha256 `9f8d30e9d4fd`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T08:40:47Z at commit `77a5df487d` · sha256 `5746a55de8`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 22/77 (28.6%) |
| Document type | 55/77 (71.4%) |
| Date | 33/77 (42.9%) |
| Date role (when the reviewed date was chosen) | 20/32 (62.5%) |
| Parties | 70/77 (90.9%) |
| Relation word (when parties were named) | 58/77 (75.3%) |
| Party in the right role (when parties were named) | 56/66 (84.9%) |
| Ready / review routing | 4/57 (7.0%) |
| Description has every fact | 17/77 (22.1%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 74/77 (96.1%) |
| Description fact coverage | 42.4% (mean of 77) |
| Description specificity | 93.5% (mean of 77) |
| Evidence recall (model's quotes) | 45.7% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 100.0% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 41.7% | 12 |
| Complex-document filename accuracy | 32.1% | 56 |
| Description completeness | 42.4% | 77 |
| Unsupported-fact rate (documents) | 100.0% | 77 |
| Review rate | 100.0% | 77 |
| Evidence recall | 45.7% | 77 |
| Total latency p50 | 20.82 s | 77 |
| Total latency p95 | 29.97 s | 77 |
| Generation latency p50 | 6.01 s | 77 |
| Generation latency p95 | 7.35 s | 77 |
| Generated tokens p50 | 87 | 77 |
| Generated tokens p95 | 103 | 77 |
| Prompt tokens p50 | 1268 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 77 |
| Trap date chosen | 0 of 77 |
| Forbidden party named | 3 of 77 |
| Spurious parties named | 8 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 269 (0.0%) |
| Right name sent to review anyway | 14 of 53 |
| Sent to review for an unsupported fact | 77 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 0/1 | 100.0% | 22.76 s | 28.70 s |
| annual_report | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 16.56 s | 24.85 s |
| bill_of_lading | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 23.64 s | 23.64 s |
| certificate | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 26.68 s | 26.68 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 29.03 s | 29.03 s |
| condition_report | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 18.46 s | 18.46 s |
| contract | 16 | 10/16 | 12/16 | 15/16 | 0/12 | 100.0% | 24.40 s | 114.89 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 18.37 s | 18.37 s |
| form | 5 | 0/5 | 2/5 | 4/5 | 0/4 | 100.0% | 19.69 s | 20.99 s |
| inspection_report | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 14.21 s | 14.21 s |
| insurance | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 21.15 s | 31.31 s |
| invoice | 7 | 0/7 | 0/7 | 6/7 | 0/5 | 100.0% | 16.50 s | 20.39 s |
| letter | 5 | 2/5 | 4/5 | 4/5 | 0/4 | 100.0% | 20.32 s | 22.55 s |
| log | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 29.72 s | 29.72 s |
| maintenance_record | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 20.82 s | 20.82 s |
| minutes | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 24.85 s | 24.85 s |
| newsletter | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 21.48 s | 21.48 s |
| notice | 10 | 6/10 | 7/10 | 9/10 | 0/8 | 100.0% | 19.53 s | 24.24 s |
| payroll | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 16.67 s | 16.67 s |
| presentation | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 17.50 s | 23.81 s |
| price_list | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 20.02 s | 20.02 s |
| purchase_order | 2 | 0/2 | 0/2 | 2/2 | 0/1 | 100.0% | 17.45 s | 19.36 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 24.05 s | 24.05 s |
| rate_confirmation | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 19.44 s | 19.44 s |
| receipt | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 100.0% | 16.18 s | 16.18 s |
| remittance_advice | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 100.0% | 23.06 s | 23.06 s |
| report | 2 | 0/2 | 1/2 | 2/2 | 0/2 | 100.0% | 24.87 s | 28.44 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 25.23 s | 25.23 s |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 22.19 s | 22.19 s |
| statement | 3 | 0/3 | 0/3 | 3/3 | 1/3 | 100.0% | 18.60 s | 19.53 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 18.37 s | 18.37 s |
| mixed | 3 | 2/3 | 2/3 | 3/3 | – | 100.0% | 24.40 s | 28.70 s |
| native | 47 | 13/47 | 17/47 | 41/47 | 0/44 | 100.0% | 20.02 s | 29.65 s |
| ocr_corrupted | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 20.39 s | 20.39 s |
| office | 5 | 1/5 | 3/5 | 4/5 | 0/5 | 100.0% | 23.81 s | 25.23 s |
| scan | 16 | 6/16 | 8/16 | 16/16 | 4/4 | 100.0% | 22.63 s | 114.89 s |
| sheet | 3 | 0/3 | 1/3 | 3/3 | 0/2 | 100.0% | 24.87 s | 29.72 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 19.54 s | 19.54 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 6/37 | 13/37 | 31/37 | 3/26 | 100.0% | 19.53 s | 25.23 s |
| 2-4 | 24 | 10/24 | 13/24 | 24/24 | 1/17 | 100.0% | 21.67 s | 29.03 s |
| 5-9 | 4 | 1/4 | 2/4 | 4/4 | 0/4 | 100.0% | 17.50 s | 22.19 s |
| 10-24 | 4 | 2/4 | 2/4 | 4/4 | 0/3 | 100.0% | 23.81 s | 61.70 s |
| 25-49 | 4 | 3/4 | 3/4 | 4/4 | 0/3 | 100.0% | 29.65 s | 114.89 s |
| 50-99 | 2 | 0/2 | 0/2 | 1/2 | 0/2 | 100.0% | 28.19 s | 31.31 s |
| 100+ | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 24.85 s | 28.44 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 5/12 | 5/12 | 11/12 | 0/10 | 0 of 12 | 28.19 s | 114.89 s | 6.14 s | 7.84 s |
| complex | 56 | 18/56 | 26/56 | 49/56 | 0/46 | 0 of 56 | 21.48 s | 29.97 s | 6.35 s | 7.36 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 0 |
| competing_dates | 67 | 18/67 | 28/67 | 61/67 | 2 |
| complex_pdf | 5 | 2/5 | 2/5 | 4/5 | 0 |
| contract | 20 | 13/20 | 15/20 | 19/20 | 0 |
| csv | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 0/7 | 0/7 | 7/7 | 0 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 1 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 2/10 | 4/10 | 10/10 | 0 |
| form | 7 | 0/7 | 2/7 | 6/7 | 0 |
| healthcare | 3 | 1/3 | 2/3 | 3/3 | 0 |
| hr | 4 | 2/4 | 3/4 | 4/4 | 0 |
| image_only_scan | 16 | 6/16 | 8/16 | 16/16 | 0 |
| image_region | 1 | 0/1 | 0/1 | 1/1 | 0 |
| information_dense | 10 | 4/10 | 4/10 | 9/10 | 0 |
| invoice | 7 | 0/7 | 0/7 | 6/7 | 1 |
| irrelevant_names | 17 | 4/17 | 6/17 | 14/17 | 1 |
| key_value | 11 | 1/11 | 2/11 | 10/11 | 1 |
| layout_parties | 11 | 0/11 | 0/11 | 8/11 | 2 |
| letter | 5 | 2/5 | 4/5 | 4/5 | 1 |
| low_resolution_scan | 3 | 0/3 | 0/3 | 3/3 | 0 |
| middle_fact | 6 | 1/6 | 1/6 | 5/6 | 0 |
| mixed_scan | 3 | 2/3 | 2/3 | 3/3 | 0 |
| multi_column | 7 | 4/7 | 5/7 | 6/7 | 0 |
| noisy_scan | 3 | 1/3 | 1/3 | 3/3 | 0 |
| notice | 10 | 6/10 | 7/10 | 9/10 | 0 |
| ocr_corrupted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 0/4 | 0/4 | 4/4 | 0 |
| pages_10 | 3 | 2/3 | 2/3 | 3/3 | 0 |
| pages_100 | 2 | 0/2 | 0/2 | 2/2 | 0 |
| pages_25 | 4 | 3/4 | 3/4 | 4/4 | 0 |
| pages_5 | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pages_50 | 2 | 0/2 | 0/2 | 1/2 | 0 |
| png | 5 | 1/5 | 2/5 | 5/5 | 0 |
| pptx | 2 | 0/2 | 0/2 | 2/2 | 0 |
| presentation | 2 | 0/2 | 0/2 | 2/2 | 0 |
| purchase_order | 2 | 0/2 | 0/2 | 2/2 | 0 |
| referenced_agreement | 20 | 10/20 | 15/20 | 17/20 | 1 |
| rotated_page | 1 | 0/1 | 0/1 | 1/1 | 0 |
| rotated_scan | 4 | 2/4 | 2/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 0/1 | 0/1 | 0 |
| sow | 1 | 1/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 3 | 0/3 | 1/3 | 3/3 | 0 |
| statement | 3 | 0/3 | 0/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 10/41 | 13/41 | 39/41 | 1 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 0/4 | 0/4 | 4/4 | 0 |
| xlsx | 2 | 0/2 | 0/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 8.66 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 3.19 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.19 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 4.03 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.97 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 2.10 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 4.00 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.34 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 43.05 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 93.48 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 7.02 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.39 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 8.10 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 5.18 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 5.15 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 3.99 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 2.46 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 6.81 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 5.51 s |
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
| `total_ms` | 77 | 20.82 s | 29.97 s | 114.89 s |
| `extraction_wall_ms` | 77 | 11.8 ms | 9.13 s | 94.26 s |
| `worker_total_ms` | 77 | 10.9 ms | 9.13 s | 94.26 s |
| `worker_snapshot_ms` | 77 | 0.34 ms | 2.2 ms | 4.2 ms |
| `worker_parse_ms` | 77 | 3.8 ms | 80.3 ms | 146.1 ms |
| `worker_analysis_ms` | 77 | 0.75 ms | 47.4 ms | 112.8 ms |
| `worker_render_ms` | 77 | 0.00 ms | 1.13 s | 6.94 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 132.4 ms | 236.5 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 8.10 s | 93.48 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 554.4 ms | 7.22 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 6.88 s | 85.60 s |
| `worker_vision_ms` | 77 | 0.00 ms | 13.1 ms | 17.2 ms |
| `analyze_wall_ms` | 77 | 19.51 s | 29.47 s | 31.08 s |
| `distill_ms` | 77 | 5.1 ms | 176.3 ms | 657.5 ms |
| `prompt_ms` | 77 | 0.05 ms | 0.10 ms | 0.13 ms |
| `inference_ms` | 77 | 19.48 s | 28.94 s | 30.73 s |
| `prefill_ms` | 77 | 13.31 s | 22.57 s | 23.95 s |
| `generation_ms` | 77 | 6.01 s | 7.35 s | 7.84 s |
| `validation_ms` | 77 | 11.7 ms | 101.0 ms | 285.0 ms |
| `naming_ms` | 77 | 1.2 ms | 20.9 ms | 30.9 ms |
| `index_ms` | 77 | 4.5 ms | 118.3 ms | 708.7 ms |
| `retrieval_ms` | 77 | 0.05 ms | 6.6 ms | 12.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 98.3 tok/s | 106.1 tok/s | 109.5 tok/s |
| `generation_tok_per_s` | 77 | 14.7 tok/s | 16.2 tok/s | 16.8 tok/s |
| `prompt_tokens` | 77 | 1268 | 2263 | 2283 |
| `cached_tokens` | 77 | 0 | 0 | 811 |
| `generated_tokens` | 77 | 87 | 103 | 116 |
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
| Parser worker | 12 MB | 717 MB | scan-lease-25p |
| Model server | 1959 MB | 1963 MB | scan-bill-of-lading |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (55)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `Notice of Rent Increase from Tamarack Property Management Co.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `Bill from Halvorsen Fixture Works LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **invoice-layout-only**: expected `2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf`, got `Invoice from Ironvale Marine Coatings Co.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `Account Summary from Kingsfold Community Bank.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, DESCRIPTION_UNSUPPORTED
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `Document - Lamplighter Robotics Inc.pdf` · review: TYPE_UNSUPPORTED, DATE_MISSING
- **services-agreement**: expected `2026-01-12 Master Services Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf`, got `2026-01-12 Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `Meadowlark Health Plan from Meadowlark Health Plan.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, PARTY_UNSUPPORTED
- **promissory-note**: expected `2025-09-08 Promissory Note between Foxhallow Outfitters LLC and Ashgrove Capital Partners LLC.pdf`, got `Promissory Note between Foxhallow Outfitters LLC and Ashgrove Capital Partners LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Document from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_UNSUPPORTED
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `Document - Keziah Ambrose.pdf` · trap: party Keziah Ambrose (recipient's owner named in the attention line) · review: TYPE_UNSUPPORTED, DATE_MISSING
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report from Rookwood Precision Metals Corporation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `Document from vendors@hartwellcounty.example.pdf` · review: TYPE_UNSUPPORTED, DATE_MISSING, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID
- **change-order-form**: expected `2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf`, got `Change Order between Briarport Library District and Stonebridge Builders Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, DESCRIPTION_UNSUPPORTED
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `Document from Cairnfield Cooperative Grocers.pdf` · review: TYPE_UNSUPPORTED, DATE_MISSING
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `Loan Condition Report from Saltash Point Museum of Art.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `Aircraft Maintenance Record from Highmeadow Air Charter LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, PARTY_UNSUPPORTED
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `Credit Agreement between Celeste Boateng and Pemberly Falls Trust Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `Annual Report from Tamsin Valley Farmers Cooperative.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Document - Fernvale Cider Works LLC.docx` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors for Celeste Fontaine.docx` · trap: party Celeste Fontaine (director) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `Quarterly Business Review for Pemberly Falls Distribution Co.pptx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **payroll-register**: expected `2026-07-15 Payroll Register for Wrenfield Bakehouse LLC.xlsx`, got `Payroll Register for Wrenfield Bakehouse LLC.xlsx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Document - Lowmarsh Farm Equipment Co.csv` · review: TYPE_UNSUPPORTED
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `Summary for Rowanbrae Vineyards.xlsx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, DESCRIPTION_INVALID
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document - Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-30 Notice of Hearing - Tavistock Boatworks, Inc.txt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-rotated-90-invoice**: expected `2026-04-14 Invoice from Wrenfield Restaurant Supply Co.png`, got `Invoice from Wrenfield Restaurant Supply Co.png` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `Purchase Order from Ridgewell Textiles Ltd.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, DATE_MISSING
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `Statement of Account from Brindle Paper & Janitorial Supply.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **ocr-corrupted-invoice**: expected `2026-03-10 Invoice from Wexcombe Millwork Co.pdf`, got `Invoice from Wexcombe Millwork Co.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quote for propeller overhaul from Highmeadow Air Charter LLC.tiff` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-08-27 Document - Calder Way Family Medicine.png` · review: TYPE_UNSUPPORTED
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `Tidewater Ledger - The Tidewater Ledger.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING
- **agreement-two-column-footnotes**: expected `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`, got `2026-03-02 Document between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` · review: TYPE_UNSUPPORTED
- **rate-confirmation-rotated**: expected `2026-06-08 Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf`, got `Carrier Rate Confirmation from Copperline Freight Brokerage LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, DESCRIPTION_UNSUPPORTED
- **inspection-log-ruled-2p**: expected `2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf`, got `Fire Extinguisher Inspection Report from Emberwatch Fire Protection Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_MISSING, DESCRIPTION_UNSUPPORTED
- … and 15 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
