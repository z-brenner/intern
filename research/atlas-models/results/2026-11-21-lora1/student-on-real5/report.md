# InternBench: live run

- **Run:** 2026-11-21T09:37:40Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 9V74 80-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `99e3ad6efd92`
- **Corpus:** 55 documents · 55 completed · gold `dc976e7dd9`
- **Wall time:** 452.50 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 8/51 (15.7%) |
| Document type | 37/55 (67.3%) |
| Date | 40/51 (78.4%) |
| Date role (when the reviewed date was chosen) | 33/38 (86.8%) |
| Parties | 31/55 (56.4%) |
| Relation word (when parties were named) | 32/50 (64.0%) |
| Party in the right role (when parties were named) | 17/40 (42.5%) |
| Ready / review routing | 7/33 (21.2%) |
| Description states nothing false | 54/55 (98.2%) |
| Description is specific | 26/55 (47.3%) |
| Description specificity | 53.3% (mean of 55) |
| Evidence recall (model's quotes) | 62.0% (mean of 54) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 81.5% (mean of 54) |
| Sent to review | 98.2% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 18.2% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 81.8% | 55 |
| Review rate | 98.2% | 55 |
| Evidence recall | 62.0% | 54 |
| Total latency p50 | 6.40 s | 55 |
| Total latency p95 | 13.45 s | 55 |
| Generation latency p50 | 2.01 s | 55 |
| Generation latency p95 | 3.03 s | 55 |
| Generated tokens p50 | 80 | 55 |
| Generated tokens p95 | 116 | 55 |
| Prompt tokens p50 | 1206 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 51 |
| Trap date chosen | 2 of 54 |
| Forbidden party named | 2 of 55 |
| Spurious parties named | 23 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 142 (0.7%) |
| Right name sent to review anyway | 7 of 27 |
| Sent to review for an unsupported fact | 45 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 13.45 s | 13.45 s |
| form | 3 | 0/2 | 2/2 | 0/3 | 1/1 | 100.0% | 9.25 s | 9.97 s |
| invoice | 5 | 0/5 | 1/5 | 1/5 | 0/1 | 100.0% | 4.21 s | 4.91 s |
| letter | 22 | 8/22 | 19/22 | 17/22 | 1/13 | 100.0% | 7.09 s | 25.49 s |
| memo | 6 | 0/6 | 6/6 | 2/6 | 0/4 | 100.0% | 6.65 s | 13.25 s |
| minutes | 3 | 0/3 | 3/3 | 2/3 | 0/2 | 100.0% | 9.32 s | 9.87 s |
| notice | 11 | 0/9 | 7/9 | 8/11 | 3/10 | 90.9% | 3.82 s | 7.91 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 3.23 s | 3.23 s |
| report | 1 | – | – | 0/1 | 1/1 | 100.0% | 5.52 s | 5.52 s |
| statement | 2 | 0/2 | 2/2 | 1/2 | 1/1 | 100.0% | 9.97 s | 11.99 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 1/6 | – | 100.0% | 3.94 s | 4.68 s |
| native | 38 | 6/36 | 32/36 | 24/38 | 3/25 | 97.4% | 6.40 s | 25.49 s |
| office | 4 | 0/2 | 2/2 | 1/4 | 3/3 | 100.0% | 5.52 s | 11.99 s |
| scan | 7 | 2/7 | 6/7 | 5/7 | 1/5 | 100.0% | 11.98 s | 13.25 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 0/16 | 7/16 | 9/19 | 4/10 | 100.0% | 3.76 s | 6.40 s |
| 2-4 | 15 | 4/15 | 14/15 | 8/15 | 1/7 | 100.0% | 9.97 s | 13.45 s |
| 5-9 | 10 | 2/9 | 9/9 | 6/10 | 2/8 | 90.0% | 6.57 s | 9.87 s |
| 10-24 | 9 | 2/9 | 9/9 | 7/9 | 0/7 | 100.0% | 8.58 s | 25.49 s |
| 25-49 | 1 | 0/1 | 1/1 | 1/1 | 0/1 | 100.0% | 13.25 s | 13.25 s |
| 50-99 | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 62.35 s | 62.35 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 2/11 | 10/11 | 8/11 | 0/8 | 0 of 11 | 8.71 s | 62.35 s | 2.00 s | 3.09 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 1/6 | 5/6 | 3/6 | 0 |
| advisory_committee | 2 | 0/2 | 2/2 | 2/2 | 0 |
| agency_guidance | 4 | 0/4 | 4/4 | 0/4 | 0 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 3 | 1/3 | 3/3 | 2/3 | 0 |
| as_of_date | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments | 5 | 1/5 | 4/5 | 3/5 | 0 |
| attachments_listed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| attendance_list | 2 | 0/2 | 2/2 | 2/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 0/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 1/1 | 0 |
| born_digital | 38 | 6/36 | 32/36 | 24/38 | 2 |
| care_of_counsel | 2 | 1/2 | 2/2 | 2/2 | 0 |
| charts | 1 | 0/1 | 1/1 | 0/1 | 0 |
| circular_letter | 2 | 0/2 | 2/2 | 0/2 | 0 |
| competing_dates | 34 | 7/34 | 29/34 | 21/34 | 1 |
| court_notice | 5 | 0/3 | 2/3 | 3/5 | 1 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 2/2 | 0/2 | 0 |
| customer_first | 5 | 0/5 | 1/5 | 1/5 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 0/4 | 2/4 | 3/4 | 0 |
| deadline_in_body | 5 | 1/3 | 2/3 | 3/5 | 1 |
| determination | 1 | 0/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 0/3 | 2 |
| due_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| effective_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_printed_to_pdf | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_thread | 1 | 0/1 | 0/1 | 0/1 | 1 |
| enforcement_release | 3 | 0/3 | 3/3 | 3/3 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 1/1 | 1/1 | 1/1 | 0 |
| eu_body | 2 | 1/2 | 2/2 | 2/2 | 0 |
| financial_report | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_release | 2 | 0/2 | 1/2 | 1/2 | 1 |
| foi_response | 5 | 0/5 | 4/5 | 2/5 | 0 |
| foreign_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 3 | 0/2 | 2/2 | 0/3 | 0 |
| former_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 0/4 | 2/4 | 3/4 | 0 |
| government_form | 2 | 0/2 | 2/2 | 0/2 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_dates | 2 | 0/2 | 2/2 | 0/2 | 0 |
| historic | 6 | 0/6 | 0/6 | 1/6 | 0 |
| historical | 2 | 0/2 | 2/2 | 0/2 | 0 |
| image | 6 | 0/6 | 0/6 | 1/6 | 0 |
| invoice | 5 | 0/5 | 1/5 | 1/5 | 0 |
| invoices_attached | 2 | 0/2 | 1/2 | 1/2 | 0 |
| issuer_not_named | 2 | 0/2 | 2/2 | 0/2 | 0 |
| letter | 22 | 8/22 | 19/22 | 17/22 | 0 |
| letter_from_company | 3 | 2/3 | 2/3 | 3/3 | 0 |
| letterhead | 3 | 1/3 | 2/3 | 2/3 | 0 |
| liquidation | 3 | 0/3 | 1/3 | 2/3 | 0 |
| long_document | 5 | 1/5 | 4/5 | 3/5 | 0 |
| many_dates | 8 | 1/6 | 5/6 | 4/8 | 1 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memo | 6 | 0/6 | 6/6 | 2/6 | 0 |
| memorandum | 4 | 0/4 | 4/4 | 2/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 3/3 | 0/3 | 1 |
| minutes | 3 | 0/3 | 3/3 | 2/3 | 1 |
| modern | 44 | 8/40 | 36/40 | 28/44 | 3 |
| month_only_date | 1 | – | – | 0/1 | 1 |
| multiple_letters | 2 | 0/2 | 2/2 | 1/2 | 0 |
| named_officials | 4 | 0/2 | 2/2 | 2/4 | 1 |
| named_person_at_org | 12 | 5/12 | 11/12 | 10/12 | 0 |
| names_removed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| new_zealand | 10 | 2/10 | 7/10 | 7/10 | 0 |
| newsletter | 1 | – | – | 0/1 | 1 |
| no_action_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| no_document_date | 3 | – | – | 1/3 | 1 |
| no_inside_address | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 1/4 | 1 |
| northern_ireland | 2 | – | – | 0/2 | 1 |
| notice | 11 | 0/9 | 7/9 | 8/11 | 1 |
| ocr_text_layer | 7 | 2/7 | 6/7 | 5/7 | 0 |
| office | 4 | 0/2 | 2/2 | 1/4 | 2 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 0/6 | 4/6 | 5/8 | 1 |
| publication_timestamp | 4 | 0/4 | 2/4 | 3/4 | 0 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 1/14 | 13/14 | 7/14 | 0 |
| regulator | 9 | 3/9 | 9/9 | 9/9 | 0 |
| reissued | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| remittance_slip | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reply_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 0/1 | 1 |
| request_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| safety_recommendation | 2 | 0/2 | 2/2 | 1/2 | 0 |
| scanned | 7 | 2/7 | 6/7 | 5/7 | 0 |
| sender_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 0/2 | 2/2 | 2/2 | 0 |
| settlement | 2 | 0/2 | 2/2 | 2/2 | 0 |
| short | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 2 | 0/2 | 2/2 | 2/2 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 1/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 2/2 | 0/2 | 0 |
| tax_invoice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| trading_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| uk | 8 | 0/6 | 5/6 | 3/8 | 3 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 26 | 2/24 | 23/24 | 15/26 | 1 |
| vat | 1 | 0/1 | 1/1 | 1/1 | 0 |
| wales | 6 | 0/6 | 5/6 | 3/6 | 2 |
| warning_letter | 3 | 2/3 | 3/3 | 3/3 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 178 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 18 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 6.40 s | 13.45 s | 62.35 s |
| `extraction_wall_ms` | 55 | 16.9 ms | 5.91 s | 50.85 s |
| `worker_total_ms` | 55 | 16.2 ms | 5.91 s | 50.84 s |
| `worker_snapshot_ms` | 55 | 0.24 ms | 1.1 ms | 2.4 ms |
| `worker_parse_ms` | 55 | 9.8 ms | 61.8 ms | 951.6 ms |
| `worker_analysis_ms` | 55 | 0.93 ms | 7.9 ms | 30.6 ms |
| `worker_render_ms` | 55 | 0.00 ms | 628.7 ms | 25.02 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 58.9 ms | 93.4 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 11.00 s | 99.52 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 687.1 ms | 7.96 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 10.05 s | 91.23 s |
| `worker_vision_ms` | 55 | 0.00 ms | 19.2 ms | 27.4 ms |
| `analyze_wall_ms` | 55 | 6.39 s | 11.50 s | 13.44 s |
| `distill_ms` | 55 | 2.9 ms | 27.3 ms | 96.3 ms |
| `prompt_ms` | 55 | 0.03 ms | 0.06 ms | 0.09 ms |
| `inference_ms` | 55 | 6.39 s | 11.31 s | 13.41 s |
| `prefill_ms` | 55 | 4.04 s | 9.39 s | 10.34 s |
| `generation_ms` | 55 | 2.01 s | 3.03 s | 3.09 s |
| `validation_ms` | 55 | 8.0 ms | 23.6 ms | 80.4 ms |
| `naming_ms` | 55 | 0.34 ms | 2.8 ms | 4.3 ms |
| `index_ms` | 55 | 2.7 ms | 24.9 ms | 90.3 ms |
| `retrieval_ms` | 55 | 0.05 ms | 2.3 ms | 5.1 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 295.6 tok/s | 306.5 tok/s | 311.0 tok/s |
| `generation_tok_per_s` | 55 | 40.6 tok/s | 43.7 tok/s | 44.2 tok/s |
| `prompt_tokens` | 55 | 1206 | 2678 | 2878 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 80 | 116 | 120 |
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
| fast | 18 | 4.0 ms | 22.0 ms | 3.6 ms | 21.2 ms | 18.1 ms | 2.3 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 12.9 ms | 56.8 ms | 12.2 ms | 55.4 ms | 46.3 ms | 7.2 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 2.04 s | 2.04 s | 2.04 s | 2.04 s | 5.0 ms | 3.4 ms | 49.4 ms | 1.95 s |
| ocr | 15 | 2.83 s | 50.85 s | 2.83 s | 50.84 s | 951.6 ms | 30.6 ms | 25.02 s | 99.52 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4.4 ms | 2.46 s | 4.1 ms | 2.46 s | 17.8 ms | 1.1 ms | 117.9 ms | 2.30 s |
| 2-4 | 15 | 12.7 ms | 4.53 s | 12.2 ms | 4.53 s | 951.6 ms | 3.4 ms | 625.3 ms | 6.73 s |
| 5-9 | 10 | 12.4 ms | 22.0 ms | 11.9 ms | 21.3 ms | 17.8 ms | 3.7 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 40.0 ms | 17.98 s | 38.1 ms | 17.98 s | 50.4 ms | 7.9 ms | 2.41 s | 34.01 s |
| 25-49 | 1 | 5.91 s | 5.91 s | 5.91 s | 5.91 s | 61.8 ms | 17.4 ms | 628.7 ms | 11.00 s |
| 50-99 | 1 | 50.85 s | 50.85 s | 50.84 s | 50.84 s | 159.2 ms | 30.6 ms | 25.02 s | 99.52 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 10.0 ms | 10.0 ms | 9.5 ms | 9.5 ms | 6.9 ms | 1.1 ms | 0.00 ms | 0.00 ms |
| form | 3 | 2.83 s | 3.57 s | 2.83 s | 3.57 s | 12.6 ms | 0.93 ms | 265.3 ms | 4.34 s |
| invoice | 5 | 958.2 ms | 1.27 s | 954.1 ms | 1.27 s | 17.8 ms | 0.53 ms | 0.00 ms | 1.13 s |
| letter | 22 | 22.0 ms | 17.98 s | 21.3 ms | 17.98 s | 159.2 ms | 6.9 ms | 2.41 s | 34.01 s |
| memo | 6 | 12.0 ms | 5.91 s | 11.6 ms | 5.91 s | 61.8 ms | 17.4 ms | 628.7 ms | 11.00 s |
| minutes | 3 | 35.7 ms | 40.0 ms | 34.0 ms | 38.1 ms | 29.0 ms | 7.9 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 3.6 ms | 20.2 ms | 3.3 ms | 19.6 ms | 16.8 ms | 1.8 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.02 s | 1.02 s | 1.02 s | 1.02 s | 10.5 ms | 0.04 ms | 0.00 ms | 986.0 ms |
| report | 1 | 4.4 ms | 4.4 ms | 3.9 ms | 3.9 ms | 3.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.6 ms | 2.04 s | 2.0 ms | 2.04 s | 5.0 ms | 3.4 ms | 49.4 ms | 1.95 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 406 MB | 1332 MB | real5-letter-18 |
| Model server | 846 MB | 847 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (43)

- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `2015-12-16 Warning Letter from Coast to Coast Limited.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter to National Transportation Safety Board.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-05**: expected `2025-09-23 Safety Recommendation Letter to Federal Aviation Administration.pdf`, got `2025-09-23 Letter to Textron Aviation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-06**: expected `2026-09-17 No-Action Letter to Eurex Clearing AG.pdf`, got `2026-09-17 No-Action Letter regarding Commission Regulations 1.49(c)(1), 1.49(e)(1)(i), and to Eurex Clearing AG.pdf` · review: LOW_CONFIDENCE
- **real5-letter-09**: expected `2025-06-23 Approval Letter to Maryland Department of Health.pdf`, got `2025-06-23 Letter to Centers for Medicare & Medicaid Services.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-11**: expected `2026-06-02 Approval Letter to Oklahoma Health Care Authority.pdf`, got `2026-06-02 SoonerSelect Contract between Oklahoma Health Care Authority and Centers for Medicare & Medicaid Services.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `2024-02-14 Letter to Welsh Government.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Miller Research Payment (2) Information requested Thank you for your request.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `2026-04-20 Guidelines for the Registration of Organisations and Issuance of Recommendations for European Commission.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2020-06-28 TAX INVOICE from Motu Economic & Public Policy Research.pdf` · review: LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Tax Invoice from Ministry for Regulation.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Document - Secretary of Foreign Affairs and Trade.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document from A. E. Schreiber.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2025-12-03 Claim Forecast from Llywodraeth Cymru.pdf` · trap: date 2025-12-03 (date of an earlier e-mail in the thread) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-01**: expected `2025-10-01 Notice of Intention to Remove Company From the Register for RS2022 Limited.pdf`, got `Notice of Intention to Remove Company for RS2022 LIMITED.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Notice of Intention to Remove Companies from New Zealand Gazette.pdf` · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document from Environmental Protection Authority.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-04**: expected `2026-10-30 Advertisement of Application for Putting Company into Liquidation with Commissioner of Inland Revenue.pdf`, got `Advertisement of Application for Putting Company into Liquidation for Rogan Construction Limited.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice for Edward Friedland.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice for United States District Court for the Northern District of Texas.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Enforcement Release from Interactive Brokers LLC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Enforcement Release from ShapeShift AG.pdf` · **filed without review**
- **real5-notice-11**: expected `2025-12-04 Enforcement Release for Gracetown, Inc.pdf`, got `2025-12-04 Document from Office of Foreign Assets Control.pdf` · review: TYPE_UNSUPPORTED
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document between Sternberg Brothers and Manufacturers of Pantaloons.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document from Wateloo Hl.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document from D.CREHANGE.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from HOSOY $1ea143.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-06**: expected `2025-09-09 Tax Invoice from Ian Davies Plant Limited.pdf`, got `2025-09-09 Tax Invoice from Ian Davies Plant Limited Llwynfelish Farm Neath Road Crynant Neath.pdf` · review: LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report for Public Debt.pdf` · review: LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document from N1-310-90-001.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `1982-03-04 Document from National Archives and Records Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `2025-05-16 General Counsel Memorandum to All Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 OMB Memorandum M-25-13 is rescinded from Office of Management and Budget.pdf` · review: LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies from Office of Management and Budget.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-05**: expected `2025-06-03 Mortgagee Letter from U.S. Department of Housing and Urban Development.pdf`, got `2025-06-03 Mortgagee Letter from Mortgagee Letter 2025-14.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-06**: expected `2025-06-27 Mortgagee Letter from U.S. Department of Housing and Urban Development.pdf`, got `2025-06-27 Mortgagee Letter from Mortgagee Letter 2025-19.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-09**: expected `2024-10-29 Meeting Minutes for Visiting Committee on Advanced Technology.pdf`, got `2024-10-29 Minutes for NIST Staff.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- … and 3 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
