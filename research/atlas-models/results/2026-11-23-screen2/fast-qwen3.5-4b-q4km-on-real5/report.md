# InternBench: live run

- **Run:** 2026-11-23T16:45:40Z · commit `70ce2164a1`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 57 documents · 57 completed · gold `77806e64ab`
- **Wall time:** 3329.53 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 13/53 (24.5%) |
| Document type | 44/57 (77.2%) |
| Date | 45/53 (84.9%) |
| Date role (when the reviewed date was chosen) | 20/43 (46.5%) |
| Parties | 31/57 (54.4%) |
| Relation word (when parties were named) | 36/52 (69.2%) |
| Party in the right role (when parties were named) | 19/42 (45.2%) |
| Ready / review routing | 8/35 (22.9%) |
| Description states nothing false | 57/57 (100.0%) |
| Description is specific | 36/57 (63.2%) |
| Description specificity | 62.6% (mean of 57) |
| Evidence recall (model's quotes) | 68.2% (mean of 56) |
| Digest recall (facts reaching the digest) | 83.9% (mean of 56) |
| Prompt recall (facts reaching the prompt sent) | 82.1% (mean of 56) |
| Sent to review | 96.5% of 57 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 45.5% | 11 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 80.7% | 57 |
| Review rate | 96.5% | 57 |
| Evidence recall | 68.2% | 56 |
| Total latency p50 | 56.46 s | 57 |
| Total latency p95 | 113.37 s | 57 |
| Generation latency p50 | 12.26 s | 57 |
| Generation latency p95 | 15.02 s | 57 |
| Generated tokens p50 | 95 | 57 |
| Generated tokens p95 | 116 | 57 |
| Prompt tokens p50 | 1222 | 57 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 53 |
| Trap date chosen | 4 of 56 |
| Forbidden party named | 8 of 57 |
| Spurious parties named | 26 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 168 (0.0%) |
| Right name sent to review anyway | 10 of 29 |
| Sent to review for an unsupported fact | 46 of 57 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 118.41 s | 118.41 s |
| form | 3 | 0/2 | 2/2 | 0/3 | 1/1 | 100.0% | 61.21 s | 62.56 s |
| invoice | 5 | 0/5 | 1/5 | 0/5 | 0/1 | 100.0% | 24.29 s | 38.41 s |
| letter | 22 | 6/22 | 20/22 | 13/22 | 2/13 | 95.5% | 64.20 s | 97.40 s |
| memo | 6 | 2/6 | 6/6 | 3/6 | 0/4 | 100.0% | 60.58 s | 70.90 s |
| minutes | 5 | 2/5 | 5/5 | 4/5 | 1/4 | 80.0% | 73.29 s | 90.57 s |
| notice | 11 | 3/9 | 9/9 | 8/11 | 2/10 | 100.0% | 31.21 s | 75.08 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 15.44 s | 15.44 s |
| report | 1 | – | – | 1/1 | 1/1 | 100.0% | 45.45 s | 45.45 s |
| statement | 2 | 0/2 | 2/2 | 2/2 | 1/1 | 100.0% | 75.37 s | 113.37 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 6 | 0/6 | 0/6 | 1/6 | – | 100.0% | 23.39 s | 29.12 s |
| native | 40 | 12/38 | 37/38 | 25/40 | 3/27 | 97.5% | 56.46 s | 97.40 s |
| office | 4 | 0/2 | 2/2 | 2/4 | 3/3 | 100.0% | 45.45 s | 113.37 s |
| scan | 7 | 1/7 | 6/7 | 3/7 | 2/5 | 85.7% | 70.90 s | 88.60 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 2/16 | 9/16 | 8/19 | 4/10 | 100.0% | 26.08 s | 56.46 s |
| 2-4 | 16 | 3/16 | 15/16 | 6/16 | 3/8 | 87.5% | 68.13 s | 118.41 s |
| 5-9 | 11 | 3/10 | 10/10 | 9/11 | 1/9 | 100.0% | 64.20 s | 90.57 s |
| 10-24 | 9 | 5/9 | 9/9 | 8/9 | 0/7 | 100.0% | 76.03 s | 96.86 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 70.90 s | 70.90 s |
| 50-99 | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 173.42 s | 173.42 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 11 | 5/11 | 11/11 | 8/11 | 0/8 | 0 of 11 | 76.03 s | 173.42 s | 12.68 s | 14.44 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_person | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_redacted | 6 | 1/6 | 6/6 | 1/6 | 4 |
| advisory_committee | 2 | 1/2 | 2/2 | 2/2 | 0 |
| agency_guidance | 4 | 2/4 | 4/4 | 2/4 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 0/1 | 0 |
| approval_letter | 3 | 2/3 | 3/3 | 3/3 | 0 |
| as_of_date | 2 | 0/2 | 2/2 | 2/2 | 0 |
| attachments | 5 | 2/5 | 5/5 | 3/5 | 1 |
| attachments_listed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| attendance_list | 2 | 1/2 | 2/2 | 2/2 | 0 |
| attendee_list | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 4 | 0/4 | 0/4 | 0/4 | 0 |
| blank_template | 1 | – | – | 0/1 | 0 |
| board_minutes | 2 | 1/2 | 2/2 | 2/2 | 0 |
| body_recounts_events | 1 | 0/1 | 1/1 | 1/1 | 0 |
| born_digital | 40 | 12/38 | 37/38 | 25/40 | 9 |
| care_of_counsel | 2 | 0/2 | 2/2 | 1/2 | 1 |
| charts | 1 | 0/1 | 1/1 | 1/1 | 0 |
| circular_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 36 | 10/36 | 33/36 | 20/36 | 6 |
| court_notice | 5 | 1/3 | 3/3 | 3/5 | 3 |
| cover_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cover_sheet | 2 | 0/2 | 2/2 | 0/2 | 0 |
| customer_first | 5 | 0/5 | 1/5 | 0/5 | 1 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_partly_illegible | 1 | 0/1 | 0/1 | 1/1 | 0 |
| dated_line_at_end | 4 | 2/4 | 4/4 | 3/4 | 0 |
| deadline_in_body | 5 | 2/3 | 3/3 | 3/5 | 3 |
| determination | 1 | 0/1 | 1/1 | 1/1 | 0 |
| digital_signature_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docx | 3 | 0/1 | 1/1 | 1/3 | 2 |
| due_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| effective_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_printed_to_pdf | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email_thread | 1 | 0/1 | 0/1 | 0/1 | 1 |
| enforcement_release | 3 | 1/3 | 3/3 | 3/3 | 0 |
| engraved_letterhead | 1 | 0/1 | 0/1 | 0/1 | 0 |
| eu_agency | 1 | 0/1 | 1/1 | 0/1 | 0 |
| eu_body | 2 | 1/2 | 2/2 | 1/2 | 0 |
| financial_report | 1 | 0/1 | 1/1 | 1/1 | 0 |
| foi_release | 2 | 0/2 | 1/2 | 0/2 | 2 |
| foi_response | 5 | 0/5 | 5/5 | 0/5 | 4 |
| foreign_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 3 | 0/2 | 2/2 | 0/3 | 0 |
| former_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| gazette | 4 | 2/4 | 4/4 | 3/4 | 0 |
| government_form | 2 | 0/2 | 2/2 | 0/2 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_dates | 2 | 0/2 | 2/2 | 0/2 | 0 |
| historic | 6 | 0/6 | 0/6 | 1/6 | 0 |
| historical | 2 | 0/2 | 2/2 | 0/2 | 0 |
| image | 6 | 0/6 | 0/6 | 1/6 | 0 |
| invoice | 5 | 0/5 | 1/5 | 0/5 | 1 |
| invoices_attached | 2 | 0/2 | 2/2 | 0/2 | 1 |
| issuer_not_named | 2 | 0/2 | 2/2 | 0/2 | 0 |
| joint_meeting | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter | 22 | 6/22 | 20/22 | 13/22 | 5 |
| letter_from_company | 3 | 1/3 | 2/3 | 3/3 | 0 |
| letterhead | 3 | 1/3 | 2/3 | 2/3 | 0 |
| liquidation | 3 | 2/3 | 3/3 | 2/3 | 0 |
| long_document | 5 | 2/5 | 5/5 | 3/5 | 1 |
| many_dates | 8 | 1/6 | 6/6 | 5/8 | 3 |
| many_parties | 1 | 0/1 | 1/1 | 0/1 | 0 |
| memo | 6 | 2/6 | 6/6 | 3/6 | 0 |
| memorandum | 4 | 0/4 | 4/4 | 1/4 | 0 |
| metadata_date_trap | 3 | 0/3 | 3/3 | 0/3 | 1 |
| minutes | 5 | 2/5 | 5/5 | 4/5 | 1 |
| modern | 46 | 13/42 | 41/42 | 28/46 | 11 |
| month_only_date | 1 | – | – | 1/1 | 1 |
| multiple_letters | 2 | 0/2 | 2/2 | 2/2 | 0 |
| named_officials | 4 | 0/2 | 2/2 | 2/4 | 3 |
| named_person_at_org | 12 | 3/12 | 11/12 | 8/12 | 1 |
| names_removed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| new_zealand | 10 | 3/10 | 10/10 | 5/10 | 3 |
| newsletter | 1 | – | – | 1/1 | 1 |
| no_action_letter | 2 | 1/2 | 2/2 | 1/2 | 0 |
| no_document_date | 3 | – | – | 1/3 | 3 |
| no_inside_address | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_issuer_printed | 4 | 0/3 | 3/3 | 2/4 | 1 |
| northern_ireland | 2 | – | – | 1/2 | 1 |
| notice | 11 | 3/9 | 9/9 | 8/11 | 3 |
| ocr_text_layer | 7 | 1/7 | 6/7 | 3/7 | 1 |
| office | 4 | 0/2 | 2/2 | 2/4 | 2 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| partly_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholders | 1 | – | – | 0/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 0 |
| public_notice | 8 | 2/6 | 6/6 | 5/8 | 3 |
| publication_timestamp | 4 | 2/4 | 4/4 | 3/4 | 0 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| reference_number | 14 | 2/14 | 14/14 | 4/14 | 4 |
| regulator | 9 | 3/9 | 9/9 | 7/9 | 1 |
| reissued | 1 | 0/1 | 1/1 | 0/1 | 1 |
| relative_date_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| remittance_slip | 1 | 0/1 | 1/1 | 0/1 | 1 |
| reply_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report | 1 | – | – | 1/1 | 1 |
| request_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| safety_recommendation | 2 | 0/2 | 2/2 | 2/2 | 0 |
| scanned | 7 | 1/7 | 6/7 | 3/7 | 1 |
| sender_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 2 | 0/2 | 2/2 | 2/2 | 0 |
| settlement | 2 | 0/2 | 2/2 | 2/2 | 0 |
| short | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_scan | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 4 | 2/4 | 4/4 | 4/4 | 0 |
| stamped_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| statement | 2 | 0/2 | 2/2 | 2/2 | 0 |
| struck_through_text | 1 | 0/1 | 0/1 | 0/1 | 0 |
| tables | 2 | 0/2 | 2/2 | 1/2 | 0 |
| tax_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| trading_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_addressees | 1 | 0/1 | 1/1 | 0/1 | 0 |
| type_not_printed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| uk | 8 | 0/6 | 5/6 | 2/8 | 6 |
| us_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| us_federal | 28 | 8/26 | 25/26 | 20/28 | 3 |
| vat | 1 | 0/1 | 1/1 | 0/1 | 1 |
| wales | 6 | 0/6 | 5/6 | 1/6 | 5 |
| warning_letter | 3 | 1/3 | 3/3 | 2/3 | 1 |
| xlsx | 1 | 0/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 187 · layout 102 · ocr 44 · ocr_regions 68
- **Documents by route class:** fast 20 · layout 21 · ocr 15 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 57 | 56.46 s | 113.37 s | 173.42 s |
| `extraction_wall_ms` | 57 | 21.4 ms | 7.67 s | 66.77 s |
| `worker_total_ms` | 57 | 20.6 ms | 7.66 s | 66.75 s |
| `worker_snapshot_ms` | 57 | 0.33 ms | 1.8 ms | 3.1 ms |
| `worker_parse_ms` | 57 | 12.0 ms | 79.4 ms | 219.6 ms |
| `worker_analysis_ms` | 57 | 1.1 ms | 9.7 ms | 38.0 ms |
| `worker_render_ms` | 57 | 0.00 ms | 905.7 ms | 29.56 s |
| `worker_image_decode_ms` | 57 | 0.00 ms | 59.1 ms | 113.6 ms |
| `worker_ocr_ms` | 57 | 0.00 ms | 14.27 s | 131.14 s |
| `worker_ocr_encode_ms` | 57 | 0.00 ms | 924.8 ms | 8.59 s |
| `worker_ocr_engine_ms` | 57 | 0.00 ms | 12.98 s | 122.16 s |
| `worker_vision_ms` | 57 | 0.00 ms | 21.9 ms | 31.6 ms |
| `analyze_wall_ms` | 57 | 56.46 s | 106.65 s | 118.40 s |
| `distill_ms` | 57 | 4.1 ms | 33.8 ms | 120.6 ms |
| `prompt_ms` | 57 | 0.04 ms | 0.07 ms | 0.10 ms |
| `inference_ms` | 57 | 56.45 s | 106.45 s | 118.37 s |
| `prefill_ms` | 57 | 42.48 s | 94.61 s | 102.53 s |
| `generation_ms` | 57 | 12.26 s | 15.02 s | 16.92 s |
| `validation_ms` | 57 | 9.0 ms | 27.2 ms | 67.4 ms |
| `naming_ms` | 57 | 0.46 ms | 3.6 ms | 5.2 ms |
| `index_ms` | 57 | 3.4 ms | 31.3 ms | 113.1 ms |
| `retrieval_ms` | 57 | 0.06 ms | 2.7 ms | 6.3 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 57 | 28.5 tok/s | 28.9 tok/s | 29.1 tok/s |
| `generation_tok_per_s` | 57 | 7.7 tok/s | 8.4 tok/s | 8.5 tok/s |
| `prompt_tokens` | 57 | 1222 | 2678 | 2878 |
| `cached_tokens` | 57 | 477 | 477 | 477 |
| `generated_tokens` | 57 | 95 | 116 | 126 |
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
| fast | 20 | 5.2 ms | 26.3 ms | 4.5 ms | 25.5 ms | 22.2 ms | 2.1 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 17.9 ms | 73.3 ms | 17.0 ms | 71.0 ms | 59.3 ms | 9.1 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 2.69 s | 2.69 s | 2.69 s | 2.69 s | 6.3 ms | 4.2 ms | 78.9 ms | 2.56 s |
| ocr | 15 | 3.78 s | 66.77 s | 3.78 s | 66.75 s | 219.6 ms | 38.0 ms | 29.56 s | 131.14 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 5.5 ms | 3.38 s | 5.1 ms | 3.38 s | 22.6 ms | 1.4 ms | 143.1 ms | 3.19 s |
| 2-4 | 16 | 14.8 ms | 5.67 s | 14.2 ms | 5.67 s | 219.6 ms | 4.2 ms | 761.8 ms | 9.04 s |
| 5-9 | 11 | 20.5 ms | 28.4 ms | 19.9 ms | 27.4 ms | 23.2 ms | 4.6 ms | 0.00 ms | 0.00 ms |
| 10-24 | 9 | 52.3 ms | 22.46 s | 50.0 ms | 22.46 s | 65.6 ms | 9.7 ms | 3.34 s | 42.48 s |
| 25-49 | 1 | 7.67 s | 7.67 s | 7.66 s | 7.66 s | 79.4 ms | 21.9 ms | 905.7 ms | 14.27 s |
| 50-99 | 1 | 66.77 s | 66.77 s | 66.75 s | 66.75 s | 207.0 ms | 38.0 ms | 29.56 s | 131.14 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| email | 1 | 12.4 ms | 12.4 ms | 11.8 ms | 11.8 ms | 8.7 ms | 1.3 ms | 0.00 ms | 0.00 ms |
| form | 3 | 3.78 s | 4.75 s | 3.78 s | 4.75 s | 16.1 ms | 1.3 ms | 397.0 ms | 5.80 s |
| invoice | 5 | 1.29 s | 1.71 s | 1.29 s | 1.71 s | 22.6 ms | 0.63 ms | 0.00 ms | 1.54 s |
| letter | 22 | 28.9 ms | 22.46 s | 27.9 ms | 22.46 s | 207.0 ms | 8.4 ms | 3.34 s | 42.48 s |
| memo | 6 | 14.8 ms | 7.67 s | 14.2 ms | 7.66 s | 79.4 ms | 21.9 ms | 905.7 ms | 14.27 s |
| minutes | 5 | 20.5 ms | 52.3 ms | 19.9 ms | 50.0 ms | 38.7 ms | 9.7 ms | 0.00 ms | 0.00 ms |
| notice | 11 | 4.6 ms | 26.3 ms | 4.2 ms | 25.5 ms | 22.2 ms | 2.1 ms | 0.00 ms | 0.00 ms |
| receipt | 1 | 1.33 s | 1.33 s | 1.33 s | 1.33 s | 17.0 ms | 0.06 ms | 0.00 ms | 1.28 s |
| report | 1 | 5.4 ms | 5.4 ms | 4.5 ms | 4.5 ms | 4.2 ms | 0.00 ms | 0.00 ms | 0.00 ms |
| statement | 2 | 3.0 ms | 2.69 s | 2.0 ms | 2.69 s | 6.3 ms | 4.2 ms | 78.9 ms | 2.56 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 433 MB | 1361 MB | real5-letter-18 |
| Model server | 3267 MB | 3267 MB | real5-office-04 |
| Benchmark (engine) | 24 MB | 24 MB | real5-office-04 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (40)

- **real5-letter-01**: expected `2017-05-08 Warning Letter to Inghams Enterprises (NZ) Pty Limited.pdf`, got `2017-05-08 Warning Letter to Mr Brannan.pdf` · trap: party Mr Brannan (addressee, a person at the company or its counsel) · review: LOW_CONFIDENCE
- **real5-letter-03**: expected `2015-12-16 Warning Letter to Coast to Coast Limited.pdf`, got `2015-12-16 Warning Letter from Coast to Coast Limited.pdf` · review: LOW_CONFIDENCE
- **real5-letter-04**: expected `2024-08-28 Safety Recommendation Letter to Federal Motor Carrier Safety Administration.pdf`, got `2024-08-28 Letter to Mr. Vinn White Deputy Administrator Federal Motor Carrier Safety Administration.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-05**: expected `2025-09-23 Safety Recommendation Letter to Federal Aviation Administration.pdf`, got `2025-09-23 Letter to The Honorable Bryan Bedford Administrator Federal Aviation Administration.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-06**: expected `2026-09-17 No-Action Letter to Eurex Clearing AG.pdf`, got `2026-09-17 No-Action Letter to Messrs. Graulich and Seinsheimer.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-10**: expected `2026-07-21 Approval Letter to Delaware Health and Social Services.pdf`, got `2026-07-21 Letter to Delaware Health and Social Services, Department of Health and Social Services.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-12**: expected `2024-02-14 Freedom of Information Response from Welsh Government.pdf`, got `2024-02-14 Letter from ATISN 20183 - FOI Request Acquisition of Companies House, Crown Way, Cardiff.pdf` · trap: party Companies House (building the request is about) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-13**: expected `2026-02-25 Freedom of Information Response from Welsh Government.pdf`, got `2026-02-25 Letter to Miller Research UK Ltd.pdf` · trap: party Miller Research UK Ltd (company the request is about) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-14**: expected `2025-03-03 Letter to European Commission.pdf`, got `2025-03-03 Letter to Mr Berrigan.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-15**: expected `2026-04-20 Letter to European Commission.pdf`, got `2026-04-20 Letter to Commissioner McGrath.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real5-letter-17**: expected `2017-02-16 Letter to Island Air Charters.pdf`, got `Letter to Island Air Charters.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real5-letter-18**: expected `2021-07-23 Official Information Act Response from Climate Change Commission.pdf`, got `2021-07-23 Tax Invoice from Ministry for the Environment.pdf` · trap: party Ministry for the Environment (ministry named in the letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real5-letter-19**: expected `2024-11-15 Official Information Act Response from Ministry for Regulation.pdf`, got `2024-11-15 Letter from Naumi Hotels NZ Pty Ltd.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-20**: expected `2022-08-26 Official Information Act Response from Ministry of Foreign Affairs and Trade.pdf`, got `2022-08-26 Letter to Tatou NZ Ltd.pdf` · trap: party Tatou NZ Ltd (company the request is about) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-21**: expected `1883-09-20 Letter from Albert E. Schreiber & Co.jpg`, got `Document from Albert E. Schreiber & Co.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-letter-22**: expected `2026-01-19 Email from Neath Port Talbot Council.pdf`, got `2025-11-27 Email to Tim Gweithredu Awdurdod Goruchwylio Supervisory Authority Implementation.pdf` · trap: date 2025-11-27 (date of an earlier e-mail in the thread) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-letter-23**: expected `2026-08-12 Request for No-Action Relief from Electron Exchange DCM, LLC.pdf`, got `2026-08-12 Letter from Commodity Futures Trading Commission.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-02**: expected `2026-05-07 Notice of Intention to Remove Companies From the Register from Khov Jones Limited.pdf`, got `2026-05-07 Document from New Zealand Gazette.pdf` · review: TYPE_UNSUPPORTED, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-03**: expected `2026-11-03 Determination of a New Organism from Environmental Protection Authority.pdf`, got `2026-11-03 Document from Environmental Protection Authority.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-05**: expected `2025-03-03 Public Notice from United States District Court for the Southern District of New York.pdf`, got `2025-03-03 Public Notice for Edward Friedland.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-06**: expected `2025-04-16 Public Notice from United States District Court for the Northern District of Texas.pdf`, got `2025-04-16 Public Notice from United States District Court Northern District of Texas.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-09**: expected `2025-07-15 Enforcement Release for Interactive Brokers LLC.pdf`, got `2025-07-15 Enforcement Release from Interactive Brokers LLC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-notice-10**: expected `2025-09-22 Enforcement Release for ShapeShift AG.pdf`, got `2025-09-22 Enforcement Release from ShapeShift AG.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-01**: expected `1896-10-20 Bill from Sternberg Brothers.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-02**: expected `1887-11-23 Bill from S. F. & A. Rothschild.jpg`, got `Document from Teme10.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-03**: expected `1885-05-02 Bill from Blun & Co.jpg`, got `Document between E Bought and BLUN & CO.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-04**: expected `1882-03-30 Bill from D. Crehange.jpg`, got `Document between Boyd Suits and Shipe HWHOLESALE CLOTHIERS.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-05**: expected `1891-07-12 Receipt from Rosenthal Bros.jpg`, got `Document from Soyg New York 189.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real5-invoice-06**: expected `2025-09-09 Tax Invoice from Ian Davies Plant Limited.pdf`, got `2025-09-09 Tax Invoice from Neath Port Talbot County Borough Council.pdf` · trap: party Neath Port Talbot County Borough Council (the customer) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-invoice-07**: expected `2025-09-30 Debt Position and Activity Report.pdf`, got `2025-09-30 Activity Report.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-01**: expected `1990-06-11 Request for Records Disposition Authority from Agricultural Research Service.pdf`, got `1990-06-15 Document from National Archives and Records Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-form-02**: expected `1982-03-02 Request for Records Disposition Authority from Internal Revenue Service.pdf`, got `1982-03-04 Document - National Archives and Records Service.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-01**: expected `2025-02-14 Memorandum from National Labor Relations Board.pdf`, got `2025-02-14 General Counsel Memorandum to Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-02**: expected `2025-05-16 Memorandum from National Labor Relations Board.pdf`, got `2025-05-16 General Counsel Memorandum to All Regional Directors, Officers-in-Charge, and Resident Officers.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-03**: expected `2025-01-29 Memorandum from Office of Management and Budget.pdf`, got `2025-01-29 Memorandum for Heads of Executive Departments and Agencies.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-04**: expected `2025-04-03 Memorandum from Office of Management and Budget.pdf`, got `2025-04-03 Memorandum for the Heads of Executive Departments and Agencies.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-08**: expected `2025-05-29 Board Meeting Minutes for Federal Retirement Thrift Investment Board.pdf`, got `2025-05-29 Document from Federal Retirement Thrift Investment Board.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real5-memo-10**: expected `2024-02-14 Meeting Minutes for Visiting Committee on Advanced Technology.pdf`, got `2024-02-14 Minutes for Visiting Committee on Advanced Technology (VCAT or Committee).pdf` · review: LOW_CONFIDENCE
- **real5-office-01**: expected `2018-06-14 Minutes.docx`, got `2018-06-14 Minutes for Phil Brown.docx` · trap: party Phil Brown (chair of the review, a person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real5-office-04**: expected `2023-09-30 Allocation, Spend and Commitment Statement.xlsx`, got `2023-09-30 Document - Wales Rural Development Programme.xlsx` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
