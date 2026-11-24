# InternBench: live run

- **Run:** 2026-11-23T16:52:25Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 57 documents · 57 completed · gold `77806e64ab`
- **Wall time:** 3323.91 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 19/53 (35.9%) |
| Document type | 43/57 (75.4%) |
| Date | 45/53 (84.9%) |
| Date role (when the reviewed date was chosen) | 21/44 (47.7%) |
| Parties | 36/57 (63.2%) |
| Relation word (when parties were named) | 31/48 (64.6%) |
| Party in the right role (when parties were named) | 20/35 (57.1%) |
| Ready / review routing | 10/35 (28.6%) |
| Description states nothing false | 55/57 (96.5%) |
| Description is specific | 35/57 (61.4%) |
| Description specificity | 59.7% (mean of 57) |
| Evidence recall (model's quotes) | 65.5% (mean of 56) |
| Digest recall (facts reaching the digest) | 83.9% (mean of 56) |
| Prompt recall (facts reaching the prompt sent) | 82.1% (mean of 56) |
| Sent to review | 93.0% of 57 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 36.4% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 73.7% | 57 |
| Review rate | 93.0% | 57 |
| Evidence recall | 65.5% | 56 |
| Total latency p50 | 58.21 s | 57 |
| Total latency p95 | 96.13 s | 57 |
| Generation latency p50 | 22.20 s | 57 |
| Generation latency p95 | 30.38 s | 57 |
| Generated tokens p50 | 91 | 57 |
| Generated tokens p95 | 120 | 57 |
| Prompt tokens p50 | 1222 | 57 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 53 |
| Trap date chosen | 3 of 56 |
| Forbidden party named | 4 of 57 |
| Spurious parties named | 17 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 2 of 146 (1.4%) |
| Right name sent to review anyway | 11 of 29 |
| Sent to review for an unsupported fact | 42 of 57 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 107.52 s | 107.52 s |
| form | 3 | 0/2 | 1/2 | 0/3 | 1/1 | 100.0% | 56.93 s | 57.48 s |
| invoice | 5 | 1/5 | 1/5 | 1/5 | 0/1 | 100.0% | 22.84 s | 44.46 s |
| letter | 22 | 11/22 | 20/22 | 20/22 | 3/13 | 90.9% | 64.83 s | 93.09 s |
| memo | 6 | 2/6 | 6/6 | 3/6 | 0/4 | 100.0% | 61.89 s | 75.10 s |
| minutes | 5 | 1/5 | 5/5 | 3/5 | 1/4 | 80.0% | 69.22 s | 78.28 s |
| notice | 11 | 4/9 | 9/9 | 7/11 | 3/10 | 90.9% | 34.33 s | 77.10 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 24.29 s | 24.29 s |
| report | 1 | – | – | 1/1 | 1/1 | 100.0% | 44.29 s | 44.29 s |
| statement | 2 | 0/2 | 2/2 | 1/2 | 1/1 | 100.0% | 68.79 s | 96.13 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 1/6 | – | 100.0% | 22.84 s | 36.82 s |
| native | 40 | 17/38 | 38/38 | 30/40 | 4/27 | 95.0% | 61.83 s | 93.09 s |
| office | 4 | 0/2 | 2/2 | 1/4 | 3/3 | 100.0% | 44.29 s | 96.13 s |
| scan | 7 | 2/7 | 5/7 | 4/7 | 3/5 | 71.4% | 68.33 s | 82.44 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 3/16 | 9/16 | 8/19 | 5/10 | 94.7% | 31.42 s | 54.97 s |
| 2-4 | 16 | 6/16 | 15/16 | 10/16 | 4/8 | 81.2% | 64.57 s | 107.52 s |
| 5-9 | 11 | 6/10 | 10/10 | 10/11 | 1/9 | 100.0% | 64.83 s | 82.61 s |
| 10-24 | 9 | 4/9 | 9/9 | 7/9 | 0/7 | 100.0% | 73.14 s | 93.09 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 71.59 s | 71.59 s |
| 50-99 | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 152.39 s | 152.39 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 4/11 | 11/11 | 8/11 | 0/8 | 0 of 11 | 73.14 s | 152.39 s | 24.28 s | 27.70 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 3/6 | 6/6 | 5/6 | 0 |
| advisory_committee | 2 | 0/2 | 2/2 | 1/2 | 0 |
| agency_guidance | 4 | 2/4 | 4/4 | 2/4 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_letter | 3 | 3/3 | 3/3 | 3/3 | 0 |
| as_of_date | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attachments | 5 | 3/5 | 5/5 | 5/5 | 0 |
| attachments_listed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attendance_list | 2 | 0/2 | 2/2 | 1/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 0/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| board_minutes | 2 | 1/2 | 2/2 | 2/2 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 1/1 | 0 |
| born_digital | 40 | 17/38 | 38/38 | 30/40 | 4 |
| care_of_counsel | 2 | 1/2 | 2/2 | 2/2 | 0 |
| charts | 1 | 0/1 | 1/1 | 1/1 | 0 |
| circular_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 36 | 16/36 | 33/36 | 27/36 | 1 |
| court_notice | 5 | 1/3 | 3/3 | 2/5 | 3 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 0/2 | 1 |
| customer_first | 5 | 1/5 | 1/5 | 1/5 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 2/4 | 4/4 | 3/4 | 1 |
| deadline_in_body | 5 | 2/3 | 3/3 | 2/5 | 4 |
| determination | 1 | 0/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 1/3 | 2 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email_printed_to_pdf | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email_thread | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enforcement_release | 3 | 2/3 | 3/3 | 3/3 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eu_body | 2 | 1/2 | 2/2 | 2/2 | 0 |
| financial_report | 1 | 0/1 | 1/1 | 1/1 | 0 |
| foi_release | 2 | 1/2 | 2/2 | 1/2 | 0 |
| foi_response | 5 | 2/5 | 5/5 | 4/5 | 0 |
| foreign_company | 1 | 0/1 | 1/1 | 0/1 | 0 |
| form | 3 | 0/2 | 1/2 | 0/3 | 1 |
| former_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 2/4 | 4/4 | 3/4 | 1 |
| government_form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_dates | 2 | 0/2 | 1/2 | 0/2 | 1 |
| historic | 6 | 0/6 | 0/6 | 1/6 | 0 |
| historical | 2 | 0/2 | 1/2 | 0/2 | 1 |
| image | 6 | 0/6 | 0/6 | 1/6 | 0 |
| invoice | 5 | 1/5 | 1/5 | 1/5 | 0 |
| invoices_attached | 2 | 0/2 | 2/2 | 2/2 | 0 |
| issuer_not_named | 2 | 0/2 | 2/2 | 0/2 | 0 |
| joint_meeting | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter | 22 | 11/22 | 20/22 | 20/22 | 0 |
| letter_from_company | 3 | 0/3 | 2/3 | 2/3 | 0 |
| letterhead | 3 | 0/3 | 2/3 | 1/3 | 0 |
| liquidation | 3 | 2/3 | 3/3 | 2/3 | 1 |
| long_document | 5 | 3/5 | 5/5 | 4/5 | 0 |
| many_dates | 8 | 1/6 | 6/6 | 7/8 | 2 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo | 6 | 2/6 | 6/6 | 3/6 | 0 |
| memorandum | 4 | 0/4 | 4/4 | 1/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 2/3 | 0/3 | 2 |
| minutes | 5 | 1/5 | 5/5 | 3/5 | 1 |
| modern | 46 | 19/42 | 42/42 | 34/46 | 5 |
| month_only_date | 1 | – | – | 1/1 | 1 |
| multiple_letters | 2 | 1/2 | 2/2 | 2/2 | 0 |
| named_officials | 4 | 0/2 | 2/2 | 1/4 | 3 |
| named_person_at_org | 12 | 6/12 | 11/12 | 11/12 | 0 |
| names_removed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| new_zealand | 10 | 4/10 | 10/10 | 8/10 | 1 |
| newsletter | 1 | – | – | 1/1 | 1 |
| no_action_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| no_document_date | 3 | – | – | 0/3 | 3 |
| no_inside_address | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 1/4 | 1 |
| northern_ireland | 2 | – | – | 1/2 | 1 |
| notice | 11 | 4/9 | 9/9 | 7/11 | 4 |
| ocr_text_layer | 7 | 2/7 | 5/7 | 4/7 | 1 |
| office | 4 | 0/2 | 2/2 | 1/4 | 2 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 2/6 | 6/6 | 4/8 | 4 |
| publication_timestamp | 4 | 2/4 | 4/4 | 3/4 | 1 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 5/14 | 14/14 | 10/14 | 0 |
| regulator | 9 | 6/9 | 9/9 | 9/9 | 0 |
| reissued | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_date_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| remittance_slip | 1 | 1/1 | 1/1 | 1/1 | 0 |
| reply_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 1/1 | 1 |
| request_letter | 2 | 0/2 | 2/2 | 1/2 | 0 |
| safety_recommendation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| scanned | 7 | 2/7 | 5/7 | 4/7 | 1 |
| sender_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 1/2 | 2/2 | 2/2 | 0 |
| settlement | 2 | 1/2 | 2/2 | 2/2 | 0 |
| short | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| staff_names | 4 | 1/4 | 4/4 | 3/4 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 1/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 2/2 | 1/2 | 0 |
| tax_invoice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| trading_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| type_not_printed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| uk | 8 | 3/6 | 6/6 | 4/8 | 2 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 28 | 11/26 | 24/26 | 19/28 | 4 |
| vat | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wales | 6 | 3/6 | 6/6 | 3/6 | 1 |
| warning_letter | 3 | 2/3 | 3/3 | 3/3 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 187 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 20 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 57 | 58.21 s | 96.13 s | 152.39 s |
| `extraction_wall_ms` | 57 | 14.4 ms | 5.59 s | 57.23 s |
| `worker_total_ms` | 57 | 13.8 ms | 5.59 s | 57.22 s |
| `worker_snapshot_ms` | 57 | 0.25 ms | 1.1 ms | 2.6 ms |
| `worker_parse_ms` | 57 | 9.1 ms | 59.8 ms | 1.47 s |
| `worker_analysis_ms` | 57 | 0.93 ms | 7.8 ms | 31.4 ms |
| `worker_render_ms` | 57 | 0.00 ms | 655.4 ms | 23.71 s |
| `worker_image_decode_ms` | 57 | 0.00 ms | 48.4 ms | 84.2 ms |
| `worker_ocr_ms` | 57 | 0.00 ms | 10.24 s | 113.18 s |
| `worker_ocr_encode_ms` | 57 | 0.00 ms | 788.5 ms | 7.34 s |
| `worker_ocr_engine_ms` | 57 | 0.00 ms | 9.21 s | 105.59 s |
| `worker_vision_ms` | 57 | 0.00 ms | 18.5 ms | 27.5 ms |
| `analyze_wall_ms` | 57 | 58.19 s | 95.15 s | 107.51 s |
| `distill_ms` | 57 | 3.0 ms | 27.3 ms | 94.3 ms |
| `prompt_ms` | 57 | 0.03 ms | 0.06 ms | 0.09 ms |
| `inference_ms` | 57 | 58.18 s | 95.00 s | 107.50 s |
| `prefill_ms` | 57 | 33.35 s | 72.86 s | 78.30 s |
| `generation_ms` | 57 | 22.20 s | 30.38 s | 31.40 s |
| `validation_ms` | 57 | 6.6 ms | 22.7 ms | 53.5 ms |
| `naming_ms` | 57 | 0.39 ms | 3.0 ms | 4.7 ms |
| `index_ms` | 57 | 2.8 ms | 24.7 ms | 87.3 ms |
| `retrieval_ms` | 57 | 0.05 ms | 2.3 ms | 5.6 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 57 | 36.7 tok/s | 37.5 tok/s | 37.7 tok/s |
| `generation_tok_per_s` | 57 | 4.1 tok/s | 4.3 tok/s | 4.3 tok/s |
| `prompt_tokens` | 57 | 1222 | 2678 | 2878 |
| `cached_tokens` | 57 | 477 | 477 | 477 |
| `generated_tokens` | 57 | 91 | 120 | 128 |
| `estimated_prompt_tokens` | 57 | 1904 | 3281 | 3354 |
| `model_requests` | 57 | 1 | 1 | 1 |
| `redistillations` | 57 | 0 | 0 | 0 |
| `source_characters` | 57 | 5735 | 49993 | 104046 |
| `digest_characters` | 57 | 3997 | 8250 | 8964 |
| `prompt_characters` | 57 | 6051 | 10305 | 11013 |
| `worker_ocr_pages` | 57 | 0 | 5 | 77 |
| `worker_ocr_passes` | 57 | 0 | 5 | 77 |
| `worker_orientation_passes` | 57 | 0 | 5 | 77 |
| `worker_rendered_pixels` | 57 | 0 | 43516740 | 667903296 |
| `index_units` | 57 | 48 | 371 | 3200 |
| `context_units` | 57 | 25 | 65 | 114 |
| `context_tokens` | 57 | 1317 | 2690 | 2767 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 20 | 3.9 ms | 19.5 ms | 3.5 ms | 18.8 ms | 16.0 ms | 1.8 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 12.3 ms | 56.9 ms | 11.7 ms | 55.5 ms | 45.3 ms | 7.0 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 1.83 s | 1.83 s | 1.83 s | 1.83 s | 4.9 ms | 4.0 ms | 61.6 ms | 1.74 s |
| ocr | 15 | 2.82 s | 57.23 s | 2.82 s | 57.22 s | 1.47 s | 31.4 ms | 23.71 s | 113.18 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 4.5 ms | 2.41 s | 4.1 ms | 2.41 s | 12.0 ms | 1.1 ms | 99.8 ms | 2.28 s |
| 2-4 | 16 | 11.6 ms | 4.96 s | 11.1 ms | 4.96 s | 1.47 s | 4.0 ms | 629.4 ms | 6.32 s |
| 5-9 | 11 | 12.0 ms | 21.1 ms | 11.5 ms | 20.4 ms | 16.9 ms | 3.7 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 39.2 ms | 19.37 s | 37.4 ms | 19.37 s | 48.3 ms | 7.8 ms | 2.38 s | 36.45 s |
| 25-49 | 1 | 5.59 s | 5.59 s | 5.59 s | 5.59 s | 59.8 ms | 20.2 ms | 655.4 ms | 10.24 s |
| 50-99 | 1 | 57.23 s | 57.23 s | 57.22 s | 57.22 s | 154.5 ms | 31.4 ms | 23.71 s | 113.18 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 9.8 ms | 9.8 ms | 9.3 ms | 9.3 ms | 6.7 ms | 1.1 ms | 0.00 ms | 0.00 ms |
| form | 3 | 2.82 s | 3.47 s | 2.82 s | 3.47 s | 11.8 ms | 0.93 ms | 253.7 ms | 4.44 s |
| invoice | 5 | 1.13 s | 1.35 s | 1.13 s | 1.35 s | 12.0 ms | 0.54 ms | 0.00 ms | 1.22 s |
| letter | 22 | 21.1 ms | 19.37 s | 20.4 ms | 19.37 s | 154.5 ms | 6.9 ms | 2.38 s | 36.45 s |
| memo | 6 | 11.6 ms | 5.59 s | 11.1 ms | 5.59 s | 59.8 ms | 20.2 ms | 655.4 ms | 10.24 s |
| minutes | 5 | 14.4 ms | 39.2 ms | 13.8 ms | 37.4 ms | 28.4 ms | 7.8 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 3.6 ms | 19.5 ms | 3.3 ms | 18.8 ms | 16.0 ms | 1.8 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.35 s | 1.35 s | 1.35 s | 1.35 s | 9.3 ms | 0.05 ms | 0.00 ms | 1.32 s |
| report | 1 | 3.9 ms | 3.9 ms | 3.5 ms | 3.5 ms | 3.2 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 2.0 ms | 1.83 s | 1.4 ms | 1.83 s | 4.9 ms | 4.0 ms | 61.6 ms | 1.74 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 373 MB | 1355 MB | real5-letter-18 |
| Model server | 6084 MB | 6085 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (34)

- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `2015-12-16 Letter from Coast to Coast Limited.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter from Federal Motor Carrier Safety Administration.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-07**: expected `2026-08-10 Request for No-Action Relief from Eurex Clearing AG.pdf`, got `2026-08-10 Letter to Division of Clearing and Risk.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-14**: expected `2025-03-03 Letter to European Commission.pdf`, got `2025-03-03 Letter to Directorate General Financial Services.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `2026-04-20 Concerns on Israel's registration requirements imposed on NGOs for European Commission.pdf` · review: LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2021-07-23 Tax Invoice from He Pou a Rangi.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Letter to Ministry for Regulation.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Document.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document - Albert E. Schreiber & Co.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2026-01-19 Email - NPTCBC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter from Commodity Futures Trading Commission.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Notice of Intention to Remove Companies for Kieran Jones.pdf` · trap: party Kieran Jones (joint liquidator, a person) · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document from Environmental Protection Authority.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice from Edward Friedland.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice from United States District Court Northern District of Texas.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Enforcement Release from Interactive Brokers LLC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document between D. CREHANGE and Shipe HWHOLESALE CLOTHIERS.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from Soyg New York 189.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document - National Archives and Records Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `Document between U.S. Department of Treasury and Raymond A. O'BRIEN.pdf` · trap: party Raymond A. O'Brien (person to confer with) · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `2025-05-16 General Counsel Memorandum to All Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Memorandum for Heads of Executive Departments and Agencies.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies from Executive Office of the President.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-08**: expected `2025-05-29 Board Meeting Minutes for Federal Retirement Thrift Investment Board.pdf`, got `2025-05-29 Document from Federal Retirement Thrift Investment Board.pdf` · review: TYPE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-09**: expected `2024-10-29 Meeting Minutes for Visiting Committee on Advanced Technology.pdf`, got `2024-10-29 Minutes for Visiting Committee Members.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-10**: expected `2024-02-14 Meeting Minutes for Visiting Committee on Advanced Technology.pdf`, got `2024-02-14 Minutes from VISITING COMMITTEE ON ADVANCED TECHNOLOGY (VCAT or Committee).pdf` · review: LOW_CONFIDENCE
- **real5-office-01**: expected `2018-06-14 Minutes.docx`, got `2018-06-14 Minutes for Phil Brown.docx` · trap: party Phil Brown (chair of the review, a person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-office-04**: expected `2023-09-30 Allocation, Spend and Commitment Statement.xlsx`, got `2023-09-30 Document from RD&S.xlsx` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
