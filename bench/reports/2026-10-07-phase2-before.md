# InternBench: replay run

- **Run:** 2026-10-07T07:58:32Z · commit `9e1f5270ca-dirty`
- **Machine:** Intel(R) Xeon(R) Processor @ 2.10GHz, 4 logical cores, 15.7 GB RAM, linux (Ubuntu 24.04.4 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 72 documents · 71 completed · 1 model_failed · gold `6b4e051c7c`
- **Recording:** made 2026-10-07T04:54:20Z at commit `0660f2a599` · sha256 `1366aa8f43`

> Replay: every score is this code's, but timings and memory are the recording's, taken on the machine above - not measured by this run.

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 19/72 (26.4%) |
| Document type | 55/72 (76.4%) |
| Date | 57/72 (79.2%) |
| Date role (when the reviewed date was chosen) | 38/56 (67.9%) |
| Parties | 47/72 (65.3%) |
| Relation word (when parties were named) | 28/70 (40.0%) |
| Party in the right role (when parties were named) | 31/39 (79.5%) |
| Ready / review routing | 32/52 (61.5%) |
| Description has every fact | 5/72 (6.9%) |
| Description states nothing false | 66/71 (93.0%) |
| Description is specific | 71/72 (98.6%) |
| Description fact coverage | 47.7% (mean of 72) |
| Description specificity | 96.8% (mean of 72) |
| Evidence recall (model's quotes) | 50.0% (mean of 72) |
| Digest recall (facts reaching the digest) | 98.2% (mean of 72) |
| Prompt recall (facts reaching the prompt sent) | 98.2% (mean of 72) |
| Sent to review | 38.0% of 71 completed |

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 26 of 72 |
| Trap date chosen | 11 of 72 |
| Forbidden party named | 7 of 72 |
| Spurious parties named | 26 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 5 of 223 (2.2%) |
| Right name sent to review anyway | 1 of 48 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 1/3 | 2/3 | 3/3 | 1/1 | 33.3% | 39.32 s | 43.19 s |
| annual_report | 2 | 0/2 | 2/2 | 2/2 | 2/2 | 0.0% | 65.24 s | 88.30 s |
| bill_of_lading | 1 | 0/1 | 1/1 | 0/1 | – | 0.0% | 42.53 s | 42.53 s |
| certificate | 1 | 0/1 | 0/1 | 1/1 | – | 0.0% | 34.63 s | 34.63 s |
| claim | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 19.85 s | 19.85 s |
| condition_report | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 32.34 s | 32.34 s |
| contract | 13 | 11/13 | 12/13 | 12/13 | 7/9 | 8.3% | 46.98 s | 109.56 s |
| email | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 35.65 s | 35.65 s |
| form | 5 | 0/5 | 3/5 | 2/5 | 3/4 | 20.0% | 19.94 s | 36.13 s |
| inspection_report | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 0.0% | 47.84 s | 47.84 s |
| insurance | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 36.99 s | 36.99 s |
| invoice | 7 | 5/7 | 6/7 | 6/7 | 4/5 | 28.6% | 29.59 s | 31.98 s |
| letter | 5 | 0/5 | 4/5 | 4/5 | 2/4 | 60.0% | 36.44 s | 40.91 s |
| log | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 56.01 s | 56.01 s |
| maintenance_record | 1 | 0/1 | 0/1 | 0/1 | – | 0.0% | 46.26 s | 46.26 s |
| minutes | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 42.79 s | 42.79 s |
| newsletter | 1 | 0/1 | 1/1 | 1/1 | – | 0.0% | 22.98 s | 22.98 s |
| notice | 10 | 0/10 | 7/10 | 4/10 | 2/8 | 80.0% | 33.85 s | 42.42 s |
| payroll | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 76.54 s | 76.54 s |
| presentation | 2 | 1/2 | 1/2 | 1/2 | 2/2 | 0.0% | 35.65 s | 44.68 s |
| price_list | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 31.72 s | 31.72 s |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 0.0% | 23.59 s | 34.34 s |
| quotation | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 100.0% | 15.00 s | 15.00 s |
| rate_confirmation | 1 | 0/1 | 0/1 | 1/1 | 0/1 | 100.0% | 29.39 s | 29.39 s |
| receipt | 1 | 0/1 | 1/1 | 0/1 | 1/1 | 100.0% | 17.31 s | 17.31 s |
| remittance_advice | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 0.0% | 20.55 s | 20.55 s |
| report | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 43.36 s | 43.36 s |
| resolution | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 44.08 s | 44.08 s |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1/1 | 0.0% | 68.52 s | 68.52 s |
| statement | 3 | 0/3 | 2/3 | 1/3 | 2/3 | 66.7% | 43.92 s | 53.11 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 35.65 s | 35.65 s |
| mixed | 3 | 2/3 | 2/3 | 3/3 | – | 33.3% | 43.19 s | 46.98 s |
| native | 42 | 11/42 | 32/42 | 26/42 | 24/39 | 34.2% | 36.13 s | 78.70 s |
| ocr_corrupted | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 30.40 s | 30.40 s |
| office | 5 | 2/5 | 4/5 | 3/5 | 3/5 | 40.0% | 40.99 s | 44.68 s |
| scan | 16 | 4/16 | 14/16 | 11/16 | 3/4 | 43.8% | 28.73 s | 109.56 s |
| sheet | 3 | 0/3 | 3/3 | 2/3 | 1/2 | 33.3% | 56.01 s | 76.54 s |
| text | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 42.42 s | 42.42 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 37 | 7/37 | 30/37 | 19/37 | 13/26 | 48.6% | 31.72 s | 44.08 s |
| 2-4 | 24 | 7/24 | 19/24 | 19/24 | 12/17 | 33.3% | 39.10 s | 63.05 s |
| 5-9 | 4 | 1/4 | 2/4 | 3/4 | 4/4 | 0.0% | 62.29 s | 68.52 s |
| 10-24 | 3 | 2/3 | 2/3 | 2/3 | 1/2 | 0.0% | 44.68 s | 77.18 s |
| 25-49 | 2 | 2/2 | 2/2 | 2/2 | 1/1 | 0.0% | 78.70 s | 109.56 s |
| 50-99 | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 83.07 s | 83.07 s |
| 100+ | 1 | 0/1 | 1/1 | 1/1 | 1/1 | 0.0% | 88.30 s | 88.30 s |

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| amendment | 3 | 1/3 | 2/3 | 3/3 | 0 |
| competing_dates | 62 | 16/62 | 48/62 | 41/62 | 18 |
| complex_pdf | 5 | 0/5 | 4/5 | 2/5 | 4 |
| contract | 17 | 12/17 | 14/17 | 16/17 | 1 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_in_table | 5 | 2/5 | 5/5 | 3/5 | 1 |
| docx | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 1 | 0/1 | 1/1 | 0/1 | 0 |
| financial | 8 | 1/8 | 6/8 | 5/8 | 2 |
| form | 7 | 0/7 | 4/7 | 3/7 | 2 |
| healthcare | 3 | 0/3 | 2/3 | 2/3 | 1 |
| hr | 4 | 1/4 | 2/4 | 3/4 | 1 |
| image_only_scan | 16 | 4/16 | 14/16 | 11/16 | 3 |
| image_region | 1 | 0/1 | 0/1 | 1/1 | 0 |
| information_dense | 5 | 2/5 | 4/5 | 4/5 | 0 |
| invoice | 7 | 5/7 | 6/7 | 6/7 | 1 |
| irrelevant_names | 12 | 2/12 | 9/12 | 9/12 | 2 |
| key_value | 11 | 3/11 | 8/11 | 5/11 | 3 |
| layout_parties | 11 | 3/11 | 10/11 | 5/11 | 3 |
| letter | 5 | 0/5 | 4/5 | 4/5 | 1 |
| low_resolution_scan | 3 | 0/3 | 3/3 | 1/3 | 0 |
| middle_fact | 3 | 0/3 | 2/3 | 3/3 | 1 |
| mixed_scan | 3 | 2/3 | 2/3 | 3/3 | 0 |
| multi_column | 7 | 2/7 | 5/7 | 3/7 | 6 |
| noisy_scan | 3 | 0/3 | 3/3 | 3/3 | 0 |
| notice | 10 | 0/10 | 7/10 | 4/10 | 6 |
| ocr_corrupted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ocr_critical_fields | 4 | 0/4 | 3/4 | 2/4 | 1 |
| pages_10 | 2 | 1/2 | 1/2 | 1/2 | 0 |
| pages_100 | 1 | 0/1 | 1/1 | 1/1 | 0 |
| pages_25 | 2 | 2/2 | 2/2 | 2/2 | 0 |
| pages_5 | 1 | 0/1 | 0/1 | 1/1 | 1 |
| pages_50 | 1 | 0/1 | 1/1 | 1/1 | 0 |
| png | 5 | 1/5 | 4/5 | 4/5 | 1 |
| pptx | 2 | 1/2 | 1/2 | 1/2 | 1 |
| presentation | 2 | 1/2 | 1/2 | 1/2 | 1 |
| purchase_order | 2 | 1/2 | 2/2 | 2/2 | 0 |
| referenced_agreement | 20 | 7/20 | 16/20 | 15/20 | 2 |
| rotated_page | 1 | 0/1 | 0/1 | 1/1 | 1 |
| rotated_scan | 4 | 1/4 | 4/4 | 2/4 | 1 |
| simple_digital | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sow | 1 | 0/1 | 0/1 | 1/1 | 1 |
| spreadsheet | 3 | 0/3 | 3/3 | 2/3 | 0 |
| statement | 3 | 0/3 | 2/3 | 1/3 | 1 |
| stream_order | 3 | 0/3 | 1/3 | 0/3 | 5 |
| table | 36 | 8/36 | 28/36 | 23/36 | 10 |
| tiff | 2 | 0/2 | 2/2 | 1/2 | 0 |
| unusual | 4 | 0/4 | 3/4 | 2/4 | 2 |
| xlsx | 2 | 0/2 | 2/2 | 2/2 | 0 |

## OCR

| Document | Layer | Pages | CER | CER (any case) | WER | Dates | Names | IDs | Confidence | OCR time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| scan-clean-lease-2p | scan | 2 | 0.8% | 0.8% | 1.3% | 100.0% | 100.0% | 100.0% | 87 | 2.12 s |
| scan-mixed-amendment | mixed | 1 | 2.3% | 2.3% | 5.7% | 100.0% | 100.0% | – | 94 | 643.5 ms |
| scan-rotated-90-invoice | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 96 | 725.7 ms |
| scan-upside-down-po | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 96 | 2.25 s |
| scan-skewed-notice | scan | 1 | 64.6% | 64.6% | 67.9% | 33.3% | 100.0% | 100.0% | 95 | 742.7 ms |
| scan-low-res-receipt | scan | 1 | 15.8% | 15.8% | 23.0% | 100.0% | 100.0% | 100.0% | 84 | 303.3 ms |
| scan-noisy-statement | scan | 1 | 38.1% | 38.1% | 97.2% | 100.0% | 100.0% | 100.0% | 68 | 5.57 s |
| scan-faint-letter | scan | 1 | 13.4% | 13.4% | 15.7% | 100.0% | 100.0% | – | 95 | 758.7 ms |
| scan-agreement-10p | scan | 10 | 0.1% | 0.1% | 0.3% | 100.0% | 100.0% | 50.0% | 95 | 11.76 s |
| scan-lease-25p | scan | 25 | 0.1% | 0.1% | 0.1% | 100.0% | 100.0% | 100.0% | 96 | 28.01 s |
| scan-fax-two-frames | scan | 1 (+1 unread) | 56.9% | 56.9% | 53.8% | 50.0% | 100.0% | 0.0% | 94 | 499.8 ms |
| scan-patient-intake-form | scan | 1 | 0.8% | 0.8% | 2.4% | 100.0% | 100.0% | 100.0% | 94 | 1.27 s |
| scan-rotated-page-in-pdf | scan | 2 | 23.3% | 23.0% | 28.6% | 100.0% | 100.0% | 100.0% | 96 | 1.71 s |
| scan-cancellation-notice-150dpi | scan | 1 | 0.0% | 0.0% | 0.0% | 100.0% | 100.0% | 100.0% | 96 | 1.07 s |
| scan-remittance-advice-120dpi | scan | 1 | 45.2% | 45.1% | 47.6% | 100.0% | 100.0% | 100.0% | 86 | 697.7 ms |
| scan-mixed-middle-page | mixed | 1 | 50.4% | 50.2% | 54.0% | – | – | 100.0% | 94 | 777.3 ms |
| mixed-signature-region | mixed | 1 | 32.3% | 32.3% | 30.3% | 0.0% | 0.0% | – | – | 0.00 ms |
| scan-certificate-of-insurance | scan | 1 | 11.0% | 11.0% | 13.2% | 85.7% | 100.0% | 100.0% | 95 | 1.47 s |
| scan-bill-of-lading | scan | 1 | 8.0% | 7.8% | 11.2% | 100.0% | 100.0% | 100.0% | 93 | 1.13 s |
| **All scanned pages** |  | 54 (+1 unread) | 5.5% | 5.5% | 6.7% | 89.5% | 92.0% | 93.8% | 94 | – |

Error rates are edit distances over the drawn text's length, pooled over pages; Dates, Names and IDs are the fraction of those drawn on the read pages that survive OCR. A page the reader does not return (a TIFF frame it does not read, shown as unread) counts as read empty, as does every page of a scan whose extraction failed: every character and value on it missed.

## Structure

| Document | Route class | Reading order (snippets) | Table rows | Table cells | Key-values | Routes (pages) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice-date-in-table | unrouted | – | 6/7 (85.7%) | 38/39 (97.4%) | 3/7 (42.9%) | – |
| invoice-layout-only | unrouted | – | 5/5 (100.0%) | 25/25 (100.0%) | 7/7 (100.0%) | – |
| purchase-order | unrouted | – | 8/8 (100.0%) | 48/48 (100.0%) | 3/3 (100.0%) | – |
| lease-two-column | unrouted | 8/8 (100.0%) | 5/10 (50.0%) | 15/20 (75.0%) | – | – |
| declarations-interleaved | unrouted | 5/8 (62.5%) | 6/6 (100.0%) | 12/12 (100.0%) | 5/6 (83.3%) | – |
| vendor-registration-form | unrouted | – | – | – | 6/6 (100.0%) | – |
| change-order-form | unrouted | – | 6/6 (100.0%) | 18/18 (100.0%) | 6/6 (100.0%) | – |
| scan-patient-intake-form | unrouted | – | – | – | 0/6 (0.0%) | – |
| newsletter-three-column | unrouted | 9/9 (100.0%) | – | – | – | – |
| agreement-two-column-footnotes | unrouted | 10/10 (100.0%) | – | – | – | – |
| meeting-notice-columns | unrouted | 11/11 (100.0%) | – | – | – | – |
| meeting-notice-interleaved | unrouted | 3/11 (27.3%) | – | – | – | – |
| meeting-notice-reversed | unrouted | 1/11 (9.1%) | – | – | – | – |
| rate-confirmation-rotated | unrouted | – | 0/4 (0.0%) | 17/28 (60.7%) | 4/5 (80.0%) | – |
| inspection-log-ruled-2p | unrouted | – | 45/45 (100.0%) | 315/315 (100.0%) | 5/5 (100.0%) | – |
| price-list-unruled | unrouted | – | 21/21 (100.0%) | 126/126 (100.0%) | – | – |
| invoice-label-above | unrouted | – | – | – | 0/8 (0.0%) | – |
| invoice-right-aligned | unrouted | – | 7/7 (100.0%) | 28/28 (100.0%) | 8/8 (100.0%) | – |
| invoice-boxed-grid | unrouted | – | 5/5 (100.0%) | 20/20 (100.0%) | 1/11 (9.1%) | – |
| benefits-change-checkbox-form | unrouted | – | 12/17 (70.6%) | 17/22 (77.3%) | 0/7 (0.0%) | – |
| loss-notice-boxed-fields | unrouted | – | – | – | 0/17 (0.0%) | – |
| scan-rotated-page-in-pdf | unrouted | – | 0/6 (0.0%) | 31/36 (86.1%) | 5/5 (100.0%) | – |
| scan-cancellation-notice-150dpi | unrouted | – | – | – | 7/7 (100.0%) | – |
| scan-remittance-advice-120dpi | unrouted | – | 0/7 (0.0%) | 34/35 (97.1%) | 0/7 (0.0%) | – |
| scan-mixed-middle-page | unrouted | – | 0/6 (0.0%) | 22/24 (91.7%) | – | – |
| mixed-signature-region | unrouted | 1/5 (20.0%) | – | – | – | – |
| scan-certificate-of-insurance | unrouted | – | 4/5 (80.0%) | 29/30 (96.7%) | 0/5 (0.0%) | – |
| scan-bill-of-lading | unrouted | – | 1/5 (20.0%) | 23/31 (74.2%) | 8/10 (80.0%) | – |
| **All** |  | 48/73 (65.8%) | 131/170 (77.1%) | 818/857 (95.4%) | 68/136 (50.0%) | – |

Measured over the page text the engine receives. Reading order: the most gold snippets found in the gold's order (a longest increasing run), of all of them. Table rows: rows whose cells are all on one line of their table, in order, with no other row between them and an empty check box left empty. Table cells: cells found in their table's lines. Key-values: values after their label on its line (before the next label), alone on the next line, or in the cell under it in a linearised table. Routes: pages whose layout took the expected route, judged only when the worker sends layouts. See `docs/internbench.md` for the exact rules.

## Routes

- **Documents by route class:** unrouted 72
- **Pages the gold gives a route for:** 36, 0 judged

None was judged: a route is judged only on a page read with a layout, and those documents came without one (a worker before the router, or an extraction that failed).

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 71 | 35.65 s | 78.70 s | 109.56 s |
| `extraction_wall_ms` | 71 | 4.3 ms | 2.48 s | 31.07 s |
| `worker_total_ms` | 71 | 3.9 ms | 2.48 s | 31.06 s |
| `worker_snapshot_ms` | 71 | 0.20 ms | 0.50 ms | 1.5 ms |
| `worker_parse_ms` | 71 | 2.1 ms | 10.8 ms | 61.2 ms |
| `worker_analysis_ms` | 71 | 0.02 ms | 0.18 ms | 1.6 ms |
| `worker_render_ms` | 71 | 0.00 ms | 225.2 ms | 3.00 s |
| `worker_image_decode_ms` | 71 | 0.00 ms | 41.9 ms | 123.7 ms |
| `worker_ocr_ms` | 71 | 0.00 ms | 2.25 s | 28.01 s |
| `worker_ocr_encode_ms` | 71 | 0.00 ms | 93.5 ms | 1.18 s |
| `worker_ocr_engine_ms` | 71 | 0.00 ms | 2.15 s | 26.81 s |
| `worker_vision_ms` | 71 | 0.00 ms | 225.2 ms | 264.6 ms |
| `analyze_wall_ms` | 71 | 35.64 s | 78.49 s | 88.23 s |
| `distill_ms` | 71 | 0.91 ms | 14.8 ms | 44.6 ms |
| `prompt_ms` | 71 | 0.00 ms | 0.00 ms | 0.00 ms |
| `inference_ms` | 71 | 35.62 s | 78.44 s | 88.10 s |
| `prefill_ms` | 71 | 24.59 s | 64.77 s | 72.67 s |
| `generation_ms` | 71 | 11.03 s | 17.00 s | 20.04 s |
| `validation_ms` | 71 | 11.5 ms | 41.8 ms | 78.2 ms |
| `naming_ms` | 71 | 0.87 ms | 3.3 ms | 9.3 ms |

Over the 71 documents that completed. The other 1 (failed, or not scored) are left out: a failed document's time is how long it took to fail.

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 71 | 83.3 tok/s | 87.0 tok/s | 88.9 tok/s |
| `generation_tok_per_s` | 71 | 13.0 tok/s | 13.9 tok/s | 14.0 tok/s |
| `prompt_tokens` | 71 | 2030 | 5040 | 5783 |
| `cached_tokens` | 71 | 46 | 1216 | 1229 |
| `generated_tokens` | 71 | 142 | 209 | 256 |
| `estimated_prompt_tokens` | 71 | 2386 | 5791 | 6386 |
| `model_requests` | 71 | 1 | 1 | 1 |
| `redistillations` | 71 | 0 | 0 | 0 |
| `source_characters` | 71 | 2316 | 35721 | 127637 |
| `digest_characters` | 71 | 2823 | 13288 | 14380 |
| `prompt_characters` | 71 | 7832 | 18442 | 19534 |
| `worker_ocr_pages` | 71 | 0 | 2 | 25 |
| `worker_ocr_passes` | 71 | 0 | 2 | 25 |
| `worker_orientation_passes` | 71 | 0 | 0 | 1 |
| `worker_rendered_pixels` | 71 | 0 | 16830000 | 210375000 |

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Model server | 4732 MB | 5578 MB | scan-bill-of-lading |
| Benchmark (engine) | 12 MB | 20 MB | scan-bill-of-lading |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (53)

- **notice-rent-increase**: expected `2026-05-12 Notice of Rent Increase for Imogen Castellanos.pdf`, got `2026-05-12 Notice of Rent Increase for Cresthaven Court Holdings LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **invoice-date-in-table**: expected `2026-03-04 Invoice from Halvorsen Fixture Works LLC.pdf`, got `2026-03-04 Invoice between Quillon Ridge Bakery, Inc and Halvorsen Fixture Works LLC.pdf` · trap: party Quillon Ridge Bakery, Inc. (bill-to customer) · **filed without review**
- **account-statement**: expected `2026-04-01 Account Statement from Kingsfold Community Bank.pdf`, got `2026-03-01 Document - Kingsfold Community Bank.pdf` · trap: date 2026-03-01 (start of the statement period) · review: TYPE_UNSUPPORTED
- **second-amendment**: expected `2025-09-29 Second Amendment to Software License and Support Agreement with Umberlee Imaging Software Inc.pdf`, got `2025-09-29 Second Amendment to Software License and Support with Umberlee Imaging Software Inc.pdf` · **filed without review**
- **sow-under-msa-5p**: expected `2026-05-26 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf`, got `2024-10-07 Statement of Work between Thornbury Data Labs LLC and Saltmarsh Regional Water Authority.pdf` · trap: date 2024-10-07 (date of the master agreement the SOW is issued under) · **filed without review**
- **notice-of-default**: expected `2025-10-14 Notice of Default for Glasswing Ceramics LLC.pdf`, got `2025-07-01 Notice of Default and Reservation of Rights with Basalt Commercial Credit Corp.pdf` · trap: date 2025-07-01 (missed installment due date) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **offer-letter**: expected `2026-02-10 Offer Letter to Dalia Brennagh.pdf`, got `2026-03-16 Offer of Employment - Corvid Data Systems Inc.pdf` · trap: date 2026-03-16 (proposed start date) · **filed without review**
- **prior-authorization**: expected `2026-03-09 Notice of Prior Authorization Approval for Florian Okonkwo.pdf`, got `2026-03-09 Notice of Prior Authorization Approval for Silverlode Health Partners.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **explanation-of-benefits**: expected `2026-01-15 Explanation of Benefits for Paloma Achterberg.pdf`, got `2026-01-15 Statement of Benefits - Paloma Achterberg.pdf` · **filed without review**
- **capital-call-notice**: expected `2026-02-06 Capital Call Notice from Ravensmoor Growth Partners III, L.P.pdf`, got `2026-02-06 Capital Call Notice for Ravensmoor Growth Partners III, L.P.pdf` · **filed without review**
- **engagement-letter**: expected `2026-01-09 Engagement Letter between Halloran Ostrowski CPAs LLP and Silverbirch Ceramics Inc.pdf`, got `2026-01-09 Letter with Halloran Ostrowski CPAs LLP.pdf` · **filed without review**
- **letterhead-letter**: expected `2026-03-19 Credit Approval Letter from Moonrake Paper & Packaging Co.pdf`, got `2026-03-19 Document between Moonrake Paper & Packaging Co and Briarport Coffee Roasters LLC.pdf` · review: TYPE_UNSUPPORTED
- **declarations-interleaved**: expected `2026-07-01 Commercial Package Policy Declarations for Wexcombe Bicycle Cooperative.pdf`, got `2026-06-18 Commercial Package Policy Declarations - Kingsfold Insurance Agency, Inc.pdf` · trap: party Kingsfold Insurance Agency, Inc. (insurance agency (producer)) · review: PARTY_UNSUPPORTED
- **annual-report-excerpt-8p**: expected `2026-03-12 Annual Report from Rookwood Precision Metals Corporation.pdf`, got `2025-12-31 2025 Annual Report - Rookwood Precision Metals Corporation.pdf` · **filed without review**
- **vendor-registration-form**: expected `2026-04-22 Vendor Registration Form for Emberglow Coatings Ltd.pdf`, got `2026-04-22 Vendor Registration Form.pdf` · **filed without review**
- **change-order-form**: expected `2026-05-07 Change Order between Briarport Library District and Stonebridge Builders Inc.pdf`, got `2026-05-07 Change Order - Briarport Library District.pdf` · **filed without review**
- **board-minutes**: expected `2025-09-17 Board Meeting Minutes for Cairnfield Cooperative Grocers.pdf`, got `2025-09-17 Minutes - Cairnfield Cooperative Grocers.pdf` · **filed without review**
- **field-condition-report**: expected `2025-10-02 Loan Condition Report from Saltash Point Museum of Art.pdf`, got `2025-10-02 Loan Condition Report - Haverford Family Collection.pdf` · **filed without review**
- **aircraft-maintenance-log**: expected `2026-08-14 Aircraft Maintenance Record for Highmeadow Air Charter LLC.pdf`, got `2026-08-10 Maintenance Record between Halden Aero Works and HA-180.pdf` · trap: date 2026-08-10 (first maintenance entry); party Halden Aero Works (aircraft manufacturer) · **filed without review**
- **data-processing-agreement-10p**: model_failed (MODEL_REPLY_TRUNCATED): expected `2026-02-02 Data Processing Addendum between Thistledown Learning Inc and Skylark Analytics Ltd.pdf`
- **credit-agreement-50p**: expected `2026-06-12 Credit Agreement between Marrowfield Packaging Holdings, Inc and Halden Bay National Bank, N.A.pdf`, got `2026-06-12 Credit Agreement - Marrowfield Packaging Holdings, Inc.pdf` · review: DESCRIPTION_UNSUPPORTED
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
- **scan-skewed-notice**: expected `2026-08-03 Notice of Nonrenewal of Lease for Wendell Okafor.tiff`, got `2026-08-03 Notice of Nonrenewal of Lease - Harrowgate Apartments LLC.tiff` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **scan-low-res-receipt**: expected `2026-05-09 Receipt from Quarry Bend Hardware & Feed.png`, got `2026-05-09 Document - Quarry Bend Hardware & Feed.png` · review: TYPE_UNSUPPORTED
- **scan-noisy-statement**: expected `2026-08-31 Statement of Account from Brindle Paper & Janitorial Supply.pdf`, got `2026-08-31 Statement of Account - Copper Flats Dental Group PLLC.pdf` · review: PARSER_WARNING
- **scan-faint-letter**: expected `2026-03-02 Letter of Resignation from Mireille Saltonstall.png`, got `2026-03-02 Document from Mireille Saltonstall.png` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **ocr-corrupted-invoice**: expected `2026-03-10 Invoice from Wexcombe Millwork Co.pdf`, got `Invoice from Wexcombe Millwork Co.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **scan-fax-two-frames**: expected `2026-08-12 Quotation from Kestrel Ridge Aero Services.tiff`, got `2026-08-12 Document to Highmeadow Air Charter LLC.tiff` · review: TYPE_UNSUPPORTED, PARSER_WARNING
- **scan-patient-intake-form**: expected `2026-08-27 New Patient Intake Form for Ione Kowalczyk.png`, got `1990-11-23 New Patient Intake Form - Ione Kowalczyk.png` · trap: date 1990-11-23 (patient date of birth) · **filed without review**
- **newsletter-three-column**: expected `2026-09-14 Newsletter from Saltmarsh Point Homeowners Association.pdf`, got `2026-09-14 Newsletter - Saltmarsh Point Homeowners Association.pdf` · **filed without review**
- **meeting-notice-columns**: expected `2026-08-03 Notice of Special Meeting of Members from Larchmont Valley Federal Credit Union.pdf`, got `2026-09-15 Notice of Special Meeting of Members with Larchmont Valley Federal Credit Union.pdf` · trap: date 2026-09-15 (date of the meeting); party Ironbridge Community Credit Union (merger partner) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- … and 13 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
