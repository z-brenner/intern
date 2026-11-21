# InternBench: live run

- **Run:** 2026-11-21T04:51:36Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 55 documents · 55 completed · gold `dc976e7dd9`
- **Wall time:** 885.30 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 8/51 (15.7%) |
| Document type | 42/55 (76.4%) |
| Date | 42/51 (82.3%) |
| Date role (when the reviewed date was chosen) | 11/40 (27.5%) |
| Parties | 29/55 (52.7%) |
| Relation word (when parties were named) | 23/48 (47.9%) |
| Party in the right role (when parties were named) | 11/34 (32.4%) |
| Ready / review routing | 8/33 (24.2%) |
| Description states nothing false | 54/55 (98.2%) |
| Description is specific | 28/55 (50.9%) |
| Description specificity | 53.3% (mean of 55) |
| Evidence recall (model's quotes) | 53.4% (mean of 54) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 81.5% (mean of 54) |
| Sent to review | 96.4% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 27.3% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 76.4% | 55 |
| Review rate | 96.4% | 55 |
| Evidence recall | 53.4% | 54 |
| Total latency p50 | 14.00 s | 55 |
| Total latency p95 | 26.90 s | 55 |
| Generation latency p50 | 6.18 s | 55 |
| Generation latency p95 | 7.89 s | 55 |
| Generated tokens p50 | 91 | 55 |
| Generated tokens p95 | 116 | 55 |
| Prompt tokens p50 | 1206 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 51 |
| Trap date chosen | 3 of 54 |
| Forbidden party named | 9 of 55 |
| Spurious parties named | 23 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 128 (0.8%) |
| Right name sent to review anyway | 6 of 27 |
| Sent to review for an unsupported fact | 42 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 26.90 s | 26.90 s |
| form | 3 | 0/2 | 2/2 | 1/3 | 1/1 | 100.0% | 17.70 s | 18.85 s |
| invoice | 5 | 1/5 | 1/5 | 3/5 | 1/1 | 80.0% | 9.54 s | 10.70 s |
| letter | 22 | 6/22 | 18/22 | 14/22 | 1/13 | 100.0% | 16.94 s | 34.81 s |
| memo | 6 | 0/6 | 6/6 | 2/6 | 0/4 | 100.0% | 13.63 s | 19.25 s |
| minutes | 3 | 0/3 | 3/3 | 1/3 | 0/2 | 100.0% | 17.93 s | 20.81 s |
| notice | 11 | 1/9 | 9/9 | 7/11 | 3/10 | 90.9% | 10.48 s | 19.06 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 5.84 s | 5.84 s |
| report | 1 | – | – | 0/1 | 1/1 | 100.0% | 10.60 s | 10.60 s |
| statement | 2 | 0/2 | 2/2 | 1/2 | 1/1 | 100.0% | 20.39 s | 22.18 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 3/6 | – | 100.0% | 8.36 s | 9.95 s |
| native | 38 | 7/36 | 34/36 | 21/38 | 4/25 | 94.7% | 14.00 s | 34.81 s |
| office | 4 | 0/2 | 2/2 | 1/4 | 3/3 | 100.0% | 10.60 s | 22.18 s |
| scan | 7 | 1/7 | 6/7 | 4/7 | 1/5 | 100.0% | 19.25 s | 24.20 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 0/16 | 8/16 | 8/19 | 4/10 | 100.0% | 8.92 s | 14.88 s |
| 2-4 | 15 | 2/15 | 14/15 | 7/15 | 2/7 | 93.3% | 18.85 s | 26.90 s |
| 5-9 | 10 | 3/9 | 9/9 | 6/10 | 2/8 | 90.0% | 14.74 s | 20.81 s |
| 10-24 | 9 | 3/9 | 9/9 | 7/9 | 0/7 | 100.0% | 18.66 s | 34.81 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 19.25 s | 19.25 s |
| 50-99 | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 80.72 s | 80.72 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 3/11 | 11/11 | 8/11 | 0/8 | 0 of 11 | 18.90 s | 80.72 s | 6.24 s | 7.17 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 0/6 | 5/6 | 3/6 | 1 |
| advisory_committee | 2 | 0/2 | 2/2 | 1/2 | 1 |
| agency_guidance | 4 | 0/4 | 4/4 | 2/4 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 0/1 | 1 |
| approval_letter | 3 | 3/3 | 3/3 | 3/3 | 0 |
| as_of_date | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments | 5 | 3/5 | 5/5 | 5/5 | 0 |
| attachments_listed | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attendance_list | 2 | 0/2 | 2/2 | 1/2 | 1 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 2/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 1/1 | 0 |
| born_digital | 38 | 7/36 | 34/36 | 21/38 | 10 |
| care_of_counsel | 2 | 1/2 | 2/2 | 2/2 | 0 |
| charts | 1 | 0/1 | 1/1 | 0/1 | 0 |
| circular_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| competing_dates | 34 | 7/34 | 30/34 | 21/34 | 5 |
| court_notice | 5 | 0/3 | 3/3 | 4/5 | 3 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 2/2 | 1/2 | 0 |
| customer_first | 5 | 1/5 | 1/5 | 3/5 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 0/1 | 1 |
| dated_line_at_end | 4 | 0/4 | 4/4 | 2/4 | 2 |
| deadline_in_body | 5 | 0/3 | 3/3 | 2/5 | 4 |
| determination | 1 | 0/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 0/3 | 1 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| email | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email_printed_to_pdf | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email_thread | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enforcement_release | 3 | 1/3 | 3/3 | 2/3 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| eu_agency | 1 | 0/1 | 1/1 | 0/1 | 1 |
| eu_body | 2 | 0/2 | 1/2 | 1/2 | 1 |
| financial_report | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_release | 2 | 1/2 | 2/2 | 1/2 | 1 |
| foi_response | 5 | 0/5 | 4/5 | 2/5 | 1 |
| foreign_company | 1 | 0/1 | 1/1 | 0/1 | 0 |
| form | 3 | 0/2 | 2/2 | 1/3 | 0 |
| former_name | 1 | 0/1 | 1/1 | 0/1 | 0 |
| gazette | 4 | 0/4 | 4/4 | 2/4 | 2 |
| government_form | 2 | 0/2 | 2/2 | 1/2 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 3/6 | 0 |
| handwritten_dates | 2 | 0/2 | 2/2 | 1/2 | 0 |
| historic | 6 | 0/6 | 0/6 | 3/6 | 0 |
| historical | 2 | 0/2 | 2/2 | 1/2 | 0 |
| image | 6 | 0/6 | 0/6 | 3/6 | 0 |
| invoice | 5 | 1/5 | 1/5 | 3/5 | 0 |
| invoices_attached | 2 | 0/2 | 2/2 | 2/2 | 0 |
| issuer_not_named | 2 | 0/2 | 2/2 | 0/2 | 0 |
| letter | 22 | 6/22 | 18/22 | 14/22 | 4 |
| letter_from_company | 3 | 0/3 | 2/3 | 2/3 | 0 |
| letterhead | 3 | 0/3 | 2/3 | 1/3 | 1 |
| liquidation | 3 | 0/3 | 3/3 | 1/3 | 2 |
| long_document | 5 | 3/5 | 5/5 | 4/5 | 0 |
| many_dates | 8 | 0/6 | 6/6 | 4/8 | 2 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo | 6 | 0/6 | 6/6 | 2/6 | 0 |
| memorandum | 4 | 0/4 | 4/4 | 0/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 3/3 | 1/3 | 1 |
| minutes | 3 | 0/3 | 3/3 | 1/3 | 2 |
| modern | 44 | 8/40 | 38/40 | 24/44 | 10 |
| month_only_date | 1 | – | – | 0/1 | 0 |
| multiple_letters | 2 | 1/2 | 2/2 | 2/2 | 0 |
| named_officials | 4 | 0/2 | 2/2 | 3/4 | 3 |
| named_person_at_org | 12 | 6/12 | 10/12 | 8/12 | 3 |
| names_removed | 1 | 0/1 | 1/1 | 0/1 | 1 |
| new_zealand | 10 | 1/10 | 10/10 | 7/10 | 2 |
| newsletter | 1 | – | – | 0/1 | 0 |
| no_action_letter | 2 | 1/2 | 2/2 | 1/2 | 0 |
| no_document_date | 3 | – | – | 2/3 | 2 |
| no_inside_address | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 1/4 | 1 |
| northern_ireland | 2 | – | – | 0/2 | 0 |
| notice | 11 | 1/9 | 9/9 | 7/11 | 5 |
| ocr_text_layer | 7 | 1/7 | 6/7 | 4/7 | 1 |
| office | 4 | 0/2 | 2/2 | 1/4 | 1 |
| paid_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 0/1 | 1/1 | 0/1 | 0 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 0/6 | 6/6 | 5/8 | 5 |
| publication_timestamp | 4 | 0/4 | 4/4 | 2/4 | 2 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 1/14 | 13/14 | 6/14 | 2 |
| regulator | 9 | 3/9 | 9/9 | 7/9 | 0 |
| reissued | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| remittance_slip | 1 | 1/1 | 1/1 | 1/1 | 0 |
| reply_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 0/1 | 0 |
| request_letter | 2 | 0/2 | 2/2 | 1/2 | 0 |
| safety_recommendation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| scanned | 7 | 1/7 | 6/7 | 4/7 | 1 |
| sender_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 1/2 | 2/2 | 2/2 | 0 |
| settlement | 2 | 1/2 | 2/2 | 2/2 | 0 |
| short | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 2 | 0/2 | 2/2 | 1/2 | 1 |
| stamped_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| statement | 2 | 0/2 | 2/2 | 1/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 2/2 | 0/2 | 0 |
| tax_invoice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| trading_name | 2 | 0/2 | 2/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 0/1 | 1 |
| type_not_printed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| uk | 8 | 1/6 | 5/6 | 2/8 | 3 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 26 | 6/24 | 23/24 | 15/26 | 5 |
| vat | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wales | 6 | 1/6 | 5/6 | 2/6 | 3 |
| warning_letter | 3 | 1/3 | 3/3 | 3/3 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 178 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 18 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 14.00 s | 26.90 s | 80.72 s |
| `extraction_wall_ms` | 55 | 16.6 ms | 5.70 s | 56.99 s |
| `worker_total_ms` | 55 | 15.7 ms | 5.70 s | 56.97 s |
| `worker_snapshot_ms` | 55 | 0.24 ms | 1.2 ms | 2.6 ms |
| `worker_parse_ms` | 55 | 8.9 ms | 61.1 ms | 947.9 ms |
| `worker_analysis_ms` | 55 | 0.93 ms | 8.2 ms | 31.9 ms |
| `worker_render_ms` | 55 | 0.00 ms | 641.7 ms | 23.08 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 57.6 ms | 84.6 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 10.38 s | 112.38 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 733.5 ms | 7.17 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 9.40 s | 104.96 s |
| `worker_vision_ms` | 55 | 0.00 ms | 18.7 ms | 27.2 ms |
| `analyze_wall_ms` | 55 | 13.61 s | 22.17 s | 26.89 s |
| `distill_ms` | 55 | 2.9 ms | 27.8 ms | 94.3 ms |
| `prompt_ms` | 55 | 0.03 ms | 0.06 ms | 0.09 ms |
| `inference_ms` | 55 | 13.60 s | 22.17 s | 26.87 s |
| `prefill_ms` | 55 | 7.72 s | 17.35 s | 18.83 s |
| `generation_ms` | 55 | 6.18 s | 7.89 s | 8.53 s |
| `validation_ms` | 55 | 6.7 ms | 23.9 ms | 54.6 ms |
| `naming_ms` | 55 | 0.36 ms | 3.1 ms | 4.8 ms |
| `index_ms` | 55 | 2.8 ms | 24.9 ms | 88.1 ms |
| `retrieval_ms` | 55 | 0.05 ms | 2.3 ms | 5.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 156.0 tok/s | 158.7 tok/s | 159.2 tok/s |
| `generation_tok_per_s` | 55 | 14.9 tok/s | 15.5 tok/s | 15.5 tok/s |
| `prompt_tokens` | 55 | 1206 | 2678 | 2878 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 91 | 116 | 128 |
| `estimated_prompt_tokens` | 55 | 1886 | 3281 | 3354 |
| `model_requests` | 55 | 1 | 1 | 1 |
| `redistillations` | 55 | 0 | 0 | 0 |
| `source_characters` | 55 | 5596 | 49993 | 104046 |
| `digest_characters` | 55 | 3967 | 8250 | 8964 |
| `prompt_characters` | 55 | 6022 | 10305 | 11013 |
| `worker_ocr_pages` | 55 | 0 | 5 | 77 |
| `worker_ocr_passes` | 55 | 0 | 5 | 77 |
| `worker_orientation_passes` | 55 | 0 | 5 | 77 |
| `worker_rendered_pixels` | 55 | 0 | 43516740 | 667903296 |
| `index_units` | 55 | 48 | 371 | 3200 |
| `context_units` | 55 | 25 | 65 | 114 |
| `context_tokens` | 55 | 1297 | 2690 | 2767 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 18 | 3.8 ms | 20.7 ms | 3.2 ms | 19.8 ms | 16.6 ms | 2.3 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 12.6 ms | 55.5 ms | 12.0 ms | 53.9 ms | 44.3 ms | 7.1 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 1.78 s | 1.78 s | 1.78 s | 1.78 s | 5.2 ms | 4.0 ms | 61.1 ms | 1.69 s |
| ocr | 15 | 2.87 s | 56.99 s | 2.87 s | 56.97 s | 947.9 ms | 31.9 ms | 23.08 s | 112.38 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4.5 ms | 2.36 s | 4.1 ms | 2.36 s | 11.5 ms | 1.1 ms | 101.8 ms | 2.23 s |
| 2-4 | 15 | 12.2 ms | 4.40 s | 11.7 ms | 4.40 s | 947.9 ms | 4.0 ms | 575.3 ms | 5.90 s |
| 5-9 | 10 | 11.7 ms | 21.0 ms | 11.2 ms | 20.4 ms | 17.0 ms | 3.7 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 40.8 ms | 19.78 s | 39.3 ms | 19.77 s | 48.4 ms | 8.2 ms | 2.42 s | 37.37 s |
| 25-49 | 1 | 5.70 s | 5.70 s | 5.70 s | 5.70 s | 61.1 ms | 20.4 ms | 641.7 ms | 10.38 s |
| 50-99 | 1 | 56.99 s | 56.99 s | 56.97 s | 56.97 s | 154.3 ms | 31.9 ms | 23.08 s | 112.38 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 9.9 ms | 9.9 ms | 9.5 ms | 9.5 ms | 6.8 ms | 1.1 ms | 0.00 ms | 0.00 ms |
| form | 3 | 2.87 s | 3.50 s | 2.87 s | 3.50 s | 12.3 ms | 0.95 ms | 269.0 ms | 4.40 s |
| invoice | 5 | 1.11 s | 1.36 s | 1.11 s | 1.36 s | 11.5 ms | 0.53 ms | 0.00 ms | 1.24 s |
| letter | 22 | 21.0 ms | 19.78 s | 20.4 ms | 19.77 s | 154.3 ms | 6.8 ms | 2.42 s | 37.37 s |
| memo | 6 | 11.6 ms | 5.70 s | 11.2 ms | 5.70 s | 61.1 ms | 20.4 ms | 641.7 ms | 10.38 s |
| minutes | 3 | 34.9 ms | 40.8 ms | 33.5 ms | 39.3 ms | 29.7 ms | 8.2 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 3.6 ms | 19.7 ms | 3.3 ms | 18.9 ms | 16.2 ms | 1.8 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.33 s | 1.33 s | 1.33 s | 1.33 s | 7.7 ms | 0.05 ms | 0.00 ms | 1.30 s |
| report | 1 | 3.8 ms | 3.8 ms | 3.3 ms | 3.3 ms | 3.1 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.0 ms | 1.78 s | 1.5 ms | 1.78 s | 5.2 ms | 4.0 ms | 61.1 ms | 1.69 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 455 MB | 1360 MB | real5-letter-18 |
| Model server | 1578 MB | 1579 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (43)

- **real5-letter-02**: expected `2018-08-31 Warning Letter to Slingshot Communications Limited.pdf`, got `2018-08-31 Warning Letter to New Zealand.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `2015-12-16 letter from Coast to Coast Limited.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter to National Transportation Safety Board.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-07**: expected `2026-08-10 Request for No-Action Relief from Eurex Clearing AG.pdf`, got `2026-08-10 Letter to Division of Clearing and Risk.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-08**: expected `2026-07-24 No-Action Letter to Kraken Derivatives Exchange Inc.pdf`, got `2026-07-24 Letter from United States.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `2024-02-08 Letter from ATISN 20183 - FOI Request.pdf` · trap: date 2024-02-08 (date the request was received) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Grŵp Addysg, Diwylliant a'r Gymraeg Education, Culture and Welsh Language Group.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-14**: expected `2025-03-03 Letter to European Commission.pdf`, got `2025-03-03 Letter to Verena Ross.pdf` · trap: party Verena Ross (ESMA Chair who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `Letter to Anu Talus.pdf` · trap: party Anu Talus (EDPB Chair who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real5-letter-16**: expected `2025-05-05 Response Letter to Moritz Körner.pdf`, got `2025-05-05 Response to your Inquiry Regarding the Data Protection Assessment of In-Car.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Rebecca MacPherson.pdf` · trap: party Rebecca MacPherson (Assistant Chief Counsel who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2021-07-23 Tax Invoice from He Pou a Rangi.pdf` · review: LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Letter to Ministry for Regulation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Letter.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document from Albert E. Schreiber.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2026-01-19 Email from Kaymac Marine & Civil Engineering Ltd.pdf` · trap: party Kaymac Marine & Civil Engineering Ltd (issuer of an attached invoice) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter to Electron Exchange DCM, LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-01**: expected `2025-10-01 Notice of Intention to Remove Company From the Register for RS2022 Limited.pdf`, got `2025-10-01 Notice of Intention to Remove Company for Hamish Pryde.pdf` · trap: party Hamish Pryde (liquidator, a person) · review: LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Notice of Intention to Remove Companies for Kieran Jones.pdf` · trap: party Kieran Jones (joint liquidator, a person) · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document from Environmental Protection Authority.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-04**: expected `2026-10-30 Advertisement of Application for Putting Company into Liquidation with Commissioner of Inland Revenue.pdf`, got `2026-10-30 Document - Rogan Construction Limited.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice for Stewart D. Aaron.pdf` · trap: party Stewart D. Aaron (magistrate judge the notice is about) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice from United States District Court Northern District of Texas.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Enforcement Release from ShapeShift AG.pdf` · **filed without review**
- **real5-notice-11**: expected `2025-12-04 Enforcement Release for Gracetown, Inc.pdf`, got `2025-12-04 Document from U.S. Department of the Treasury.pdf` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document from Sternberg Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document - BLUN & CO.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document from D.CREHANGE.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from Soyg New York.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report from Public Debt.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document from National Archives and Records Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `1982-03-04 Document from U.S. Department of Treasury.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `2025-05-16 General Counsel Memorandum to All Regional Directors.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Memorandum for Heads of Executive Departments and Agencies.pdf` · review: PARTY_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies - President Trump.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-05**: expected `2025-06-03 Mortgagee Letter from U.S. Department of Housing and Urban Development.pdf`, got `2025-06-03 Mortgagee Letter to All Fha-Approved Mortgagees.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-06**: expected `2025-06-27 Mortgagee Letter from U.S. Department of Housing and Urban Development.pdf`, got `2025-06-27 Mortgagee Letter to All Fha-Approved Mortgagees.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-09**: expected `2024-10-29 Meeting Minutes for Visiting Committee on Advanced Technology.pdf`, got `2024-10-29 Minutes for Jason Matusow.pdf` · trap: party Jason Matusow (VCAT Chair) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- … and 3 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
