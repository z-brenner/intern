# InternBench: replay run

- **Run:** 2026-10-28T13:22:00Z · commit `4ad2f36b4c`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.10 GB · sha256 `9f8d30e9d4fd`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-28T12:48:09Z at commit `4ad2f36b4c` · sha256 `824cbb855f`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 40/77 (51.9%) |
| Document type | 65/77 (84.4%) |
| Date | 63/77 (81.8%) |
| Date role (when the reviewed date was chosen) | 44/62 (71.0%) |
| Parties | 66/77 (85.7%) |
| Relation word (when parties were named) | 59/74 (79.7%) |
| Party in the right role (when parties were named) | 56/69 (81.2%) |
| Ready / review routing | 36/57 (63.2%) |
| Description has every fact | 22/77 (28.6%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 76/77 (98.7%) |
| Description fact coverage | 54.5% (mean of 77) |
| Description specificity | 95.7% (mean of 77) |
| Evidence recall (model's quotes) | 51.7% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 35.1% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 58.3% | 12 |
| Complex-document filename accuracy | 58.9% | 56 |
| Description completeness | 54.5% | 77 |
| Unsupported-fact rate (documents) | 31.2% | 77 |
| Review rate | 35.1% | 77 |
| Evidence recall | 51.7% | 77 |
| Total latency p50 | 21.20 s | 77 |
| Total latency p95 | 32.81 s | 77 |
| Generation latency p50 | 5.75 s | 77 |
| Generation latency p95 | 6.91 s | 77 |
| Generated tokens p50 | 89 | 77 |
| Generated tokens p95 | 105 | 77 |
| Prompt tokens p50 | 1421 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 19 of 77 |
| Trap date chosen | 6 of 77 |
| Forbidden party named | 5 of 77 |
| Spurious parties named | 8 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 307 (0.0%) |
| Right name sent to review anyway | 6 of 53 |
| Sent to review for an unsupported fact | 24 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 1/1 | 33.3% | 24.67 s | 28.06 s |
| annual_report | 2 | 0/2 | 0/2 | 1/2 | 0/2 | 100.0% | 20.12 s | 29.11 s |
| bill_of_lading | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 23.59 s | 23.59 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 25.43 s | 25.43 s |
| claim | 1 | 0/1 | 0/1 | 1/1 | – | 0.0% | 24.63 s | 24.63 s |
| condition_report | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 23.13 s | 23.13 s |
| contract | 16 | 15/16 | 15/16 | 16/16 | 11/12 | 6.2% | 26.47 s | 109.77 s |
| email | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 17.35 s | 17.35 s |
| form | 5 | 1/5 | 3/5 | 4/5 | 3/4 | 40.0% | 21.76 s | 22.24 s |
| inspection_report | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 17.55 s | 17.55 s |
| insurance | 2 | 0/2 | 1/2 | 2/2 | 1/2 | 50.0% | 24.26 s | 32.81 s |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 4/5 | 28.6% | 17.70 s | 19.59 s |
| letter | 5 | 1/5 | 5/5 | 3/5 | 1/4 | 60.0% | 20.51 s | 22.27 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 27.03 s | 27.03 s |
| maintenance_record | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 24.19 s | 24.19 s |
| minutes | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 26.76 s | 26.76 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 19.29 s | 19.29 s |
| notice | 10 | 8/10 | 10/10 | 9/10 | 4/8 | 50.0% | 18.96 s | 24.22 s |
| payroll | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 18.01 s | 18.01 s |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 1/2 | 50.0% | 20.06 s | 24.93 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 19.76 s | 19.76 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 19.84 s | 19.94 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 21.81 s | 21.81 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 19.25 s | 19.25 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 1/1 | 100.0% | 16.18 s | 16.18 s |
| remittance_advice | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 0.0% | 21.20 s | 21.20 s |
| report | 2 | 1/2 | 1/2 | 2/2 | 1/2 | 50.0% | 25.09 s | 30.08 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 23.91 s | 23.91 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 23.40 s | 23.40 s |
| statement | 3 | 1/3 | 3/3 | 3/3 | 2/3 | 0.0% | 20.83 s | 22.43 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 17.35 s | 17.35 s |
| mixed | 3 | 3/3 | 3/3 | 3/3 | – | 33.3% | 22.33 s | 28.06 s |
| native | 47 | 27/47 | 39/47 | 40/47 | 30/44 | 34.0% | 20.51 s | 30.85 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 17.85 s | 17.85 s |
| office | 5 | 1/5 | 5/5 | 4/5 | 3/5 | 40.0% | 23.91 s | 26.47 s |
| scan | 16 | 6/16 | 11/16 | 14/16 | 1/4 | 25.0% | 21.20 s | 109.77 s |
| sheet | 3 | 2/3 | 3/3 | 3/3 | 2/2 | 33.3% | 25.09 s | 27.03 s |
| text | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 18.39 s | 18.39 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 17/37 | 30/37 | 30/37 | 15/26 | 40.5% | 19.59 s | 25.43 s |
| 2-4 | 24 | 15/24 | 23/24 | 21/24 | 13/17 | 25.0% | 22.33 s | 27.16 s |
| 5-9 | 4 | 1/4 | 2/4 | 3/4 | 2/4 | 50.0% | 20.06 s | 23.40 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 0.0% | 24.93 s | 59.05 s |
| 25-49 | 4 | 4/4 | 4/4 | 4/4 | 3/3 | 0.0% | 29.06 s | 109.77 s |
| 50-99 | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 32.81 s | 33.83 s |
| 100+ | 2 | 0/2 | 0/2 | 2/2 | 0/2 | 100.0% | 29.11 s | 30.08 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 7/12 | 8/12 | 12/12 | 6/10 | 1 of 12 | 29.11 s | 109.77 s | 5.92 s | 8.06 s |
| complex | 56 | 33/56 | 46/56 | 49/56 | 31/46 | 13 of 56 | 21.76 s | 32.81 s | 5.82 s | 6.91 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 3/3 | 3/3 | 3/3 | 0 |
| competing_dates | 67 | 34/67 | 54/67 | 59/67 | 8 |
| complex_pdf | 5 | 2/5 | 4/5 | 4/5 | 0 |
| contract | 20 | 18/20 | 18/20 | 20/20 | 1 |
| csv | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_in_table | 7 | 5/7 | 6/7 | 6/7 | 1 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 1 | 0/1 | 0/1 | 1/1 | 1 |
| financial | 10 | 5/10 | 7/10 | 9/10 | 0 |
| form | 7 | 1/7 | 4/7 | 6/7 | 0 |
| healthcare | 3 | 2/3 | 2/3 | 3/3 | 0 |
| hr | 4 | 3/4 | 4/4 | 4/4 | 0 |
| image_only_scan | 16 | 6/16 | 11/16 | 14/16 | 4 |
| image_region | 1 | 1/1 | 1/1 | 1/1 | 0 |
| information_dense | 10 | 6/10 | 6/10 | 10/10 | 0 |
| invoice | 7 | 7/7 | 7/7 | 7/7 | 0 |
| irrelevant_names | 17 | 8/17 | 13/17 | 15/17 | 1 |
| key_value | 11 | 5/11 | 9/11 | 10/11 | 2 |
| layout_parties | 11 | 6/11 | 9/11 | 9/11 | 1 |
| letter | 5 | 1/5 | 5/5 | 3/5 | 2 |
| low_resolution_scan | 3 | 1/3 | 1/3 | 2/3 | 2 |
| middle_fact | 6 | 1/6 | 1/6 | 6/6 | 1 |
| mixed_scan | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_column | 7 | 5/7 | 7/7 | 7/7 | 0 |
| noisy_scan | 3 | 0/3 | 2/3 | 2/3 | 2 |
| notice | 10 | 8/10 | 10/10 | 9/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 1/4 | 2/4 | 4/4 | 1 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 0/2 | 0/2 | 2/2 | 0 |
| pages_25 | 4 | 4/4 | 4/4 | 4/4 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 0/2 | 0/2 | 2/2 | 0 |
| png | 5 | 1/5 | 2/5 | 3/5 | 3 |
| pptx | 2 | 0/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 0/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 13/20 | 17/20 | 18/20 | 3 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 2/4 | 3/4 | 4/4 | 1 |
| simple_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 2/3 | 3/3 | 3/3 | 0 |
| statement | 3 | 1/3 | 3/3 | 3/3 | 0 |
| stream_order | 3 | 3/3 | 3/3 | 3/3 | 0 |
| table | 41 | 22/41 | 32/41 | 36/41 | 5 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 0/4 | 3/4 | 2/4 | 2 |
| xlsx | 2 | 1/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 8.08 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 3.39 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.03 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 4.12 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 3.57 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.90 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 3.44 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 3.65 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 40.44 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 89.76 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 5.11 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.39 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 6.42 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 4.18 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 4.39 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 2.76 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.90 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 5.93 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 4.78 s |
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
| `total_ms` | 77 | 21.20 s | 32.81 s | 109.77 s |
| `extraction_wall_ms` | 77 | 9.4 ms | 6.60 s | 90.04 s |
| `worker_total_ms` | 77 | 8.0 ms | 6.60 s | 90.03 s |
| `worker_snapshot_ms` | 77 | 0.24 ms | 1.1 ms | 240.3 ms |
| `worker_parse_ms` | 77 | 3.4 ms | 78.5 ms | 157.8 ms |
| `worker_analysis_ms` | 77 | 0.70 ms | 50.8 ms | 106.7 ms |
| `worker_render_ms` | 77 | 0.00 ms | 422.4 ms | 5.32 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 114.6 ms | 134.4 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 6.42 s | 89.76 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 565.3 ms | 6.98 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 5.56 s | 82.39 s |
| `worker_vision_ms` | 77 | 0.00 ms | 11.7 ms | 19.5 ms |
| `analyze_wall_ms` | 77 | 19.48 s | 29.80 s | 33.76 s |
| `distill_ms` | 77 | 5.2 ms | 126.8 ms | 629.3 ms |
| `prompt_ms` | 77 | 0.05 ms | 0.12 ms | 0.25 ms |
| `inference_ms` | 77 | 19.45 s | 29.10 s | 33.63 s |
| `prefill_ms` | 77 | 14.04 s | 23.48 s | 26.04 s |
| `generation_ms` | 77 | 5.75 s | 6.91 s | 8.12 s |
| `validation_ms` | 77 | 25.1 ms | 110.1 ms | 414.3 ms |
| `naming_ms` | 77 | 1.1 ms | 13.5 ms | 22.5 ms |
| `index_ms` | 77 | 5.1 ms | 154.4 ms | 782.8 ms |
| `retrieval_ms` | 77 | 0.05 ms | 9.4 ms | 20.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 102.5 tok/s | 110.2 tok/s | 112.0 tok/s |
| `generation_tok_per_s` | 77 | 15.6 tok/s | 17.0 tok/s | 17.5 tok/s |
| `prompt_tokens` | 77 | 1421 | 2416 | 2436 |
| `cached_tokens` | 77 | 0 | 0 | 964 |
| `generated_tokens` | 77 | 89 | 105 | 112 |
| `estimated_prompt_tokens` | 77 | 1606 | 2648 | 2754 |
| `model_requests` | 77 | 1 | 1 | 1 |
| `redistillations` | 77 | 0 | 0 | 0 |
| `source_characters` | 77 | 2460 | 112672 | 211186 |
| `digest_characters` | 77 | 2496 | 5387 | 6343 |
| `prompt_characters` | 77 | 4975 | 7872 | 8828 |
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
| Parser worker | 12 MB | 758 MB | scan-agreement-10p |
| Model server | 1962 MB | 1963 MB | scan-clean-lease-2p |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (37)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice for Imogen Castellanos.pdf` · **filed without review**
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-04-01 Account Summary from Kingsfold Community Bank.pdf` · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Document from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_UNSUPPORTED
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Trade Credit Application for Rosalind Merriweather.pdf` · trap: party Rosalind Merriweather (credit manager who signs) · **filed without review**
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-06-18 Commercial General Liability Coverage Part Declarations for Wexcombe Bicycle Cooperative.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report for Investor Relations.pdf` · review: DATE_UNSUPPORTED
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `Vendor Registration Form.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes of the Regular Meeting of the Board of Directors for Beckett Halloran.pdf` · trap: party Beckett Halloran (board president) · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report for Delphine Moncrieff.pdf` · trap: party Delphine Moncrieff (conservator who examined the work) · review: PARTY_UNSUPPORTED
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-10 Aircraft Maintenance Record from Highmeadow Air Charter LLC.pdf` · trap: date 2026-08-10 (first maintenance entry) · review: PARTY_UNSUPPORTED
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `Agreement to be duly executed between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf` · review: DATE_UNSUPPORTED
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `Annual Report from Tamsin Valley Farmers Cooperative.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-08 Document - Fernvale Cider Works LLC.docx` · review: TYPE_UNSUPPORTED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 RESOLUTION for Board of Directors.docx` · **filed without review**
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review - Q3 2026 for Pemberly Falls Distribution Co.pptx` · **filed without review**
- **product-launch-plan**: expected `2026-06-03 Product Launch Plan from Quarrystone Audio Labs Inc.pptx`, got `2026-06-03 Fieldnote S2 Product Launch Plan from Quarrystone Audio Labs Inc.pptx` · review: PARTY_UNSUPPORTED
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 REPORT from Rowanbrae Vineyards - 2026 Harvest Log.xlsx` · review: PARTY_UNSUPPORTED
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-20 Document from Signe Holmqvist.eml` · trap: date 2026-03-20 (approval deadline in the quoted request) · review: TYPE_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-29 Notice of Hearing to THE CLERK OF THE.txt` · review: PARTY_UNSUPPORTED
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-31 Document.png` · trap: date 2026-05-31 (end of a sale promotion) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement - Copper Flats Dental Group PLLC.pdf` · **filed without review**
- **scan-faint-letter**: expected `2026-03-02 Letter of Resignation from Mireille Saltonstall.png`, got `2026-03-02 Letter of Resignation - Dr. Rufus Pemberton.png` · trap: party Dr. Rufus Pemberton (practice owner the letter is addressed to) · **filed without review**
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 QUOTATION from Kestrel Ridge Aero Services - Parts and Quotes.tiff` · **filed without review**
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `New Patient Intake Form for Calder Way Family Medicine.png` · review: DATE_UNSUPPORTED
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter of the Saltmarsh Point Homeowners Association for Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **inspection-log-ruled-2p**: expected `2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf`, got `2026-05-19 Fire Extinguisher Inspection Report for Leopold Haverkamp.pdf` · trap: party Leopold Haverkamp (inspector who signs) · **filed without review**
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Price List.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-29 Westbrook Kettle Brewing Co. Benefits Enrollment Change Form for Westbrook Kettle Brewing Co.pdf` · **filed without review**
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Property Loss Notice from Harrowmere Mutual Insurance Company.pdf` · **filed without review**
- **scan-rotated-page-in-pdf**: expected `2026-04-27 Notice of Freight Claim from Hollin Brook Furniture Gallery.pdf`, got `2026-04-20 Notice of Freight Claim from Hollin Brook Furniture Gallery.pdf` · trap: date 2026-04-20 (delivery date) · **filed without review**
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-05-28 Invoice from Tolliver grain & feed cooperative.png` · trap: date 2026-05-28 (date of SPC-30417) · **filed without review**
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-06-23 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf` · **filed without review**
- **scan-bill-of-lading**: expected `2026-09-08 Bill of Lading from Corriveau Millwork Supply Co.pdf`, got `Straight Bill of Lading from Corriveau Millwork Supply Co.pdf` · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · review: DATE_UNSUPPORTED
- **watershed-monitoring-report-100p**: expected `2026-06-19 Watershed Monitoring Report for Cobalt Basin Water Authority.pdf`, got `Annual Watershed Monitoring Report for Cobalt Basin Water Authority.pdf` · review: DATE_UNSUPPORTED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
