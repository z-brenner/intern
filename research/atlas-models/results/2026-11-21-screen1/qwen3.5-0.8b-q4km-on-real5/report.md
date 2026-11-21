# InternBench: live run

- **Run:** 2026-11-21T04:51:36Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 55 documents · 55 completed · gold `dc976e7dd9`
- **Wall time:** 556.57 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 6/51 (11.8%) |
| Document type | 36/55 (65.5%) |
| Date | 33/51 (64.7%) |
| Date role (when the reviewed date was chosen) | 8/32 (25.0%) |
| Parties | 18/55 (32.7%) |
| Relation word (when parties were named) | 21/45 (46.7%) |
| Party in the right role (when parties were named) | 11/32 (34.4%) |
| Ready / review routing | 6/33 (18.2%) |
| Description states nothing false | 54/55 (98.2%) |
| Description is specific | 24/55 (43.6%) |
| Description specificity | 44.9% (mean of 55) |
| Evidence recall (model's quotes) | 24.7% (mean of 54) |
| Digest recall (facts reaching the digest) | 83.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 81.5% (mean of 54) |
| Sent to review | 100.0% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 18.2% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 90.9% | 55 |
| Review rate | 100.0% | 55 |
| Evidence recall | 24.7% | 54 |
| Total latency p50 | 8.00 s | 55 |
| Total latency p95 | 17.59 s | 55 |
| Generation latency p50 | 3.77 s | 55 |
| Generation latency p95 | 4.84 s | 55 |
| Generated tokens p50 | 98 | 55 |
| Generated tokens p95 | 124 | 55 |
| Prompt tokens p50 | 1206 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 51 |
| Trap date chosen | 6 of 54 |
| Forbidden party named | 10 of 55 |
| Spurious parties named | 32 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 120 (0.8%) |
| Right name sent to review anyway | 5 of 27 |
| Sent to review for an unsupported fact | 50 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 17.59 s | 17.59 s |
| form | 3 | 0/2 | 1/2 | 0/3 | 1/1 | 100.0% | 11.10 s | 12.12 s |
| invoice | 5 | 0/5 | 0/5 | 1/5 | 0/1 | 100.0% | 6.20 s | 6.54 s |
| letter | 22 | 5/22 | 15/22 | 13/22 | 1/13 | 100.0% | 8.65 s | 29.82 s |
| memo | 6 | 0/6 | 4/6 | 1/6 | 0/4 | 100.0% | 8.29 s | 14.05 s |
| minutes | 3 | 0/3 | 3/3 | 0/3 | 0/2 | 100.0% | 10.52 s | 10.72 s |
| notice | 11 | 1/9 | 8/9 | 3/11 | 2/10 | 100.0% | 5.38 s | 9.73 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 4.21 s | 4.21 s |
| report | 1 | – | – | 0/1 | 1/1 | 100.0% | 6.42 s | 6.42 s |
| statement | 2 | 0/2 | 2/2 | 0/2 | 1/1 | 100.0% | 12.38 s | 13.60 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 1/6 | – | 100.0% | 5.89 s | 7.30 s |
| native | 38 | 5/36 | 28/36 | 13/38 | 2/25 | 100.0% | 8.00 s | 29.82 s |
| office | 4 | 0/2 | 2/2 | 0/4 | 3/3 | 100.0% | 6.42 s | 13.60 s |
| scan | 7 | 1/7 | 3/7 | 4/7 | 1/5 | 100.0% | 13.50 s | 14.57 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 1/16 | 8/16 | 6/19 | 4/10 | 100.0% | 5.17 s | 7.86 s |
| 2-4 | 15 | 2/15 | 7/15 | 6/15 | 1/7 | 100.0% | 11.81 s | 17.59 s |
| 5-9 | 10 | 1/9 | 8/9 | 4/10 | 1/8 | 100.0% | 8.00 s | 11.54 s |
| 10-24 | 9 | 2/9 | 9/9 | 2/9 | 0/7 | 100.0% | 10.52 s | 29.82 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 14.05 s | 14.05 s |
| 50-99 | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 71.63 s | 71.63 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 2/11 | 10/11 | 2/11 | 0/8 | 0 of 11 | 10.53 s | 71.63 s | 3.87 s | 5.01 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 1/6 | 4/6 | 2/6 | 0 |
| advisory_committee | 2 | 0/2 | 2/2 | 0/2 | 0 |
| agency_guidance | 4 | 0/4 | 2/4 | 0/4 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_letter | 3 | 1/3 | 3/3 | 1/3 | 0 |
| as_of_date | 2 | 0/2 | 2/2 | 0/2 | 0 |
| attachments | 5 | 1/5 | 4/5 | 1/5 | 0 |
| attachments_listed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| attendance_list | 2 | 0/2 | 2/2 | 0/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 0/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| body_recounts_events | 1 | 0/1 | 0/1 | 1/1 | 0 |
| born_digital | 38 | 5/36 | 28/36 | 13/38 | 11 |
| care_of_counsel | 2 | 0/2 | 0/2 | 2/2 | 0 |
| charts | 1 | 0/1 | 1/1 | 0/1 | 0 |
| circular_letter | 2 | 0/2 | 2/2 | 0/2 | 0 |
| competing_dates | 34 | 5/34 | 24/34 | 13/34 | 8 |
| court_notice | 5 | 1/3 | 3/3 | 2/5 | 5 |
| cover_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| cover_sheet | 2 | 0/2 | 1/2 | 0/2 | 3 |
| customer_first | 5 | 0/5 | 0/5 | 1/5 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 1/4 | 4/4 | 1/4 | 2 |
| deadline_in_body | 5 | 0/3 | 3/3 | 1/5 | 6 |
| determination | 1 | 0/1 | 1/1 | 0/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 0/3 | 2 |
| due_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| effective_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_printed_to_pdf | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_thread | 1 | 0/1 | 0/1 | 0/1 | 1 |
| enforcement_release | 3 | 0/3 | 2/3 | 1/3 | 1 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eu_body | 2 | 1/2 | 1/2 | 1/2 | 1 |
| financial_report | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_release | 2 | 0/2 | 0/2 | 1/2 | 1 |
| foi_response | 5 | 0/5 | 3/5 | 1/5 | 0 |
| foreign_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 3 | 0/2 | 1/2 | 0/3 | 3 |
| former_name | 1 | 0/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 1/4 | 4/4 | 1/4 | 2 |
| government_form | 2 | 0/2 | 1/2 | 0/2 | 3 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_dates | 2 | 0/2 | 1/2 | 0/2 | 3 |
| historic | 6 | 0/6 | 0/6 | 1/6 | 0 |
| historical | 2 | 0/2 | 1/2 | 0/2 | 3 |
| image | 6 | 0/6 | 0/6 | 1/6 | 0 |
| invoice | 5 | 0/5 | 0/5 | 1/5 | 0 |
| invoices_attached | 2 | 0/2 | 1/2 | 0/2 | 0 |
| issuer_not_named | 2 | 0/2 | 0/2 | 0/2 | 0 |
| letter | 22 | 5/22 | 15/22 | 13/22 | 2 |
| letter_from_company | 3 | 1/3 | 2/3 | 3/3 | 0 |
| letterhead | 3 | 1/3 | 2/3 | 2/3 | 1 |
| liquidation | 3 | 1/3 | 3/3 | 1/3 | 2 |
| long_document | 5 | 1/5 | 4/5 | 1/5 | 0 |
| many_dates | 8 | 0/6 | 4/6 | 3/8 | 4 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo | 6 | 0/6 | 4/6 | 1/6 | 0 |
| memorandum | 4 | 0/4 | 2/4 | 1/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 2/3 | 0/3 | 4 |
| minutes | 3 | 0/3 | 3/3 | 0/3 | 1 |
| modern | 44 | 6/40 | 30/40 | 16/44 | 12 |
| month_only_date | 1 | – | – | 0/1 | 1 |
| multiple_letters | 2 | 0/2 | 2/2 | 0/2 | 1 |
| named_officials | 4 | 0/2 | 2/2 | 1/4 | 5 |
| named_person_at_org | 12 | 3/12 | 9/12 | 7/12 | 2 |
| names_removed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| new_zealand | 10 | 2/10 | 7/10 | 4/10 | 2 |
| newsletter | 1 | – | – | 0/1 | 1 |
| no_action_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| no_document_date | 3 | – | – | 0/3 | 4 |
| no_inside_address | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 0/4 | 1 |
| northern_ireland | 2 | – | – | 0/2 | 1 |
| notice | 11 | 1/9 | 8/9 | 3/11 | 8 |
| ocr_text_layer | 7 | 1/7 | 3/7 | 4/7 | 3 |
| office | 4 | 0/2 | 2/2 | 0/4 | 2 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 0/1 | 0/1 | 1/1 | 1 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 1/6 | 6/6 | 2/8 | 7 |
| publication_timestamp | 4 | 1/4 | 4/4 | 1/4 | 2 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 1/14 | 9/14 | 5/14 | 0 |
| regulator | 9 | 2/9 | 6/9 | 6/9 | 1 |
| reissued | 1 | 0/1 | 0/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| remittance_slip | 1 | 0/1 | 0/1 | 1/1 | 0 |
| reply_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 0/1 | 1 |
| request_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| safety_recommendation | 2 | 0/2 | 2/2 | 0/2 | 1 |
| scanned | 7 | 1/7 | 3/7 | 4/7 | 3 |
| sender_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 0/2 | 2/2 | 0/2 | 0 |
| settlement | 2 | 0/2 | 2/2 | 0/2 | 0 |
| short | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| staff_names | 2 | 0/2 | 2/2 | 0/2 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 0/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 1/2 | 0/2 | 0 |
| tax_invoice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| trading_name | 2 | 1/2 | 1/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 0/1 | 1 |
| type_not_printed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| uk | 8 | 0/6 | 3/6 | 2/8 | 3 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 26 | 2/24 | 19/24 | 7/26 | 10 |
| vat | 1 | 0/1 | 0/1 | 1/1 | 0 |
| wales | 6 | 0/6 | 3/6 | 2/6 | 2 |
| warning_letter | 3 | 1/3 | 1/3 | 3/3 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 178 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 18 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 8.00 s | 17.59 s | 71.63 s |
| `extraction_wall_ms` | 55 | 16.6 ms | 5.66 s | 57.80 s |
| `worker_total_ms` | 55 | 15.9 ms | 5.66 s | 57.79 s |
| `worker_snapshot_ms` | 55 | 0.25 ms | 1.2 ms | 2.5 ms |
| `worker_parse_ms` | 55 | 9.0 ms | 60.3 ms | 1.64 s |
| `worker_analysis_ms` | 55 | 0.94 ms | 7.9 ms | 31.7 ms |
| `worker_render_ms` | 55 | 0.00 ms | 627.5 ms | 23.12 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 57.5 ms | 84.7 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 10.31 s | 113.64 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 739.8 ms | 7.00 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 9.31 s | 106.41 s |
| `worker_vision_ms` | 55 | 0.00 ms | 18.4 ms | 27.1 ms |
| `analyze_wall_ms` | 55 | 7.98 s | 13.59 s | 17.58 s |
| `distill_ms` | 55 | 2.9 ms | 27.3 ms | 96.4 ms |
| `prompt_ms` | 55 | 0.03 ms | 0.06 ms | 0.11 ms |
| `inference_ms` | 55 | 7.95 s | 13.59 s | 17.57 s |
| `prefill_ms` | 55 | 4.00 s | 9.59 s | 10.40 s |
| `generation_ms` | 55 | 3.77 s | 4.84 s | 7.16 s |
| `validation_ms` | 55 | 7.3 ms | 22.9 ms | 64.8 ms |
| `naming_ms` | 55 | 0.36 ms | 3.0 ms | 4.8 ms |
| `index_ms` | 55 | 2.8 ms | 24.9 ms | 88.5 ms |
| `retrieval_ms` | 55 | 0.05 ms | 2.3 ms | 5.8 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 294.0 tok/s | 302.4 tok/s | 310.0 tok/s |
| `generation_tok_per_s` | 55 | 27.2 tok/s | 29.1 tok/s | 29.8 tok/s |
| `prompt_tokens` | 55 | 1206 | 2678 | 2878 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 98 | 124 | 173 |
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
| fast | 18 | 3.5 ms | 20.7 ms | 3.2 ms | 19.8 ms | 16.6 ms | 2.2 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 12.9 ms | 56.1 ms | 12.2 ms | 54.7 ms | 44.5 ms | 7.1 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 1.75 s | 1.75 s | 1.75 s | 1.75 s | 4.7 ms | 3.4 ms | 61.0 ms | 1.66 s |
| ocr | 15 | 2.84 s | 57.80 s | 2.84 s | 57.79 s | 1.64 s | 31.7 ms | 23.12 s | 113.64 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4.4 ms | 2.42 s | 4.1 ms | 2.42 s | 11.0 ms | 1.2 ms | 104.8 ms | 2.29 s |
| 2-4 | 15 | 12.4 ms | 5.10 s | 12.1 ms | 5.10 s | 1.64 s | 3.4 ms | 574.1 ms | 6.38 s |
| 5-9 | 10 | 12.1 ms | 21.2 ms | 11.6 ms | 20.4 ms | 16.9 ms | 3.8 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 39.2 ms | 19.83 s | 37.5 ms | 19.83 s | 47.9 ms | 7.9 ms | 2.38 s | 37.47 s |
| 25-49 | 1 | 5.66 s | 5.66 s | 5.66 s | 5.66 s | 60.3 ms | 20.6 ms | 627.5 ms | 10.31 s |
| 50-99 | 1 | 57.80 s | 57.80 s | 57.79 s | 57.79 s | 154.6 ms | 31.7 ms | 23.12 s | 113.64 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 9.9 ms | 9.9 ms | 9.4 ms | 9.4 ms | 6.8 ms | 1.1 ms | 0.00 ms | 0.00 ms |
| form | 3 | 2.84 s | 3.46 s | 2.84 s | 3.46 s | 11.7 ms | 1.00 ms | 268.7 ms | 4.41 s |
| invoice | 5 | 1.11 s | 1.34 s | 1.11 s | 1.34 s | 11.0 ms | 0.55 ms | 0.00 ms | 1.22 s |
| letter | 22 | 21.2 ms | 19.83 s | 20.4 ms | 19.83 s | 154.6 ms | 7.0 ms | 2.38 s | 37.47 s |
| memo | 6 | 11.6 ms | 5.66 s | 11.2 ms | 5.66 s | 60.3 ms | 20.6 ms | 627.5 ms | 10.31 s |
| minutes | 3 | 35.2 ms | 39.2 ms | 33.5 ms | 37.5 ms | 28.4 ms | 7.9 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 3.8 ms | 19.4 ms | 3.5 ms | 18.6 ms | 15.8 ms | 1.8 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.32 s | 1.32 s | 1.32 s | 1.32 s | 8.0 ms | 0.05 ms | 0.00 ms | 1.29 s |
| report | 1 | 3.9 ms | 3.9 ms | 3.4 ms | 3.4 ms | 3.1 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.1 ms | 1.75 s | 1.6 ms | 1.75 s | 4.7 ms | 3.4 ms | 61.0 ms | 1.66 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 444 MB | 1400 MB | real5-letter-18 |
| Model server | 856 MB | 857 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (45)

- **real5-letter-01**: expected `2017-05-08 Warning Letter to Inghams Enterprises (NZ) Pty Limited.pdf`, got `Letter from COMMERCE.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `Letter from Coast to Coast Limited.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter - Vinn White.pdf` · trap: party Vinn White (addressee, FMCSA's Deputy Administrator) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-05**: expected `2025-09-23 Safety Recommendation Letter to Federal Aviation Administration.pdf`, got `2025-09-23 Letter to Jennifer L. Homendy.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-08**: expected `2026-07-24 No-Action Letter to Kraken Derivatives Exchange Inc.pdf`, got `2026-07-24 Letter for Kraken Derivatives Exchange Inc.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-09**: expected `2025-06-23 Approval Letter to Maryland Department of Health.pdf`, got `2025-06-23 Letter from Department of Health & Human Services.pdf` · review: TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-10**: expected `2026-07-21 Approval Letter to Delaware Health and Social Services.pdf`, got `2026-07-21 Letter from Department of Health and Human Services.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `Letter to Welsh Government.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Miller Research Payment (2) Information requested.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-14**: expected `2025-03-03 Letter to European Commission.pdf`, got `2025-03-03 Letter to Directorate General Financial Services.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `Letter to Michael McGrath.pdf` · trap: party Michael McGrath (Commissioner addressed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2020-06-21 Tax Invoice.pdf` · review: PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Invoice from KIA Kaha. Stressed OR Overwhelmed Call OR Text 1737 for Free Korero.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Letter to Secretary of Foreign Affairs and Trade.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document from Albert E. Schreiber & Co.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2025-12-03 Email.pdf` · trap: date 2025-12-03 (date of an earlier e-mail in the thread) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter to ElectronX DCM.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-01**: expected `2025-10-01 Notice of Intention to Remove Company From the Register for RS2022 Limited.pdf`, got `2025-10-01 Notice from Hamish Pryde.pdf` · trap: party Hamish Pryde (liquidator, a person) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Document - Kieran Jones.pdf` · trap: party Kieran Jones (joint liquidator, a person) · review: TYPE_MISSING, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document.pdf` · review: TYPE_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice for Stewart D. Aaron.pdf` · trap: party Stewart D. Aaron (magistrate judge the notice is about) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice for United States District Court for the Northern District of Texas.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Document from U.S. Department of the Treasury.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Document from U.S. Department of the Treasury.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-11**: expected `2025-12-04 Enforcement Release for Gracetown, Inc.pdf`, got `2018-04-06 Document - Gracetown, Inc.pdf` · trap: date 2018-04-06 (date Deripaska was designated) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document from Wateloo Hl.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document - Shipe HWHOLESALE CLOTHIERS.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from HOSOY.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-06**: expected `2025-09-09 Tax Invoice from Ian Davies Plant Limited.pdf`, got `Tax Invoice from Ian Davies Plant Limited Llwynfelish Farm Neath Road Crynant Neath.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Document from Debt Position and Activity Report.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document between Barbara LaCour and N1-310-90-001.pdf` · trap: party Barbara LaCour (officer who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `2026-09-08 Document - General Services Administration.pdf` · trap: date 2026-09-08 (date the status sheet is accurate as of); party General Services Administration (addressee in the form's TO line) · review: TYPE_UNSUPPORTED, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `General Counsel Memorandum to Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `General Counsel Memorandum to All Regional Directors.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Budget from Office of Management and Budget.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-05**: expected `2025-06-03 Mortgagee Letter from U.S. Department of Housing and Urban Development.pdf`, got `2025-06-03 Mortgagee Letter from Frank Cassidy.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- … and 5 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
