# InternBench: replay run

- **Run:** 2026-10-28T09:48:09Z · commit `77a5df487d`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.10 GB · sha256 `3793e1e6e226`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T09:15:26Z at commit `77a5df487d` · sha256 `42827c4245`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 40/77 (51.9%) |
| Document type | 55/77 (71.4%) |
| Date | 69/77 (89.6%) |
| Date role (when the reviewed date was chosen) | 39/66 (59.1%) |
| Parties | 70/77 (90.9%) |
| Relation word (when parties were named) | 62/77 (80.5%) |
| Party in the right role (when parties were named) | 59/69 (85.5%) |
| Ready / review routing | 4/57 (7.0%) |
| Description has every fact | 17/77 (22.1%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 76/77 (98.7%) |
| Description fact coverage | 44.8% (mean of 77) |
| Description specificity | 94.4% (mean of 77) |
| Evidence recall (model's quotes) | 69.7% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 98.7% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 12 |
| Complex-document filename accuracy | 53.6% | 56 |
| Description completeness | 44.8% | 77 |
| Unsupported-fact rate (documents) | 97.4% | 77 |
| Review rate | 98.7% | 77 |
| Evidence recall | 69.7% | 77 |
| Total latency p50 | 21.18 s | 77 |
| Total latency p95 | 28.29 s | 77 |
| Generation latency p50 | 5.97 s | 77 |
| Generation latency p95 | 7.78 s | 77 |
| Generated tokens p50 | 92 | 77 |
| Generated tokens p95 | 117 | 77 |
| Prompt tokens p50 | 1268 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 77 |
| Trap date chosen | 6 of 77 |
| Forbidden party named | 3 of 77 |
| Spurious parties named | 7 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 277 (0.0%) |
| Right name sent to review anyway | 27 of 53 |
| Sent to review for an unsupported fact | 75 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 0/1 | 100.0% | 26.17 s | 27.36 s |
| annual_report | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 20.28 s | 28.29 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 25.69 s | 25.69 s |
| certificate | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 24.93 s | 24.93 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 23.20 s | 23.20 s |
| condition_report | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 21.19 s | 21.19 s |
| contract | 16 | 13/16 | 16/16 | 16/16 | 0/12 | 100.0% | 24.18 s | 107.90 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 17.28 s | 17.28 s |
| form | 5 | 1/5 | 3/5 | 5/5 | 0/4 | 100.0% | 20.96 s | 22.68 s |
| inspection_report | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 14.36 s | 14.36 s |
| insurance | 2 | 1/2 | 1/2 | 2/2 | 0/2 | 100.0% | 23.40 s | 27.19 s |
| invoice | 7 | 4/7 | 6/7 | 7/7 | 0/5 | 100.0% | 17.74 s | 19.13 s |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0/4 | 100.0% | 19.63 s | 23.38 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 27.77 s | 27.77 s |
| maintenance_record | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 21.29 s | 21.29 s |
| minutes | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 23.82 s | 23.82 s |
| newsletter | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 19.98 s | 19.98 s |
| notice | 10 | 7/10 | 10/10 | 8/10 | 0/8 | 100.0% | 18.64 s | 25.11 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 17.94 s | 17.94 s |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 18.74 s | 24.64 s |
| price_list | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 19.08 s | 19.08 s |
| purchase_order | 2 | 0/2 | 2/2 | 2/2 | 0/1 | 50.0% | 20.02 s | 22.48 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 21.70 s | 21.70 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 17.66 s | 17.66 s |
| receipt | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 17.62 s | 17.62 s |
| remittance_advice | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 20.21 s | 20.21 s |
| report | 2 | 1/2 | 2/2 | 2/2 | 0/2 | 100.0% | 25.31 s | 27.37 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 24.12 s | 24.12 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 24.96 s | 24.96 s |
| statement | 3 | 1/3 | 2/3 | 3/3 | 1/3 | 100.0% | 19.87 s | 20.52 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 17.28 s | 17.28 s |
| mixed | 3 | 2/3 | 2/3 | 3/3 | – | 100.0% | 21.91 s | 27.36 s |
| native | 47 | 24/47 | 42/47 | 41/47 | 0/44 | 100.0% | 20.28 s | 28.17 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 18.02 s | 18.02 s |
| office | 5 | 2/5 | 5/5 | 4/5 | 0/5 | 100.0% | 24.12 s | 24.64 s |
| scan | 16 | 10/16 | 14/16 | 16/16 | 4/4 | 93.8% | 20.96 s | 107.90 s |
| sheet | 3 | 1/3 | 3/3 | 3/3 | 0/2 | 100.0% | 25.31 s | 27.77 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 21.11 s | 21.11 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 15/37 | 33/37 | 33/37 | 3/26 | 97.3% | 19.13 s | 25.31 s |
| 2-4 | 24 | 15/24 | 22/24 | 22/24 | 1/17 | 100.0% | 21.91 s | 27.36 s |
| 5-9 | 4 | 2/4 | 3/4 | 4/4 | 0/4 | 100.0% | 18.74 s | 24.96 s |
| 10-24 | 4 | 2/4 | 4/4 | 4/4 | 0/3 | 100.0% | 24.64 s | 58.59 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 0/3 | 100.0% | 24.75 s | 107.90 s |
| 50-99 | 2 | 1/2 | 1/2 | 2/2 | 0/2 | 100.0% | 27.19 s | 29.26 s |
| 100+ | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 27.37 s | 28.29 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 8/12 | 11/12 | 11/12 | 0/10 | 0 of 12 | 27.19 s | 107.90 s | 5.74 s | 6.20 s |
| complex | 56 | 30/56 | 50/56 | 51/56 | 0/46 | 0 of 56 | 22.10 s | 28.29 s | 5.96 s | 8.12 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 0 |
| competing_dates | 67 | 34/67 | 59/67 | 60/67 | 9 |
| complex_pdf | 5 | 4/5 | 5/5 | 4/5 | 0 |
| contract | 20 | 15/20 | 18/20 | 20/20 | 1 |
| csv | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 4/7 | 6/7 | 7/7 | 1 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 4/10 | 8/10 | 8/10 | 3 |
| form | 7 | 2/7 | 4/7 | 7/7 | 2 |
| healthcare | 3 | 1/3 | 2/3 | 3/3 | 1 |
| hr | 4 | 3/4 | 3/4 | 4/4 | 0 |
| image_only_scan | 16 | 10/16 | 14/16 | 16/16 | 2 |
| image_region | 1 | 0/1 | 0/1 | 1/1 | 0 |
| information_dense | 10 | 7/10 | 9/10 | 9/10 | 2 |
| invoice | 7 | 4/7 | 6/7 | 7/7 | 1 |
| irrelevant_names | 17 | 8/17 | 15/17 | 14/17 | 3 |
| key_value | 11 | 7/11 | 9/11 | 11/11 | 1 |
| layout_parties | 11 | 5/11 | 10/11 | 11/11 | 1 |
| letter | 5 | 2/5 | 5/5 | 5/5 | 0 |
| low_resolution_scan | 3 | 1/3 | 3/3 | 3/3 | 0 |
| middle_fact | 6 | 2/6 | 4/6 | 5/6 | 3 |
| mixed_scan | 3 | 2/3 | 2/3 | 3/3 | 0 |
| multi_column | 7 | 5/7 | 7/7 | 6/7 | 0 |
| noisy_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 7/10 | 10/10 | 8/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 2/4 | 3/4 | 4/4 | 1 |
| pages_10 | 3 | 2/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 1/2 | 2/2 | 1/2 | 1 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 1/2 | 2/2 | 1 |
| png | 5 | 2/5 | 4/5 | 5/5 | 1 |
| pptx | 2 | 1/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 0/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 10/20 | 18/20 | 18/20 | 1 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 1/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 1/3 | 2/3 | 3/3 | 1 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 23/41 | 37/41 | 36/41 | 7 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 1/4 | 4/4 | 2/4 | 2 |
| xlsx | 2 | 1/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 8.57 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 3.63 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.70 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 4.29 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.73 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.76 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 3.93 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.72 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 42.36 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 89.10 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 5.61 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.79 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 6.61 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 4.53 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 4.47 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 3.07 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.99 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 5.96 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 5.45 s |
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
| `total_ms` | 77 | 21.18 s | 28.29 s | 107.90 s |
| `extraction_wall_ms` | 77 | 12.8 ms | 7.31 s | 89.85 s |
| `worker_total_ms` | 77 | 12.0 ms | 7.31 s | 89.85 s |
| `worker_snapshot_ms` | 77 | 0.52 ms | 1.9 ms | 3.8 ms |
| `worker_parse_ms` | 77 | 4.1 ms | 59.6 ms | 163.6 ms |
| `worker_analysis_ms` | 77 | 0.80 ms | 42.2 ms | 111.9 ms |
| `worker_render_ms` | 77 | 0.00 ms | 963.4 ms | 6.40 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 88.5 ms | 269.2 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 6.61 s | 89.10 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 525.9 ms | 6.67 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 5.50 s | 81.81 s |
| `worker_vision_ms` | 77 | 0.00 ms | 7.7 ms | 14.2 ms |
| `analyze_wall_ms` | 77 | 18.81 s | 27.76 s | 29.18 s |
| `distill_ms` | 77 | 5.1 ms | 192.0 ms | 684.5 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.12 ms | 0.14 ms |
| `inference_ms` | 77 | 18.77 s | 27.29 s | 28.61 s |
| `prefill_ms` | 77 | 12.79 s | 21.37 s | 22.64 s |
| `generation_ms` | 77 | 5.97 s | 7.78 s | 8.66 s |
| `validation_ms` | 77 | 33.1 ms | 353.5 ms | 695.2 ms |
| `naming_ms` | 77 | 1.1 ms | 20.9 ms | 26.0 ms |
| `index_ms` | 77 | 3.9 ms | 132.0 ms | 626.9 ms |
| `retrieval_ms` | 77 | 0.05 ms | 6.5 ms | 22.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 100.1 tok/s | 111.7 tok/s | 114.3 tok/s |
| `generation_tok_per_s` | 77 | 14.9 tok/s | 16.6 tok/s | 17.6 tok/s |
| `prompt_tokens` | 77 | 1268 | 2263 | 2283 |
| `cached_tokens` | 77 | 0 | 0 | 811 |
| `generated_tokens` | 77 | 92 | 117 | 124 |
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
| Parser worker | 12 MB | 740 MB | scan-agreement-10p |
| Model server | 1958 MB | 1962 MB | benefits-change-checkbox-form |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (37)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice of Rent Increase from Tamarack Property Management Co.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Bill from Halvorsen Fixture Works LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **invoice-layout-only**: expected `2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf`, got `2026-02-03 Invoice from Ironvale Marine Coatings Co.pdf` · trap: date 2026-02-03 (order date) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-03-01 Account Summary from Kingsfold Community Bank.pdf` · trap: date 2026-03-01 (start of the statement period) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **purchase-order**: expected `2026-01-29 Purchase Order from Lamplighter Robotics Inc.pdf`, got `2026-01-29 Document from Lamplighter Robotics Inc.pdf` · review: TYPE_UNSUPPORTED
- **services-agreement**: expected `2026-01-12 Master Services Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf`, got `2026-01-12 Agreement between Cedarmark Cloud Services Inc and Pinehollow Credit Union.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Meadowlark Health Plan for Paloma Achterberg.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 Capital Call Notice - Ravensmoor Growth GP III, LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 letter from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_MISSING
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document from Moonrake Paper & Packaging Co.pdf` · review: TYPE_UNSUPPORTED
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Document from Hartwell County.pdf` · review: TYPE_UNSUPPORTED
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Document from Cairnfield Cooperative Grocers.pdf` · review: TYPE_UNSUPPORTED
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report for Delphine Moncrieff, Associate Conservator of Paintings.pdf` · trap: party Delphine Moncrieff (conservator who examined the work) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf` · trap: party Arlo Sinclair-Ray (inspector who signed the return to service) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `2026-09-18 Annual Report for Wainwright Lindqvist LLP.pdf` · trap: party Wainwright Lindqvist LLP (independent auditors) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Document - Fernvale Cider Works LLC.docx` · review: TYPE_UNSUPPORTED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review for Prepared for Pemberly Falls Distribution Co.pptx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Document - Lowmarsh Farm Equipment Co.csv` · review: TYPE_UNSUPPORTED
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 Summary for Rowanbrae Vineyards.xlsx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document from Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing - Tavistock Boatworks, Inc.txt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document from Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Quote for propeller overhaul from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-01-01 Document - Calder Way Family Medicine.png` · trap: date 2026-01-01 (insurance coverage effective date) · review: TYPE_UNSUPPORTED
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Tidewater Ledger - The Tidewater Ledger.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **agreement-two-column-footnotes**: expected `2026-03-02 Seed Production and Supply Agreement between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf`, got `2026-03-02 Document between Larkhaven Seed Company and Prairie Wren Growers Cooperative.pdf` · review: TYPE_UNSUPPORTED
- **invoice-boxed-grid**: expected `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf`, got `2026-07-09 Document from Whitlock & Sons Plumbing LLC.pdf` · review: TYPE_UNSUPPORTED
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `Medical Plan for Westbrook Kettle Brewing Co.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Document from Harrowmere Mutual Insurance Company - Claims Intake.pdf` · review: TYPE_UNSUPPORTED
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-07-17 Document from Tolliver Grain & Feed Cooperative RemitTANCE Advice.png` · review: TYPE_UNSUPPORTED
- **mixed-signature-region**: expected `2026-08-21 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf`, got `First Amendment to Software License Agreement between Corvane Analytics Inc and Pellworth County Water Authority.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-04-01 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf` · trap: date 2026-04-01 (policy effective date) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **msa-effective-date-in-definitions-12p**: expected `2026-03-02 Master Services Agreement between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf`, got `2026-03-02 Document between Halcyon Ridge Logistics, Inc and Quillmont Systems Integration LLC.pdf` · review: TYPE_UNSUPPORTED
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-06-24 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · trap: date 2026-06-24 (countersignature date) · review: TYPE_UNSUPPORTED, TYPE_INFERRED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
