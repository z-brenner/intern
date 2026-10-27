# InternBench: replay run

- **Run:** 2026-10-27T01:07:50Z · commit `unknown`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.80GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.52 GB · sha256 `d5f01a4ecaf4`
- **Corpus:** 77 documents · 77 completed · gold `0d03c75723`
- **Recording:** made 2026-10-26T23:50:48Z at commit `unknown` · sha256 `63becc7c65`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run, except `index_ms` and `retrieval_ms`, which this run measures from the recorded text.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 37/77 (48.0%) |
| Document type | 66/77 (85.7%) |
| Date | 67/77 (87.0%) |
| Date role (when the reviewed date was chosen) | 46/66 (69.7%) |
| Parties | 55/77 (71.4%) |
| Relation word (when parties were named) | 46/58 (79.3%) |
| Party in the right role (when parties were named) | 44/51 (86.3%) |
| Ready / review routing | 39/57 (68.4%) |
| Description has every fact | 13/77 (16.9%) |
| Description states nothing false | 77/77 (100.0%) |
| Description is specific | 69/77 (89.6%) |
| Description fact coverage | 45.0% (mean of 77) |
| Description specificity | 87.9% (mean of 77) |
| Evidence recall (model's quotes) | 55.6% (mean of 77) |
| Digest recall (facts reaching the digest) | 99.4% (mean of 77) |
| Prompt recall (facts reaching the prompt sent) | 100.0% (mean of 77) |
| Sent to review | 29.9% of 77 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 58.3% | 12 |
| Complex-document filename accuracy | 48.2% | 56 |
| Description completeness | 45.0% | 77 |
| Unsupported-fact rate (documents) | 18.2% | 77 |
| Review rate | 29.9% | 77 |
| Evidence recall | 55.6% | 77 |
| Total latency p50 | 14.44 s | 77 |
| Total latency p95 | 27.52 s | 77 |
| Generation latency p50 | 4.37 s | 77 |
| Generation latency p95 | 6.00 s | 77 |
| Generated tokens p50 | 88 | 77 |
| Generated tokens p95 | 113 | 77 |
| Prompt tokens p50 | 870 | 77 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed; this run replays a recording, so they are the recording's.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 23 of 77 |
| Trap date chosen | 5 of 77 |
| Forbidden party named | 1 of 77 |
| Spurious parties named | 6 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 260 (0.0%) |
| Right name sent to review anyway | 2 of 53 |
| Sent to review for an unsupported fact | 14 of 77 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 1/1 | 33.3% | 17.46 s | 17.87 s |
| annual_report | 2 | 0/2 | 0/2 | 1/2 | 0/2 | 100.0% | 10.90 s | 19.16 s |
| bill_of_lading | 1 | 1/1 | 1/1 | 1/1 | – | 0.0% | 18.85 s | 18.85 s |
| certificate | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 20.32 s | 20.32 s |
| claim | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 19.13 s | 19.13 s |
| condition_report | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 12.99 s | 12.99 s |
| contract | 16 | 15/16 | 15/16 | 16/16 | 12/12 | 0.0% | 18.71 s | 99.92 s |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 11.37 s | 11.37 s |
| form | 5 | 1/5 | 4/5 | 2/5 | 4/4 | 0.0% | 13.00 s | 14.96 s |
| inspection_report | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 9.08 s | 9.08 s |
| insurance | 2 | 0/2 | 2/2 | 1/2 | 2/2 | 0.0% | 16.61 s | 27.52 s |
| invoice | 7 | 5/7 | 6/7 | 6/7 | 3/5 | 42.9% | 11.29 s | 12.09 s |
| letter | 5 | 2/5 | 4/5 | 5/5 | 1/4 | 80.0% | 13.20 s | 20.94 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 21.74 s | 21.74 s |
| maintenance_record | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 15.20 s | 15.20 s |
| minutes | 1 | 1/1 | 1/1 | 1/1 | 1/1 | 0.0% | 15.37 s | 15.37 s |
| newsletter | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 13.19 s | 13.19 s |
| notice | 10 | 4/10 | 10/10 | 5/10 | 7/8 | 20.0% | 12.88 s | 35.21 s |
| payroll | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 10.53 s | 10.53 s |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 11.18 s | 18.64 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 14.07 s | 14.07 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 10.75 s | 14.44 s |
| quotation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 0.0% | 15.87 s | 15.87 s |
| rate_confirmation | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 13.51 s | 13.51 s |
| receipt | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 100.0% | 8.34 s | 8.34 s |
| remittance_advice | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 15.91 s | 15.91 s |
| report | 2 | 0/2 | 1/2 | 0/2 | 1/2 | 50.0% | 18.72 s | 22.00 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 17.02 s | 17.02 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 15.87 s | 15.87 s |
| statement | 3 | 1/3 | 2/3 | 2/3 | 1/3 | 33.3% | 13.59 s | 24.34 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 11.37 s | 11.37 s |
| mixed | 3 | 2/3 | 2/3 | 3/3 | – | 33.3% | 16.49 s | 17.87 s |
| native | 47 | 21/47 | 40/47 | 31/47 | 33/44 | 23.4% | 14.44 s | 24.34 s |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 11.78 s | 11.78 s |
| office | 5 | 2/5 | 4/5 | 4/5 | 3/5 | 40.0% | 17.02 s | 18.64 s |
| scan | 16 | 11/16 | 15/16 | 14/16 | 2/4 | 31.2% | 14.12 s | 99.92 s |
| sheet | 3 | 0/3 | 3/3 | 1/3 | 1/2 | 33.3% | 18.72 s | 21.74 s |
| text | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 12.88 s | 12.88 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 13/37 | 34/37 | 22/37 | 17/26 | 35.1% | 12.09 s | 20.32 s |
| 2-4 | 24 | 15/24 | 22/24 | 19/24 | 11/17 | 29.2% | 16.35 s | 21.74 s |
| 5-9 | 4 | 2/4 | 2/4 | 4/4 | 3/4 | 25.0% | 11.18 s | 15.87 s |
| 10-24 | 4 | 3/4 | 4/4 | 4/4 | 3/3 | 0.0% | 18.64 s | 42.18 s |
| 25-49 | 4 | 3/4 | 3/4 | 4/4 | 3/3 | 0.0% | 21.81 s | 99.92 s |
| 50-99 | 2 | 1/2 | 2/2 | 2/2 | 2/2 | 0.0% | 22.90 s | 27.52 s |
| 100+ | 2 | 0/2 | 0/2 | 0/2 | 0/2 | 100.0% | 19.16 s | 22.00 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 12 | 7/12 | 9/12 | 10/12 | 8/10 | 3 of 12 | 21.81 s | 99.92 s | 4.30 s | 5.63 s |
| complex | 56 | 27/56 | 49/56 | 40/56 | 33/46 | 17 of 56 | 14.96 s | 27.52 s | 4.46 s | 6.00 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 2/3 | 2/3 | 3/3 | 0 |
| competing_dates | 67 | 31/67 | 57/67 | 47/67 | 6 |
| complex_pdf | 5 | 0/5 | 4/5 | 1/5 | 0 |
| contract | 20 | 17/20 | 17/20 | 20/20 | 2 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_table | 7 | 4/7 | 6/7 | 6/7 | 1 |
| docx | 3 | 1/3 | 2/3 | 2/3 | 1 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| financial | 10 | 3/10 | 7/10 | 7/10 | 1 |
| form | 7 | 2/7 | 6/7 | 4/7 | 1 |
| healthcare | 3 | 1/3 | 2/3 | 1/3 | 1 |
| hr | 4 | 2/4 | 4/4 | 3/4 | 0 |
| image_only_scan | 16 | 11/16 | 15/16 | 14/16 | 1 |
| image_region | 1 | 0/1 | 0/1 | 1/1 | 0 |
| information_dense | 10 | 6/10 | 7/10 | 8/10 | 1 |
| invoice | 7 | 5/7 | 6/7 | 6/7 | 0 |
| irrelevant_names | 17 | 7/17 | 14/17 | 11/17 | 1 |
| key_value | 11 | 6/11 | 11/11 | 8/11 | 0 |
| layout_parties | 11 | 5/11 | 10/11 | 7/11 | 0 |
| letter | 5 | 2/5 | 4/5 | 5/5 | 1 |
| low_resolution_scan | 3 | 1/3 | 3/3 | 2/3 | 0 |
| middle_fact | 6 | 2/6 | 3/6 | 4/6 | 1 |
| mixed_scan | 3 | 2/3 | 2/3 | 3/3 | 0 |
| multi_column | 7 | 2/7 | 7/7 | 2/7 | 0 |
| noisy_scan | 3 | 2/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 4/10 | 10/10 | 5/10 | 0 |
| ocr_corrupted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 2/4 | 4/4 | 4/4 | 0 |
| pages_10 | 3 | 3/3 | 3/3 | 3/3 | 0 |
| pages_100 | 2 | 0/2 | 0/2 | 0/2 | 0 |
| pages_25 | 4 | 3/4 | 3/4 | 4/4 | 1 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 2 | 1/2 | 2/2 | 2/2 | 0 |
| png | 5 | 2/5 | 4/5 | 3/5 | 1 |
| pptx | 2 | 1/2 | 2/2 | 2/2 | 0 |
| presentation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 12/20 | 18/20 | 18/20 | 1 |
| rotated_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rotated_scan | 4 | 3/4 | 4/4 | 4/4 | 0 |
| simple_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 0/3 | 3/3 | 1/3 | 0 |
| statement | 3 | 1/3 | 2/3 | 2/3 | 1 |
| stream_order | 3 | 0/3 | 3/3 | 0/3 | 0 |
| table | 41 | 20/41 | 34/41 | 30/41 | 4 |
| tiff | 2 | 2/2 | 2/2 | 2/2 | 0 |
| unusual | 4 | 0/4 | 4/4 | 1/4 | 1 |
| xlsx | 2 | 0/2 | 2/2 | 1/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | 100.0% | 96 | 5.91 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 2.9% | 100.0% | 100.0% | – | 97 | 1.86 s |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 1.88 s |
| scan-upside-down-po | scan | 1 | 0.1% | 0.1% | 0.7% | 100.0% | 100.0% | 100.0% | 99 | 2.70 s |
| scan-skewed-notice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 98 | 2.49 s |
| scan-low-res-receipt | scan | 1 | 12.5% | 12.5% | 3.0% | 100.0% | 100.0% | 100.0% | 98 | 1.47 s |
| scan-noisy-statement | scan | 1 | 1.0% | 1.0% | 9.2% | 100.0% | 100.0% | 100.0% | 95 | 2.17 s |
| scan-faint-letter | scan | 1 | 0.4% | 0.4% | 0.5% | 100.0% | 100.0% | – | 98 | 2.30 s |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 98 | 31.58 s |
| scan-lease-25p | scan | 25 | 0.0% | 0.0% | 0.1% | 100.0% | 100.0% | 100.0% | 99 | 84.96 s |
| scan-fax-two-frames | scan | 2 | 2.6% | 2.6% | 1.4% | 100.0% | 100.0% | 100.0% | 99 | 5.01 s |
| scan-patient-intake-form | scan | 1 | 0.4% | 0.4% | 1.8% | 100.0% | 100.0% | 100.0% | 96 | 4.28 s |
| scan-rotated-page-in-pdf | scan | 2 | 0.1% | 0.1% | 0.5% | 100.0% | 100.0% | 100.0% | 98 | 6.10 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 97 | 3.90 s |
| scan-remittance-advice-120dpi | scan | 1 | 10.1% | 10.0% | 12.7% | 100.0% | 100.0% | 100.0% | 98 | 4.21 s |
| scan-mixed-middle-page | mixed | 1 | 4.0% | 4.0% | 4.0% | – | – | 100.0% | 96 | 3.00 s |
| mixed-signature-region | mixed | 1 | 0.2% | 0.2% | 1.0% | 100.0% | 100.0% | – | – | 1.97 s |
| scan-certificate-of-insurance | scan | 1 | 28.8% | 28.0% | 33.3% | 100.0% | 100.0% | 100.0% | 98 | 5.59 s |
| scan-bill-of-lading | scan | 1 | 32.5% | 31.7% | 30.2% | 100.0% | 100.0% | 100.0% | 98 | 5.16 s |
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
| `total_ms` | 77 | 14.44 s | 27.52 s | 99.92 s |
| `extraction_wall_ms` | 77 | 8.8 ms | 6.11 s | 87.79 s |
| `worker_total_ms` | 77 | 8.2 ms | 6.11 s | 87.73 s |
| `worker_snapshot_ms` | 77 | 0.22 ms | 2.0 ms | 4.3 ms |
| `worker_parse_ms` | 77 | 3.0 ms | 84.6 ms | 1.23 s |
| `worker_analysis_ms` | 77 | 0.67 ms | 62.2 ms | 378.9 ms |
| `worker_render_ms` | 77 | 0.00 ms | 456.1 ms | 5.84 s |
| `worker_image_decode_ms` | 77 | 0.00 ms | 104.1 ms | 124.5 ms |
| `worker_ocr_ms` | 77 | 0.00 ms | 5.91 s | 84.96 s |
| `worker_ocr_encode_ms` | 77 | 0.00 ms | 495.5 ms | 6.68 s |
| `worker_ocr_engine_ms` | 77 | 0.00 ms | 4.89 s | 71.49 s |
| `worker_vision_ms` | 77 | 0.00 ms | 13.9 ms | 24.1 ms |
| `analyze_wall_ms` | 77 | 13.18 s | 23.16 s | 35.20 s |
| `distill_ms` | 77 | 4.1 ms | 126.6 ms | 595.1 ms |
| `prompt_ms` | 77 | 0.04 ms | 0.09 ms | 0.31 ms |
| `inference_ms` | 77 | 13.13 s | 23.01 s | 35.15 s |
| `prefill_ms` | 77 | 8.21 s | 18.89 s | 29.05 s |
| `generation_ms` | 77 | 4.37 s | 6.00 s | 6.27 s |
| `validation_ms` | 77 | 24.6 ms | 198.6 ms | 561.9 ms |
| `naming_ms` | 77 | 0.91 ms | 12.8 ms | 26.6 ms |
| `index_ms` | 77 | 3.8 ms | 113.8 ms | 527.1 ms |
| `retrieval_ms` | 77 | 0.04 ms | 5.6 ms | 13.3 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 77 | 106.0 tok/s | 113.7 tok/s | 115.4 tok/s |
| `generation_tok_per_s` | 77 | 19.8 tok/s | 22.0 tok/s | 22.3 tok/s |
| `prompt_tokens` | 77 | 870 | 1861 | 2080 |
| `cached_tokens` | 77 | 628 | 628 | 1255 |
| `generated_tokens` | 77 | 88 | 113 | 118 |
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
| Parser worker | 11 MB | 710 MB | scan-agreement-10p |
| Model server | 1638 MB | 2127 MB | scan-clean-lease-2p |
| Benchmark (engine) | 11 MB | 32 MB | watershed-monitoring-report-100p |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (40)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice for Imogen Castellanos.pdf` · **filed without review**
- **invoice-layout-only**: expected `2026-02-17 Invoice from Ironvale Marine Coatings Co.pdf`, got `INVOICE from Ironvale Marine Coatings Co.pdf` · review: DATE_UNSUPPORTED
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-03-01 Statement from Kingsfold Community Bank.pdf` · trap: date 2026-03-01 (start of the statement period) · review: TYPE_INFERRED
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **notice-of-default**: expected `2025-10-14 Notice of Default for Glasswing Ceramics LLC.pdf`, got `2025-10-14 Loan Agreement between Basalt Commercial Credit Corp and Glasswing Ceramics LLC.pdf` · **filed without review**
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Meadowlark Health Plan.pdf` · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Document from Halloran Ostrowski Cpas LLP.pdf` · review: TYPE_UNSUPPORTED
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document - Moonrake Paper & Packaging Co.pdf` · review: TYPE_UNSUPPORTED
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-07-01 Commercial Package Policy Declarations for Commercial General Liability Coverage Part Declarations.pdf` · **filed without review**
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `Annual Report from Rookwood Precision Metals Corporation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Vendor Registration Form.pdf` · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report for Anselm Greaves.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-14 Aircraft Maintenance Record for Arlo Sinclair-Ray.pdf` · trap: party Arlo Sinclair-Ray (inspector who signed the return to service) · **filed without review**
- **annual-report-100p**: expected `2026-09-18 Annual Report from Tamsin Valley Farmers Cooperative.pdf`, got `Annual Report.pdf` · review: DATE_MISSING
- **demand-letter**: expected `2026-07-08 Demand Letter to Fernvale Cider Works LLC.docx`, got `2026-07-22 Document - Fernvale Cider Works LLC.docx` · trap: date 2026-07-22 (payment deadline) · review: TYPE_UNSUPPORTED
- **board-resolution**: expected `2025-11-03 Action by Unanimous Written Consent of the Board of Directors for Summerhill Robotics, Inc.docx`, got `2025-11-03 Unanimous Written Consent of the Board of Directors for The undersigned.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **quarterly-business-review**: expected `2026-09-24 Quarterly Business Review for Pemberly Falls Distribution Co.pptx`, got `2026-09-24 Quarterly Business Review to Pemberly Falls Distribution Co.pptx` · **filed without review**
- **payroll-register**: expected `2026-07-15 Payroll Register for Wrenfield Bakehouse LLC.xlsx`, got `2026-07-15 Payroll Register.xlsx` · review: DESCRIPTION_INVALID
- **ap-aging-report**: expected `2026-08-31 Accounts Payable Aging Summary for Lowmarsh Farm Equipment Co.csv`, got `2026-08-31 Accounts Payable Aging Summary.csv` · **filed without review**
- **harvest-log**: expected `2026-10-02 Harvest Log for Rowanbrae Vineyards.xlsx`, got `2026-10-02 Summary.xlsx` · **filed without review**
- **email-approval-thread**: expected `2026-03-17 Approval Email from Signe Holmqvist.eml`, got `2026-03-17 Document - Signe Holmqvist.eml` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **court-hearing-notice**: expected `2026-07-29 Notice of Hearing between Gilchrist Harbor Marine Supply, LLC and Tavistock Boatworks, Inc.txt`, got `2026-07-30 Notice of Hearing - Halden County Superior Court Clerk.txt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-upside-down-po**: expected `2026-06-03 Purchase Order from Thornbury Outdoor Gear Co.pdf`, got `2026-06-03 Purchase Order from Ridgewell Textiles Ltd.pdf` · **filed without review**
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document.png` · review: TYPE_UNSUPPORTED
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `2026-01-01 New Patient Intake Form.png` · trap: date 2026-01-01 (insurance coverage effective date) · **filed without review**
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 The Tidewater Ledger.pdf` · **filed without review**
- **meeting-notice-columns**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-08-03 Notice of Special Meeting of Members.pdf` · **filed without review**
- **meeting-notice-interleaved**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-08-03 Notice of Special Meeting of Members.pdf` · **filed without review**
- **meeting-notice-reversed**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-08-03 Notice of Special Meeting of Members.pdf` · **filed without review**
- **inspection-log-ruled-2p**: expected `2026-05-19 Fire Extinguisher Inspection Report for Corbel Street Lofts Condominium Association.pdf`, got `2026-05-19 Fire Extinguisher Inspection Report.pdf` · review: TYPE_INFERRED
- **price-list-unruled**: expected `2026-03-16 Price List from Thistledown Wholesale Nursery.pdf`, got `2026-03-16 Price List.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **invoice-boxed-grid**: expected `2026-07-09 Invoice from Whitlock & Sons Plumbing LLC.pdf`, got `2026-07-09 Service Invoice.pdf` · **filed without review**
- **benefits-change-checkbox-form**: expected `2026-06-29 Benefits Enrollment Change Form for Annika Solberg-Reyes.pdf`, got `2026-06-29 Benefits Enrollment Change Form for Westbrook Kettle Brewing Co.pdf` · **filed without review**
- **loss-notice-boxed-fields**: expected `2026-02-16 Property Loss Notice for Delacroix Bakehouse LLC.pdf`, got `2026-02-16 Property Loss Notice.pdf` · **filed without review**
- **scan-remittance-advice-120dpi**: expected `2026-07-17 Remittance Advice from Tolliver Grain & Feed Cooperative.png`, got `2026-07-17 Document - Tolliver Grain & Feed Cooperative.png` · review: TYPE_UNSUPPORTED
- **mixed-signature-region**: expected `2026-08-21 First Amendment to Software License Agreement with Corvane Analytics Inc.pdf`, got `First Amendment to Software License Agreement between Corvane Analytics Inc and Pellworth County Water Authority.pdf` · review: DATE_UNSUPPORTED
- **scan-certificate-of-insurance**: expected `2026-06-23 Certificate of Liability Insurance for Ostergaard Roofing Contractors LLC.pdf`, got `2026-06-23 Certificate of Liability Insurance for Hartsfield & Quail Insurance Brokers.pdf` · **filed without review**
- **industrial-lease-dated-in-schedule-25p**: expected `2026-04-14 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf`, got `2026-01-09 Industrial Lease between Ardent Quay Industrial Properties LP and Moss & Lanyard Distribution Inc.pdf` · trap: date 2026-01-09 (letter of intent the lease supersedes) · **filed without review**
- **property-policy-declarations-mid-60p**: expected `2026-07-01 Commercial Property Policy for Hollowmere Craft Brewing Cooperative.pdf`, got `2026-07-01 Commercial Property Policy to Hollowmere Craft Brewing Cooperative.pdf` · **filed without review**
- **watershed-monitoring-report-100p**: expected `2026-06-19 Watershed Monitoring Report for Cobalt Basin Water Authority.pdf`, got `Annual Watershed Monitoring Report.pdf` · review: DATE_UNSUPPORTED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
