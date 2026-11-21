# InternBench: live run

- **Run:** 2026-11-21T04:51:34Z · commit `a51447d321`
- **Machine:** AMD EPYC 9V45 96-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 55 documents · 55 completed · gold `dc976e7dd9`
- **Wall time:** 643.98 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 6/51 (11.8%) |
| Document type | 37/55 (67.3%) |
| Date | 41/51 (80.4%) |
| Date role (when the reviewed date was chosen) | 14/39 (35.9%) |
| Parties | 19/55 (34.5%) |
| Relation word (when parties were named) | 29/47 (61.7%) |
| Party in the right role (when parties were named) | 9/35 (25.7%) |
| Ready / review routing | 7/33 (21.2%) |
| Description states nothing false | 54/55 (98.2%) |
| Description is specific | 32/55 (58.2%) |
| Description specificity | 55.8% (mean of 55) |
| Evidence recall (model's quotes) | 40.4% (mean of 54) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 81.5% (mean of 54) |
| Sent to review | 98.2% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 9.1% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 87.3% | 55 |
| Review rate | 98.2% | 55 |
| Evidence recall | 40.4% | 54 |
| Total latency p50 | 10.33 s | 55 |
| Total latency p95 | 22.83 s | 55 |
| Generation latency p50 | 3.60 s | 55 |
| Generation latency p95 | 4.58 s | 55 |
| Generated tokens p50 | 94 | 55 |
| Generated tokens p95 | 118 | 55 |
| Prompt tokens p50 | 1098 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 51 |
| Trap date chosen | 5 of 54 |
| Forbidden party named | 12 of 55 |
| Spurious parties named | 33 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 134 (0.8%) |
| Right name sent to review anyway | 5 of 27 |
| Sent to review for an unsupported fact | 48 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 22.83 s | 22.83 s |
| form | 3 | 0/2 | 2/2 | 0/3 | 1/1 | 100.0% | 12.21 s | 12.68 s |
| invoice | 5 | 0/5 | 1/5 | 1/5 | 0/1 | 100.0% | 6.17 s | 7.93 s |
| letter | 22 | 5/22 | 20/22 | 9/22 | 1/13 | 100.0% | 11.86 s | 25.20 s |
| memo | 6 | 1/6 | 6/6 | 2/6 | 0/4 | 100.0% | 10.33 s | 15.24 s |
| minutes | 3 | 0/3 | 3/3 | 0/3 | 0/2 | 100.0% | 14.73 s | 16.32 s |
| notice | 11 | 0/9 | 7/9 | 6/11 | 3/10 | 90.9% | 6.55 s | 13.26 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 4.21 s | 4.21 s |
| report | 1 | – | – | 0/1 | 1/1 | 100.0% | 8.99 s | 8.99 s |
| statement | 2 | 0/2 | 2/2 | 1/2 | 1/1 | 100.0% | 13.33 s | 13.84 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 1/6 | – | 100.0% | 5.46 s | 7.94 s |
| native | 38 | 4/36 | 33/36 | 15/38 | 3/25 | 97.4% | 10.33 s | 25.20 s |
| office | 4 | 0/2 | 2/2 | 0/4 | 3/3 | 100.0% | 8.99 s | 16.32 s |
| scan | 7 | 2/7 | 6/7 | 3/7 | 1/5 | 100.0% | 14.76 s | 17.00 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 0/16 | 7/16 | 5/19 | 5/10 | 94.7% | 6.06 s | 9.24 s |
| 2-4 | 15 | 3/15 | 14/15 | 6/15 | 1/7 | 100.0% | 13.14 s | 22.83 s |
| 5-9 | 10 | 2/9 | 9/9 | 5/10 | 1/8 | 100.0% | 10.74 s | 16.32 s |
| 10-24 | 9 | 1/9 | 9/9 | 3/9 | 0/7 | 100.0% | 13.37 s | 25.20 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 15.24 s | 15.24 s |
| 50-99 | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 53.00 s | 53.00 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 1/11 | 11/11 | 3/11 | 0/8 | 0 of 11 | 14.66 s | 53.00 s | 3.87 s | 5.15 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 0/6 | 6/6 | 1/6 | 2 |
| advisory_committee | 2 | 0/2 | 2/2 | 0/2 | 0 |
| agency_guidance | 4 | 1/4 | 4/4 | 2/4 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_letter | 3 | 1/3 | 3/3 | 2/3 | 1 |
| as_of_date | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments | 5 | 1/5 | 5/5 | 2/5 | 1 |
| attachments_listed | 1 | 0/1 | 0/1 | 0/1 | 0 |
| attendance_list | 2 | 0/2 | 2/2 | 0/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 1/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| body_recounts_events | 1 | 1/1 | 1/1 | 1/1 | 0 |
| born_digital | 38 | 4/36 | 33/36 | 15/38 | 12 |
| care_of_counsel | 2 | 2/2 | 2/2 | 2/2 | 0 |
| charts | 1 | 0/1 | 1/1 | 1/1 | 0 |
| circular_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| competing_dates | 34 | 6/34 | 29/34 | 16/34 | 9 |
| court_notice | 5 | 0/3 | 1/3 | 3/5 | 6 |
| cover_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| cover_sheet | 2 | 0/2 | 2/2 | 0/2 | 2 |
| customer_first | 5 | 0/5 | 1/5 | 1/5 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 0/4 | 3/4 | 1/4 | 3 |
| deadline_in_body | 5 | 1/3 | 3/3 | 2/5 | 5 |
| determination | 1 | 0/1 | 1/1 | 0/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 0/3 | 3 |
| due_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| effective_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| email | 1 | 0/1 | 0/1 | 0/1 | 0 |
| email_printed_to_pdf | 1 | 0/1 | 0/1 | 0/1 | 0 |
| email_thread | 1 | 0/1 | 0/1 | 0/1 | 0 |
| enforcement_release | 3 | 0/3 | 3/3 | 3/3 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eu_body | 2 | 1/2 | 2/2 | 1/2 | 0 |
| financial_report | 1 | 0/1 | 1/1 | 1/1 | 0 |
| foi_release | 2 | 0/2 | 1/2 | 0/2 | 0 |
| foi_response | 5 | 0/5 | 5/5 | 1/5 | 2 |
| foreign_company | 1 | 0/1 | 1/1 | 0/1 | 0 |
| form | 3 | 0/2 | 2/2 | 0/3 | 2 |
| former_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 0/4 | 3/4 | 1/4 | 3 |
| government_form | 2 | 0/2 | 2/2 | 0/2 | 2 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_dates | 2 | 0/2 | 2/2 | 0/2 | 2 |
| historic | 6 | 0/6 | 0/6 | 1/6 | 0 |
| historical | 2 | 0/2 | 2/2 | 0/2 | 2 |
| image | 6 | 0/6 | 0/6 | 1/6 | 0 |
| invoice | 5 | 0/5 | 1/5 | 1/5 | 0 |
| invoices_attached | 2 | 0/2 | 2/2 | 0/2 | 0 |
| issuer_not_named | 2 | 0/2 | 2/2 | 0/2 | 0 |
| letter | 22 | 5/22 | 20/22 | 9/22 | 4 |
| letter_from_company | 3 | 0/3 | 2/3 | 0/3 | 0 |
| letterhead | 3 | 0/3 | 1/3 | 0/3 | 2 |
| liquidation | 3 | 0/3 | 2/3 | 1/3 | 3 |
| long_document | 5 | 1/5 | 5/5 | 2/5 | 1 |
| many_dates | 8 | 0/6 | 6/6 | 4/8 | 4 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo | 6 | 1/6 | 6/6 | 2/6 | 0 |
| memorandum | 4 | 0/4 | 4/4 | 0/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 3/3 | 0/3 | 3 |
| minutes | 3 | 0/3 | 3/3 | 0/3 | 1 |
| modern | 44 | 6/40 | 37/40 | 17/44 | 14 |
| month_only_date | 1 | – | – | 0/1 | 2 |
| multiple_letters | 2 | 0/2 | 2/2 | 0/2 | 1 |
| named_officials | 4 | 0/2 | 1/2 | 2/4 | 5 |
| named_person_at_org | 12 | 2/12 | 11/12 | 5/12 | 2 |
| names_removed | 1 | 0/1 | 0/1 | 0/1 | 0 |
| new_zealand | 10 | 2/10 | 9/10 | 3/10 | 4 |
| newsletter | 1 | – | – | 0/1 | 2 |
| no_action_letter | 2 | 1/2 | 2/2 | 1/2 | 0 |
| no_document_date | 3 | – | – | 1/3 | 3 |
| no_inside_address | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 1/4 | 1 |
| northern_ireland | 2 | – | – | 0/2 | 2 |
| notice | 11 | 0/9 | 7/9 | 6/11 | 8 |
| ocr_text_layer | 7 | 2/7 | 6/7 | 3/7 | 2 |
| office | 4 | 0/2 | 2/2 | 0/4 | 3 |
| paid_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 0/6 | 4/6 | 3/8 | 8 |
| publication_timestamp | 4 | 0/4 | 3/4 | 1/4 | 3 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 2/14 | 14/14 | 5/14 | 2 |
| regulator | 9 | 3/9 | 9/9 | 6/9 | 0 |
| reissued | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 0/1 | 0/1 | 0 |
| remittance_slip | 1 | 0/1 | 1/1 | 0/1 | 0 |
| reply_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 0/1 | 2 |
| request_letter | 2 | 0/2 | 2/2 | 0/2 | 0 |
| safety_recommendation | 2 | 0/2 | 2/2 | 0/2 | 1 |
| scanned | 7 | 2/7 | 6/7 | 3/7 | 2 |
| sender_at_end | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period | 2 | 0/2 | 2/2 | 2/2 | 0 |
| settlement | 2 | 0/2 | 2/2 | 2/2 | 0 |
| short | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| staff_names | 2 | 0/2 | 2/2 | 0/2 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 1/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 2/2 | 1/2 | 0 |
| tax_invoice | 1 | 0/1 | 1/1 | 0/1 | 0 |
| trading_name | 2 | 1/2 | 2/2 | 1/2 | 0 |
| two_addressees | 1 | 0/1 | 1/1 | 0/1 | 0 |
| type_not_printed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| uk | 8 | 0/6 | 5/6 | 1/8 | 4 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 26 | 3/24 | 22/24 | 12/26 | 9 |
| vat | 1 | 0/1 | 1/1 | 0/1 | 0 |
| wales | 6 | 0/6 | 5/6 | 1/6 | 2 |
| warning_letter | 3 | 2/3 | 3/3 | 2/3 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 178 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 18 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 10.33 s | 22.83 s | 53.00 s |
| `extraction_wall_ms` | 55 | 16.9 ms | 4.64 s | 35.97 s |
| `worker_total_ms` | 55 | 16.3 ms | 4.64 s | 35.95 s |
| `worker_snapshot_ms` | 55 | 0.27 ms | 1.1 ms | 2.2 ms |
| `worker_parse_ms` | 55 | 7.8 ms | 50.0 ms | 843.2 ms |
| `worker_analysis_ms` | 55 | 0.88 ms | 6.5 ms | 27.4 ms |
| `worker_render_ms` | 55 | 0.00 ms | 538.2 ms | 19.71 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 31.4 ms | 68.6 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 8.62 s | 70.38 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 555.0 ms | 6.11 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 7.82 s | 63.98 s |
| `worker_vision_ms` | 55 | 0.00 ms | 13.3 ms | 20.4 ms |
| `analyze_wall_ms` | 55 | 10.09 s | 17.03 s | 22.83 s |
| `distill_ms` | 55 | 2.4 ms | 21.4 ms | 79.6 ms |
| `prompt_ms` | 55 | 0.03 ms | 0.05 ms | 0.08 ms |
| `inference_ms` | 55 | 10.07 s | 16.90 s | 22.81 s |
| `prefill_ms` | 55 | 6.47 s | 12.99 s | 16.51 s |
| `generation_ms` | 55 | 3.60 s | 4.58 s | 6.29 s |
| `validation_ms` | 55 | 7.1 ms | 20.6 ms | 44.5 ms |
| `naming_ms` | 55 | 0.28 ms | 2.2 ms | 3.4 ms |
| `index_ms` | 55 | 2.3 ms | 20.7 ms | 74.5 ms |
| `retrieval_ms` | 55 | 0.04 ms | 1.9 ms | 4.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 167.1 tok/s | 174.1 tok/s | 177.5 tok/s |
| `generation_tok_per_s` | 55 | 26.0 tok/s | 27.8 tok/s | 28.0 tok/s |
| `prompt_tokens` | 55 | 1098 | 2251 | 2758 |
| `cached_tokens` | 55 | 469 | 488 | 488 |
| `generated_tokens` | 55 | 94 | 118 | 153 |
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
| fast | 18 | 3.5 ms | 17.3 ms | 3.0 ms | 16.5 ms | 13.7 ms | 2.0 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 11.2 ms | 47.2 ms | 10.6 ms | 46.0 ms | 37.5 ms | 6.3 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 1.58 s | 1.58 s | 1.58 s | 1.58 s | 4.3 ms | 3.0 ms | 49.4 ms | 1.49 s |
| ocr | 15 | 1.98 s | 35.97 s | 1.98 s | 35.95 s | 843.2 ms | 27.4 ms | 19.71 s | 70.38 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4.1 ms | 1.94 s | 3.6 ms | 1.94 s | 20.6 ms | 0.99 ms | 91.9 ms | 1.82 s |
| 2-4 | 15 | 10.5 ms | 3.55 s | 10.0 ms | 3.55 s | 843.2 ms | 3.0 ms | 451.7 ms | 4.90 s |
| 5-9 | 10 | 10.0 ms | 17.9 ms | 9.5 ms | 17.1 ms | 13.9 ms | 3.3 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 31.1 ms | 12.19 s | 29.6 ms | 12.19 s | 40.0 ms | 6.5 ms | 2.13 s | 22.87 s |
| 25-49 | 1 | 4.64 s | 4.64 s | 4.64 s | 4.64 s | 50.0 ms | 15.3 ms | 538.2 ms | 8.62 s |
| 50-99 | 1 | 35.97 s | 35.97 s | 35.95 s | 35.95 s | 132.8 ms | 27.4 ms | 19.71 s | 70.38 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 9.1 ms | 9.1 ms | 8.5 ms | 8.5 ms | 5.9 ms | 0.99 ms | 0.00 ms | 0.00 ms |
| form | 3 | 1.98 s | 2.75 s | 1.98 s | 2.75 s | 10.3 ms | 0.98 ms | 234.1 ms | 3.35 s |
| invoice | 5 | 651.5 ms | 878.0 ms | 651.0 ms | 877.6 ms | 20.6 ms | 0.51 ms | 0.00 ms | 766.4 ms |
| letter | 22 | 17.7 ms | 12.19 s | 16.9 ms | 12.19 s | 132.8 ms | 6.0 ms | 2.13 s | 22.87 s |
| memo | 6 | 10.5 ms | 4.64 s | 9.9 ms | 4.64 s | 50.0 ms | 15.3 ms | 538.2 ms | 8.62 s |
| minutes | 3 | 30.4 ms | 31.1 ms | 28.9 ms | 29.6 ms | 22.0 ms | 6.5 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 3.6 ms | 16.9 ms | 3.2 ms | 16.3 ms | 13.6 ms | 1.6 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 688.9 ms | 688.9 ms | 688.6 ms | 688.6 ms | 12.7 ms | 0.06 ms | 0.00 ms | 658.5 ms |
| report | 1 | 3.4 ms | 3.4 ms | 2.9 ms | 2.9 ms | 2.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.8 ms | 1.58 s | 1.4 ms | 1.58 s | 4.3 ms | 3.0 ms | 49.4 ms | 1.49 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 439 MB | 1381 MB | real5-letter-18 |
| Model server | 4427 MB | 4482 MB | real5-notice-09 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (45)

- **real5-letter-02**: expected `2018-08-31 Warning Letter to Slingshot Communications Limited.pdf`, got `2018-08-31 Letter to Fair Trading Act 1986.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter to Mr. Andrew Voveris President Triton Logistics Incorporated.pdf` · trap: party Triton Logistics Incorporated (recipient of a later letter in the file) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-05**: expected `2025-09-23 Safety Recommendation Letter to Federal Aviation Administration.pdf`, got `2025-09-23 Letter to Jennifer L. Homendy.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-06**: expected `2026-09-17 No-Action Letter to Eurex Clearing AG.pdf`, got `2026-09-17 No-Action Letter to Messrs. Graulich and Seinsheimer.pdf` · review: LOW_CONFIDENCE
- **real5-letter-07**: expected `2026-08-10 Request for No-Action Relief from Eurex Clearing AG.pdf`, got `2026-08-10 Letter to Division of Clearing and Risk.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-10**: expected `2026-07-21 Approval Letter to Delaware Health and Social Services.pdf`, got `2026-07-21 Letter from Delaware Health and Social Services.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-11**: expected `2026-06-02 Approval Letter to Oklahoma Health Care Authority.pdf`, got `2026-06-02 SoonerSelect Contract between Melissa Miller and Centers for Medicare & Medicaid Services.pdf` · trap: party Melissa Miller (addressee, the Medicaid Director) · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `2024-02-14 Letter from ATISN 20183 - FOI Request Acquisition of Companies House, Crown Way, Cardiff.pdf` · trap: party Companies House (building the request is about) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Welsh Government.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-14**: expected `2025-03-03 Letter to European Commission.pdf`, got `2025-03-03 Letter to Directorate General Financial Services.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `2026-04-20 Letter to Commissioner McGrath.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2021-07-23 Tax Invoice from Allen+Clarke.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Letter to New World.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Document - Tatou NZ Ltd.pdf` · trap: party Tatou NZ Ltd (company the request is about) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `Email - Godrer Graig - NPTCBC - 7234e.05 -Oct 2025.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter to Division of Market Oversight.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real5-notice-01**: expected `2025-10-01 Notice of Intention to Remove Company From the Register for RS2022 Limited.pdf`, got `2025-10-01 Notice of Intention to Remove Company for Hamish Pryde.pdf` · trap: party Hamish Pryde (liquidator, a person) · **filed without review**
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Document - Kieran Jones.pdf` · trap: party Kieran Jones (joint liquidator, a person) · review: TYPE_UNSUPPORTED, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document.pdf` · review: TYPE_UNSUPPORTED, PARTY_MISSING
- **real5-notice-04**: expected `2026-10-30 Advertisement of Application for Putting Company into Liquidation with Commissioner of Inland Revenue.pdf`, got `2026-10-02 Advertisement of Application for Putting Company into Liquidation for Rogan Construction Limited.pdf` · trap: date 2026-10-02 (date the application was filed) · review: LOW_CONFIDENCE
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-11-30 Public Notice from Edward A. Friedland.pdf` · trap: date 2025-11-30 (date the magistrate judge's term expires); party Edward A. Friedland (district court executive) · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice from United States District Court Northern District of Texas.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Document - Interactive Brokers LLC.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Document - ShapeShift AG.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real5-notice-11**: expected `2025-12-04 Enforcement Release for Gracetown, Inc.pdf`, got `2025-12-04 Document - Gracetown, Inc.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document from Sternberg Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document from teme10.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document from Wateloo Hl.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document from Hwholesale Clothiers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from SoYG New York 189.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-06**: expected `2025-09-09 Tax Invoice from Ian Davies Plant Limited.pdf`, got `2025-09-09 Tax Invoice from Neath Port Talbot C. B. Council The Quays.pdf` · review: LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document - Barbara LaCour.pdf` · trap: party Barbara LaCour (officer who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `1982-03-04 Document - Raymond A. O'BRIEN.pdf` · trap: party Raymond A. O'Brien (person to confer with) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `2025-05-16 General Counsel Memorandum to All Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Memorandum for Heads of Executive Departments and Agencies.pdf` · review: PARTY_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- … and 5 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
