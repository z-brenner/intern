# InternBench: live run

- **Run:** 2026-11-21T09:37:47Z · commit `ae77a5a7d2`
- **Machine:** INTEL(R) XEON(R) PLATINUM 8573C, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `97291f77f59a`
- **Corpus:** 55 documents · 55 completed · gold `dc976e7dd9`
- **Wall time:** 687.55 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 5/51 (9.8%) |
| Document type | 38/55 (69.1%) |
| Date | 36/51 (70.6%) |
| Date role (when the reviewed date was chosen) | 8/35 (22.9%) |
| Parties | 22/55 (40.0%) |
| Relation word (when parties were named) | 26/48 (54.2%) |
| Party in the right role (when parties were named) | 12/34 (35.3%) |
| Ready / review routing | 6/33 (18.2%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 28/55 (50.9%) |
| Description specificity | 52.1% (mean of 55) |
| Evidence recall (model's quotes) | 26.5% (mean of 54) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 81.5% (mean of 54) |
| Sent to review | 100.0% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 9.1% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 89.1% | 55 |
| Review rate | 100.0% | 55 |
| Evidence recall | 26.5% | 54 |
| Total latency p50 | 10.05 s | 55 |
| Total latency p95 | 19.64 s | 55 |
| Generation latency p50 | 4.17 s | 55 |
| Generation latency p95 | 6.28 s | 55 |
| Generated tokens p50 | 102 | 55 |
| Generated tokens p95 | 142 | 55 |
| Prompt tokens p50 | 1206 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 51 |
| Trap date chosen | 6 of 54 |
| Forbidden party named | 9 of 55 |
| Spurious parties named | 30 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 121 (0.0%) |
| Right name sent to review anyway | 5 of 27 |
| Sent to review for an unsupported fact | 49 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 19.64 s | 19.64 s |
| form | 3 | 0/2 | 1/2 | 0/3 | 1/1 | 100.0% | 13.55 s | 14.08 s |
| invoice | 5 | 0/5 | 1/5 | 0/5 | 0/1 | 100.0% | 7.13 s | 8.77 s |
| letter | 22 | 4/22 | 17/22 | 14/22 | 1/13 | 100.0% | 10.89 s | 36.27 s |
| memo | 6 | 0/6 | 5/6 | 0/6 | 0/4 | 100.0% | 10.64 s | 19.53 s |
| minutes | 3 | 0/3 | 3/3 | 1/3 | 0/2 | 100.0% | 13.30 s | 13.83 s |
| notice | 11 | 1/9 | 7/9 | 6/11 | 2/10 | 100.0% | 7.55 s | 12.46 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 5.67 s | 5.67 s |
| report | 1 | – | – | 0/1 | 1/1 | 100.0% | 7.44 s | 7.44 s |
| statement | 2 | 0/2 | 2/2 | 0/2 | 1/1 | 100.0% | 16.31 s | 17.83 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 0/6 | – | 100.0% | 6.56 s | 10.89 s |
| native | 38 | 4/36 | 31/36 | 18/38 | 2/25 | 100.0% | 9.66 s | 36.27 s |
| office | 4 | 0/2 | 2/2 | 0/4 | 3/3 | 100.0% | 7.44 s | 17.83 s |
| scan | 7 | 1/7 | 3/7 | 4/7 | 1/5 | 100.0% | 15.46 s | 19.53 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 1/16 | 7/16 | 6/19 | 4/10 | 100.0% | 6.50 s | 10.89 s |
| 2-4 | 15 | 2/15 | 10/15 | 6/15 | 1/7 | 100.0% | 14.08 s | 19.64 s |
| 5-9 | 10 | 1/9 | 8/9 | 4/10 | 1/8 | 100.0% | 10.17 s | 13.83 s |
| 10-24 | 9 | 1/9 | 9/9 | 6/9 | 0/7 | 100.0% | 12.38 s | 36.27 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 19.53 s | 19.53 s |
| 50-99 | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 88.49 s | 88.49 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 1/11 | 11/11 | 6/11 | 0/8 | 0 of 11 | 13.26 s | 88.49 s | 4.57 s | 5.91 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 1/6 | 5/6 | 3/6 | 1 |
| advisory_committee | 2 | 0/2 | 2/2 | 1/2 | 0 |
| agency_guidance | 4 | 0/4 | 3/4 | 0/4 | 0 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 3 | 0/3 | 3/3 | 2/3 | 1 |
| as_of_date | 2 | 0/2 | 2/2 | 0/2 | 0 |
| attachments | 5 | 0/5 | 5/5 | 3/5 | 1 |
| attachments_listed | 1 | 0/1 | 0/1 | 1/1 | 1 |
| attendance_list | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 0/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| body_recounts_events | 1 | 0/1 | 0/1 | 1/1 | 0 |
| born_digital | 38 | 4/36 | 31/36 | 18/38 | 11 |
| care_of_counsel | 2 | 0/2 | 0/2 | 2/2 | 1 |
| charts | 1 | 0/1 | 1/1 | 0/1 | 0 |
| circular_letter | 2 | 0/2 | 2/2 | 0/2 | 0 |
| competing_dates | 34 | 3/34 | 25/34 | 17/34 | 9 |
| court_notice | 5 | 0/3 | 2/3 | 2/5 | 4 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 0/2 | 1 |
| customer_first | 5 | 0/5 | 1/5 | 0/5 | 1 |
| date_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 1/4 | 3/4 | 3/4 | 1 |
| deadline_in_body | 5 | 1/3 | 3/3 | 3/5 | 3 |
| determination | 1 | 0/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| docx | 3 | 0/1 | 1/1 | 0/3 | 2 |
| due_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| effective_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printed_to_pdf | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_thread | 1 | 0/1 | 0/1 | 1/1 | 1 |
| enforcement_release | 3 | 0/3 | 2/3 | 2/3 | 1 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 1/1 | 1/1 | 1/1 | 0 |
| eu_body | 2 | 0/2 | 2/2 | 1/2 | 1 |
| financial_report | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_release | 2 | 0/2 | 1/2 | 1/2 | 2 |
| foi_response | 5 | 0/5 | 4/5 | 2/5 | 1 |
| foreign_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 3 | 0/2 | 1/2 | 0/3 | 1 |
| former_name | 1 | 0/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 1/4 | 3/4 | 3/4 | 1 |
| government_form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| handwritten | 6 | 0/6 | 0/6 | 0/6 | 0 |
| handwritten_dates | 2 | 0/2 | 1/2 | 0/2 | 1 |
| historic | 6 | 0/6 | 0/6 | 0/6 | 0 |
| historical | 2 | 0/2 | 1/2 | 0/2 | 1 |
| image | 6 | 0/6 | 0/6 | 0/6 | 0 |
| invoice | 5 | 0/5 | 1/5 | 0/5 | 1 |
| invoices_attached | 2 | 0/2 | 2/2 | 1/2 | 0 |
| issuer_not_named | 2 | 0/2 | 1/2 | 0/2 | 0 |
| letter | 22 | 4/22 | 17/22 | 14/22 | 4 |
| letter_from_company | 3 | 1/3 | 2/3 | 1/3 | 0 |
| letterhead | 3 | 1/3 | 2/3 | 1/3 | 1 |
| liquidation | 3 | 1/3 | 2/3 | 2/3 | 1 |
| long_document | 5 | 0/5 | 5/5 | 2/5 | 1 |
| many_dates | 8 | 1/6 | 5/6 | 3/8 | 4 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo | 6 | 0/6 | 5/6 | 0/6 | 0 |
| memorandum | 4 | 0/4 | 3/4 | 0/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 2/3 | 0/3 | 2 |
| minutes | 3 | 0/3 | 3/3 | 1/3 | 1 |
| modern | 44 | 5/40 | 33/40 | 21/44 | 13 |
| month_only_date | 1 | – | – | 0/1 | 1 |
| multiple_letters | 2 | 0/2 | 2/2 | 1/2 | 0 |
| named_officials | 4 | 0/2 | 2/2 | 1/4 | 4 |
| named_person_at_org | 12 | 3/12 | 10/12 | 8/12 | 3 |
| names_removed | 1 | 0/1 | 0/1 | 1/1 | 1 |
| new_zealand | 10 | 2/10 | 7/10 | 7/10 | 3 |
| newsletter | 1 | – | – | 0/1 | 1 |
| no_action_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| no_document_date | 3 | – | – | 1/3 | 2 |
| no_inside_address | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 0/4 | 1 |
| northern_ireland | 2 | – | – | 0/2 | 1 |
| notice | 11 | 1/9 | 7/9 | 6/11 | 6 |
| ocr_text_layer | 7 | 1/7 | 3/7 | 4/7 | 2 |
| office | 4 | 0/2 | 2/2 | 0/4 | 2 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 1/6 | 5/6 | 4/8 | 5 |
| publication_timestamp | 4 | 1/4 | 3/4 | 3/4 | 1 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 2/14 | 11/14 | 5/14 | 1 |
| regulator | 9 | 2/9 | 6/9 | 8/9 | 2 |
| reissued | 1 | 0/1 | 0/1 | 1/1 | 1 |
| relative_date_reference | 1 | 0/1 | 0/1 | 0/1 | 0 |
| remittance_slip | 1 | 0/1 | 1/1 | 0/1 | 1 |
| reply_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 0/1 | 1 |
| request_letter | 2 | 1/2 | 2/2 | 1/2 | 0 |
| safety_recommendation | 2 | 0/2 | 2/2 | 1/2 | 0 |
| scanned | 7 | 1/7 | 3/7 | 4/7 | 2 |
| sender_at_end | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period | 2 | 0/2 | 2/2 | 1/2 | 0 |
| settlement | 2 | 0/2 | 2/2 | 1/2 | 0 |
| short | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| staff_names | 2 | 0/2 | 2/2 | 1/2 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 0/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 1/2 | 0/2 | 0 |
| tax_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| trading_name | 2 | 1/2 | 1/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_not_printed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| uk | 8 | 0/6 | 4/6 | 2/8 | 4 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 26 | 1/24 | 20/24 | 10/26 | 7 |
| vat | 1 | 0/1 | 1/1 | 0/1 | 1 |
| wales | 6 | 0/6 | 4/6 | 2/6 | 3 |
| warning_letter | 3 | 1/3 | 1/3 | 3/3 | 1 |
| xlsx | 1 | 0/1 | 1/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 178 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 18 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 10.05 s | 19.64 s | 88.49 s |
| `extraction_wall_ms` | 55 | 21.8 ms | 7.42 s | 71.55 s |
| `worker_total_ms` | 55 | 20.7 ms | 7.41 s | 71.53 s |
| `worker_snapshot_ms` | 55 | 0.26 ms | 1.4 ms | 3.2 ms |
| `worker_parse_ms` | 55 | 11.8 ms | 70.2 ms | 208.9 ms |
| `worker_analysis_ms` | 55 | 1.2 ms | 10.3 ms | 42.3 ms |
| `worker_render_ms` | 55 | 0.00 ms | 886.6 ms | 29.26 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 79.1 ms | 112.5 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 13.48 s | 140.70 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 891.0 ms | 9.02 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 12.28 s | 131.35 s |
| `worker_vision_ms` | 55 | 0.00 ms | 31.0 ms | 54.2 ms |
| `analyze_wall_ms` | 55 | 9.49 s | 16.94 s | 19.62 s |
| `distill_ms` | 55 | 3.9 ms | 36.3 ms | 127.1 ms |
| `prompt_ms` | 55 | 0.04 ms | 0.08 ms | 0.12 ms |
| `inference_ms` | 55 | 9.48 s | 16.74 s | 19.60 s |
| `prefill_ms` | 55 | 5.19 s | 12.21 s | 13.30 s |
| `generation_ms` | 55 | 4.17 s | 6.28 s | 6.98 s |
| `validation_ms` | 55 | 11.6 ms | 32.4 ms | 73.4 ms |
| `naming_ms` | 55 | 0.48 ms | 4.0 ms | 6.2 ms |
| `index_ms` | 55 | 3.6 ms | 33.2 ms | 118.9 ms |
| `retrieval_ms` | 55 | 0.07 ms | 2.9 ms | 7.3 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 228.8 tok/s | 237.1 tok/s | 238.3 tok/s |
| `generation_tok_per_s` | 55 | 24.4 tok/s | 25.9 tok/s | 26.4 tok/s |
| `prompt_tokens` | 55 | 1206 | 2678 | 2878 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 102 | 142 | 181 |
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
| fast | 18 | 5.0 ms | 27.3 ms | 4.5 ms | 26.1 ms | 22.1 ms | 2.9 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 16.7 ms | 73.3 ms | 15.8 ms | 71.4 ms | 58.7 ms | 9.4 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 2.47 s | 2.47 s | 2.47 s | 2.47 s | 6.2 ms | 5.8 ms | 88.9 ms | 2.34 s |
| ocr | 15 | 3.64 s | 71.55 s | 3.64 s | 71.53 s | 208.9 ms | 42.3 ms | 29.26 s | 140.70 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 6.1 ms | 3.09 s | 5.3 ms | 3.09 s | 14.7 ms | 1.5 ms | 130.4 ms | 2.93 s |
| 2-4 | 15 | 15.8 ms | 5.19 s | 15.4 ms | 5.19 s | 70.2 ms | 5.8 ms | 718.9 ms | 8.25 s |
| 5-9 | 10 | 15.7 ms | 27.6 ms | 15.0 ms | 26.6 ms | 22.2 ms | 4.8 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 53.1 ms | 24.35 s | 50.9 ms | 24.34 s | 63.2 ms | 10.3 ms | 3.37 s | 45.86 s |
| 25-49 | 1 | 7.42 s | 7.42 s | 7.41 s | 7.41 s | 78.6 ms | 22.4 ms | 886.6 ms | 13.48 s |
| 50-99 | 1 | 71.55 s | 71.55 s | 71.53 s | 71.53 s | 208.9 ms | 42.3 ms | 29.26 s | 140.70 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 12.8 ms | 12.8 ms | 12.1 ms | 12.1 ms | 8.9 ms | 1.4 ms | 0.00 ms | 0.00 ms |
| form | 3 | 3.64 s | 4.59 s | 3.64 s | 4.59 s | 15.0 ms | 1.3 ms | 348.4 ms | 5.87 s |
| invoice | 5 | 1.42 s | 1.74 s | 1.41 s | 1.74 s | 14.7 ms | 0.72 ms | 0.00 ms | 1.56 s |
| letter | 22 | 27.6 ms | 24.35 s | 26.6 ms | 24.34 s | 70.2 ms | 9.0 ms | 3.37 s | 45.86 s |
| memo | 6 | 15.3 ms | 7.42 s | 14.6 ms | 7.41 s | 78.6 ms | 22.4 ms | 886.6 ms | 13.48 s |
| minutes | 3 | 47.1 ms | 53.1 ms | 44.6 ms | 50.9 ms | 39.0 ms | 10.3 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 5.0 ms | 26.4 ms | 4.5 ms | 25.2 ms | 21.5 ms | 2.3 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.68 s | 1.68 s | 1.68 s | 1.68 s | 11.0 ms | 0.06 ms | 0.00 ms | 1.64 s |
| report | 1 | 5.4 ms | 5.4 ms | 4.7 ms | 4.7 ms | 4.4 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.9 ms | 2.47 s | 2.1 ms | 2.47 s | 6.2 ms | 5.8 ms | 88.9 ms | 2.34 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 445 MB | 1336 MB | real5-letter-19 |
| Model server | 853 MB | 854 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (46)

- **real5-letter-01**: expected `2017-05-08 Warning Letter to Inghams Enterprises (NZ) Pty Limited.pdf`, got `2017-02-01 Letter to Inghams Enterprises (NZ) PTY Limited.pdf` · trap: date 2017-02-01 (date the original letter was issued; this is the reissued letter) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `letter from Coast to Coast Limited.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter to Mr. Vinn White Deputy Administrator Federal Motor Carrier Safety Administration.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-05**: expected `2025-09-23 Safety Recommendation Letter to Federal Aviation Administration.pdf`, got `2025-09-23 Letter to Textron Aviation.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-08**: expected `2026-07-24 No-Action Letter to Kraken Derivatives Exchange Inc.pdf`, got `2026-07-24 Letter from Kraken Derivatives Exchange Inc.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-09**: expected `2025-06-23 Approval Letter to Maryland Department of Health.pdf`, got `2025-06-23 Letter to Medicaid Director Maryland Department of Health.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-10**: expected `2026-07-21 Approval Letter to Delaware Health and Social Services.pdf`, got `2026-07-21 Letter to Andrew Wilson.pdf` · trap: party Andrew Wilson (addressee) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-11**: expected `2026-06-02 Approval Letter to Oklahoma Health Care Authority.pdf`, got `2026-06-02 Letter - Medicaid Director, Oklahoma Health Care Authority.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `Letter - Welsh Government.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Miller Research Payment (2) Information requested.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `2026-04-20 Letter to Anu Talus.pdf` · trap: party Anu Talus (EDPB Chair who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-16**: expected `2025-05-05 Response Letter to Moritz Körner.pdf`, got `2025-05-05 Letter to Anu Talus Chair of the European Data Protection Board.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2021-07-23 Tax Invoice from Allen+Clarke.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Letter to Ministry for Regulation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Letter from Tatou NZ Ltd.pdf` · trap: party Tatou NZ Ltd (company the request is about) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document - ShnMerr made the same say as Meothes m made fou Yon last Yor. , Shutterr made.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2025-12-03 Email to Neath Port Talbot Council Cyngor Castell-nedd Port Talbot The Quays, Brunel Way.pdf` · trap: date 2025-12-03 (date of an earlier e-mail in the thread) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter to Division of Market Oversight.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Document - Kieran Jones.pdf` · trap: party Kieran Jones (joint liquidator, a person) · review: TYPE_UNSUPPORTED, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document from Environmental Protection Authority.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-04**: expected `2026-10-30 Advertisement of Application for Putting Company into Liquidation with Commissioner of Inland Revenue.pdf`, got `Advertisement of Application for Putting Company into Liquidation for Rogan Construction Limited.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice for Stewart D. Aaron.pdf` · trap: party Stewart D. Aaron (magistrate judge the notice is about) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice for Judge Rebecca Rutherford.pdf` · trap: party Rebecca Rutherford (magistrate judge named) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Document - Interactive Brokers LLC.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Document from U.S. Department of the Treasury.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-11**: expected `2025-12-04 Enforcement Release for Gracetown, Inc.pdf`, got `2018-04-06 Document - Gracetown, Inc.pdf` · trap: date 2018-04-06 (date Deripaska was designated) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document from NEW YORK.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document from Teme10.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document from Wateloo Hl.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document from D.CREHANGE.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from Soyg New York.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-06**: expected `2025-09-09 Tax Invoice from Ian Davies Plant Limited.pdf`, got `2025-09-09 Tax Invoice from Neath Port Talbot County Borough Council.pdf` · trap: party Neath Port Talbot County Borough Council (the customer) · review: LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report for Marketable Debt vs. Nonmarketable Debt as of September 30, 2025.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `Document.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `1982-03-04 Document - General Services Administration.pdf` · trap: party General Services Administration (addressee in the form's TO line) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `General Counsel Memorandum to All Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Memorandum for Heads of Executive Departments and Agencies.pdf` · review: PARTY_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- … and 6 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
