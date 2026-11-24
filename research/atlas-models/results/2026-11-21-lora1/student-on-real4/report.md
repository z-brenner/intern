# InternBench: live run

- **Run:** 2026-11-21T09:36:38Z · commit `ae77a5a7d2`
- **Machine:** AMD EPYC 9V74 80-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `99e3ad6efd92`
- **Corpus:** 55 documents · 55 completed · gold `284c464532`
- **Wall time:** 598.63 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 18/52 (34.6%) |
| Document type | 47/55 (85.5%) |
| Date | 44/52 (84.6%) |
| Date role (when the reviewed date was chosen) | 33/44 (75.0%) |
| Parties | 38/55 (69.1%) |
| Relation word (when parties were named) | 39/52 (75.0%) |
| Party in the right role (when parties were named) | 10/23 (43.5%) |
| Ready / review routing | 7/38 (18.4%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 39/55 (70.9%) |
| Description specificity | 64.8% (mean of 55) |
| Evidence recall (model's quotes) | 60.0% (mean of 55) |
| Digest recall (facts reaching the digest) | 93.6% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 91.2% (mean of 55) |
| Sent to review | 96.4% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 38.5% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 69.1% | 55 |
| Review rate | 96.4% | 55 |
| Evidence recall | 60.0% | 55 |
| Total latency p50 | 6.16 s | 55 |
| Total latency p95 | 69.83 s | 55 |
| Generation latency p50 | 1.89 s | 55 |
| Generation latency p95 | 3.22 s | 55 |
| Generated tokens p50 | 76 | 55 |
| Generated tokens p95 | 119 | 55 |
| Prompt tokens p50 | 1143 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 52 |
| Trap date chosen | 3 of 55 |
| Forbidden party named | 8 of 55 |
| Spurious parties named | 14 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 146 (0.0%) |
| Right name sent to review anyway | 12 of 33 |
| Sent to review for an unsupported fact | 38 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 8/13 | 10/13 | 12/14 | 1/13 | 100.0% | 5.70 s | 7.91 s |
| letter | 25 | 5/24 | 21/24 | 14/25 | 2/12 | 100.0% | 5.94 s | 69.83 s |
| notice | 14 | 4/13 | 11/13 | 11/14 | 3/12 | 85.7% | 6.62 s | 18.00 s |
| statement | 2 | 1/2 | 2/2 | 1/2 | 1/1 | 100.0% | 3.86 s | 83.75 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 32 | 13/31 | 27/31 | 23/32 | 2/26 | 96.9% | 5.94 s | 69.83 s |
| office | 11 | 2/9 | 9/9 | 7/11 | 2/3 | 100.0% | 3.58 s | 7.78 s |
| scan | 10 | 2/10 | 6/10 | 7/10 | 3/9 | 90.0% | 15.90 s | 83.75 s |
| text | 2 | 1/2 | 2/2 | 1/2 | – | 100.0% | 3.86 s | 6.16 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 7/18 | 17/18 | 10/20 | 2/10 | 100.0% | 3.58 s | 6.57 s |
| 2-4 | 16 | 4/15 | 14/15 | 12/16 | 2/11 | 100.0% | 5.70 s | 10.82 s |
| 5-9 | 6 | 2/6 | 3/6 | 6/6 | 0/6 | 100.0% | 6.77 s | 11.22 s |
| 10-24 | 10 | 5/10 | 8/10 | 9/10 | 2/9 | 80.0% | 9.36 s | 27.86 s |
| 25-49 | 2 | 0/2 | 1/2 | 0/2 | 1/2 | 100.0% | 69.83 s | 83.75 s |
| 100+ | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 76.00 s | 76.00 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 5/13 | 10/13 | 10/13 | 3/11 | 1 of 13 | 16.14 s | 83.75 s | 2.05 s | 3.52 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 1/3 | 3/3 | 2/3 | 0 |
| addressee_redacted | 6 | 1/6 | 6/6 | 4/6 | 2 |
| agency_billing | 8 | 5/8 | 5/8 | 8/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachments | 15 | 6/15 | 12/15 | 12/15 | 0 |
| body_recounts_events | 1 | – | – | 1/1 | 1 |
| born_digital | 32 | 13/31 | 27/31 | 23/32 | 8 |
| brand_vs_legal_name | 3 | 2/3 | 3/3 | 3/3 | 0 |
| broadcast_invoice | 8 | 5/8 | 5/8 | 8/8 | 1 |
| broken_text_layer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| caption | 4 | 0/4 | 4/4 | 3/4 | 0 |
| card_statement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| certificate_of_service | 2 | 0/2 | 2/2 | 1/2 | 0 |
| certified_mail | 4 | 1/4 | 2/4 | 4/4 | 0 |
| citation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 44 | 16/44 | 37/44 | 32/44 | 7 |
| contract_numbers | 1 | 0/1 | 1/1 | 0/1 | 0 |
| court_filing | 4 | 0/4 | 4/4 | 3/4 | 0 |
| cover_letter | 8 | 3/8 | 6/8 | 8/8 | 0 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 1 |
| customer_redacted | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_stamp | 3 | 1/3 | 1/3 | 2/3 | 0 |
| deadline_in_body | 7 | 3/7 | 5/7 | 7/7 | 0 |
| doc | 3 | 1/2 | 2/2 | 2/3 | 2 |
| docket_number | 3 | 1/3 | 3/3 | 2/3 | 0 |
| docx | 4 | 0/4 | 4/4 | 3/4 | 0 |
| doubled_ocr_text | 4 | 1/4 | 2/4 | 4/4 | 0 |
| due_date | 4 | 3/4 | 4/4 | 4/4 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| embedded_second_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enclosure_list | 1 | – | – | 1/1 | 1 |
| esignature | 1 | 0/1 | 1/1 | 0/1 | 1 |
| fee_invoice | 2 | 0/1 | 1/1 | 0/2 | 2 |
| file_name_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| filed_stamp | 4 | 0/4 | 4/4 | 3/4 | 0 |
| foi_release | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_response | 15 | 2/14 | 11/14 | 9/15 | 4 |
| forecaster_initials | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 2/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 0 |
| inspection_report | 4 | 2/4 | 4/4 | 4/4 | 0 |
| internal_email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| invoice | 14 | 8/13 | 10/13 | 12/14 | 3 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| journal_number | 1 | – | – | 0/1 | 1 |
| journalist_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letter | 25 | 5/24 | 21/24 | 14/25 | 7 |
| letterhead_names | 1 | 0/1 | 1/1 | 0/1 | 1 |
| line_item_dates | 9 | 5/9 | 6/9 | 8/9 | 1 |
| long_document | 2 | 0/2 | 1/2 | 1/2 | 0 |
| long_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_dates | 1 | – | – | 1/1 | 1 |
| metadata_date_trap | 3 | 1/2 | 1/2 | 1/3 | 1 |
| military_date_format | 1 | 0/1 | 1/1 | 0/1 | 0 |
| modern | 55 | 18/52 | 44/52 | 38/55 | 11 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 0 |
| multiple_addressees | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_documents_in_file | 1 | 0/1 | 1/1 | 0/1 | 0 |
| named_person_at_org | 19 | 5/19 | 17/19 | 9/19 | 5 |
| new_zealand | 4 | 1/4 | 4/4 | 2/4 | 2 |
| no_addressee | 2 | 0/1 | 1/1 | 2/2 | 1 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 1 |
| no_letterhead_text | 2 | 0/2 | 2/2 | 1/2 | 1 |
| no_signer_name | 1 | 0/1 | 1/1 | 1/1 | 0 |
| nonprofit | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notice | 14 | 4/13 | 11/13 | 11/14 | 1 |
| notice_of_appeal | 4 | 0/4 | 4/4 | 3/4 | 0 |
| notice_of_violation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| numeric_date_dmy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 8 | 2/8 | 5/8 | 6/8 | 0 |
| odt | 1 | 0/1 | 1/1 | 1/1 | 0 |
| office | 11 | 2/9 | 9/9 | 7/11 | 3 |
| on_behalf_of_client | 2 | 1/2 | 2/2 | 2/2 | 0 |
| ordinal_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| pay_gov | 1 | 0/1 | 1/1 | 0/1 | 1 |
| payment_page | 2 | 1/2 | 2/2 | 2/2 | 0 |
| penalty_notice | 6 | 3/6 | 4/6 | 6/6 | 0 |
| person_in_official_capacity | 3 | 0/3 | 3/3 | 3/3 | 0 |
| plaintiff_appeals | 1 | 0/1 | 1/1 | 1/1 | 0 |
| political_file | 8 | 5/8 | 5/8 | 8/8 | 1 |
| preliminary_data | 1 | 0/1 | 1/1 | 0/1 | 0 |
| proration_note | 1 | 0/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 1/3 | 2/3 | 1/3 | 1 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted_signer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_number | 9 | 1/8 | 7/8 | 7/9 | 3 |
| referral | 1 | 0/1 | 1/1 | 1/1 | 0 |
| remit_to_bank | 3 | 2/3 | 3/3 | 3/3 | 0 |
| rtf | 1 | – | – | 0/1 | 1 |
| same_day_order | 1 | 0/1 | 1/1 | 1/1 | 0 |
| scanned | 10 | 2/10 | 6/10 | 7/10 | 0 |
| service_period | 11 | 7/11 | 8/11 | 11/11 | 1 |
| short | 5 | 1/4 | 4/4 | 2/5 | 2 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 0/1 | 1/1 | 0/1 | 1 |
| signature_on_later_page | 6 | 2/6 | 5/6 | 4/6 | 1 |
| statement | 2 | 1/2 | 2/2 | 1/2 | 0 |
| system_generated | 1 | – | – | 0/1 | 1 |
| tabular_body | 1 | 0/1 | 1/1 | 0/1 | 0 |
| text | 2 | 1/2 | 2/2 | 1/2 | 0 |
| title_line | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_digit_year | 8 | 5/8 | 5/8 | 8/8 | 1 |
| txt | 2 | 1/2 | 2/2 | 1/2 | 0 |
| uk | 7 | 0/5 | 5/5 | 4/7 | 3 |
| undated | 3 | – | – | 1/3 | 3 |
| us | 8 | 5/8 | 5/8 | 8/8 | 1 |
| us_city | 3 | 2/3 | 3/3 | 3/3 | 0 |
| us_federal | 16 | 6/15 | 12/15 | 10/16 | 2 |
| us_federal_court | 4 | 0/4 | 4/4 | 3/4 | 0 |
| us_state | 12 | 4/12 | 10/12 | 8/12 | 2 |
| utc_vs_local | 2 | 1/2 | 2/2 | 1/2 | 0 |
| vendor_invoice | 4 | 3/4 | 4/4 | 4/4 | 0 |
| weather_product | 2 | 1/2 | 2/2 | 1/2 | 0 |
| wmo_header | 2 | 1/2 | 2/2 | 1/2 | 0 |
| zero_amount | 1 | – | – | 0/1 | 1 |
| zone_list | 1 | 1/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 108 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 24 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 6.16 s | 69.83 s | 83.75 s |
| `extraction_wall_ms` | 55 | 11.1 ms | 57.62 s | 75.87 s |
| `worker_total_ms` | 55 | 10.6 ms | 57.61 s | 75.86 s |
| `worker_snapshot_ms` | 55 | 0.23 ms | 0.75 ms | 3.1 ms |
| `worker_parse_ms` | 55 | 4.0 ms | 95.1 ms | 98.5 ms |
| `worker_analysis_ms` | 55 | 0.68 ms | 59.8 ms | 102.8 ms |
| `worker_render_ms` | 55 | 0.00 ms | 4.43 s | 10.93 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 113.02 s | 150.24 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 5.11 s | 8.76 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 104.86 s | 144.84 s |
| `worker_vision_ms` | 55 | 0.00 ms | 0.00 ms | 20.4 ms |
| `analyze_wall_ms` | 55 | 5.98 s | 11.70 s | 13.35 s |
| `distill_ms` | 55 | 2.0 ms | 56.7 ms | 64.9 ms |
| `prompt_ms` | 55 | 0.04 ms | 0.08 ms | 0.09 ms |
| `inference_ms` | 55 | 5.95 s | 11.64 s | 13.22 s |
| `prefill_ms` | 55 | 3.81 s | 9.05 s | 11.44 s |
| `generation_ms` | 55 | 1.89 s | 3.22 s | 3.83 s |
| `validation_ms` | 55 | 7.3 ms | 32.9 ms | 57.0 ms |
| `naming_ms` | 55 | 0.19 ms | 3.1 ms | 4.1 ms |
| `index_ms` | 55 | 1.8 ms | 51.8 ms | 60.1 ms |
| `retrieval_ms` | 55 | 0.04 ms | 3.3 ms | 4.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 293.0 tok/s | 302.2 tok/s | 302.6 tok/s |
| `generation_tok_per_s` | 55 | 39.9 tok/s | 42.4 tok/s | 43.1 tok/s |
| `prompt_tokens` | 55 | 1143 | 2503 | 3121 |
| `cached_tokens` | 55 | 477 | 477 | 546 |
| `generated_tokens` | 55 | 76 | 119 | 142 |
| `estimated_prompt_tokens` | 55 | 1596 | 2878 | 3398 |
| `model_requests` | 55 | 1 | 1 | 1 |
| `redistillations` | 55 | 0 | 0 | 0 |
| `source_characters` | 55 | 2982 | 75251 | 87914 |
| `digest_characters` | 55 | 2847 | 6325 | 7031 |
| `prompt_characters` | 55 | 4896 | 8380 | 9087 |
| `worker_ocr_pages` | 55 | 0 | 29 | 66 |
| `worker_ocr_passes` | 55 | 0 | 29 | 66 |
| `worker_orientation_passes` | 55 | 0 | 29 | 66 |
| `worker_rendered_pixels` | 55 | 0 | 252367280 | 574406515 |
| `index_units` | 55 | 34 | 1694 | 1903 |
| `context_units` | 55 | 34 | 98 | 122 |
| `context_tokens` | 55 | 1008 | 2288 | 2806 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 18 | 2.8 ms | 96.0 ms | 2.5 ms | 95.8 ms | 95.1 ms | 0.34 ms | 0.00 ms | 0.00 ms |
| layout | 24 | 11.1 ms | 34.3 ms | 10.0 ms | 32.9 ms | 28.1 ms | 11.3 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 911.1 ms | 1.40 s | 910.7 ms | 1.40 s | 14.5 ms | 0.68 ms | 116.1 ms | 1.23 s |
| ocr | 11 | 6.30 s | 75.87 s | 6.29 s | 75.86 s | 98.5 ms | 102.8 ms | 10.93 s | 150.24 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 3.0 ms | 96.0 ms | 2.7 ms | 95.8 ms | 13.8 ms | 0.99 ms | 0.00 ms | 0.00 ms |
| 2-4 | 16 | 6.0 ms | 4.76 s | 5.6 ms | 4.76 s | 27.3 ms | 3.8 ms | 397.7 ms | 7.26 s |
| 5-9 | 6 | 19.0 ms | 32.0 ms | 17.2 ms | 29.3 ms | 17.3 ms | 11.3 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 42.8 ms | 19.55 s | 41.3 ms | 19.54 s | 98.5 ms | 102.8 ms | 1.24 s | 37.94 s |
| 25-49 | 2 | 57.62 s | 75.87 s | 57.61 s | 75.86 s | 3.7 ms | 11.5 ms | 7.36 s | 150.24 s |
| 100+ | 1 | 62.65 s | 62.65 s | 62.64 s | 62.64 s | 25.6 ms | 11.5 ms | 10.93 s | 124.25 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 6.2 ms | 32.0 ms | 5.6 ms | 29.3 ms | 17.3 ms | 11.3 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 11.1 ms | 57.62 s | 10.7 ms | 57.61 s | 27.3 ms | 3.0 ms | 4.43 s | 113.02 s |
| notice | 14 | 14.1 ms | 6.36 s | 13.3 ms | 6.35 s | 98.5 ms | 102.8 ms | 698.7 ms | 11.89 s |
| statement | 2 | 1.4 ms | 75.87 s | 0.36 ms | 75.86 s | 1.2 ms | 11.5 ms | 7.36 s | 150.24 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 272 MB | 1332 MB | real4-letter-13 |
| Model server | 852 MB | 852 MB | real4-office-13 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (34)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report for Pacific Gas and Electric Company.pdf` · review: LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Kaiwhakaoranga Service.pdf` · trap: party Kaiwhakaoranga Service (subject of the request, not a party) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter to Te Whatu Ora - Health NZ.pdf` · trap: party Te Whatu Ora - Health NZ (subject of the request) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-06**: expected `2024-10-04 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-10-04 Order Receipt from Ministry for Pacific Peoples.pdf` · review: PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Freedom of Information Act Request Acknowledgement.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-09-08 Application and Approval for Off-duty Employment for AFLCMCXZI.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-10**: expected `2015-06-11 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-06-11 Letter to Federal Highway Administration.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter to Kansas Judicial Council.pdf` · trap: party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-letter-12**: expected `2021-03-15 Letter to Office of Management and Budget.pdf`, got `2021-03-15 Letter to Office of Information and Regulatory Affairs.pdf` · trap: party Office of Information and Regulatory Affairs (addressee of the attached letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-13**: expected `2016-05-18 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2010-10-21 Letter to Mary Saunders.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Letter to Coast Guard.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-15**: expected `2021-08-27 Letter to Public.Resource.Org.pdf`, got `2021-08-27 Letter to Robert Stivers.pdf` · trap: party Robert Stivers (legislator on the letterhead) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-01**: expected `2026-01-02 Notice of Appeal between State of California and U.S. Department of Transportation.pdf`, got `2026-01-02 Notice of Appeal.pdf` · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real4-notice-02**: expected `2026-02-17 Notice of Appeal between City of Philadelphia and United States Department of the Interior.pdf`, got `2026-02-17 Notice of Appeal for United States.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal between JOE NEGUSE and U.S. Immigration and Customs.pdf` · review: LOW_CONFIDENCE
- **real4-notice-04**: expected `2026-05-01 Notice of Appeal between United States of America and Tim Walz.pdf`, got `2026-05-01 Notice of Appeal for United States of America.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-06**: expected `2025-12-15 Notice of Violation for Tennessee Valley Authority.pdf`, got `2025-12-15 Assessment Followup Letter Nrc Inspection Report to Tennessee Valley Authority.pdf` · review: LOW_CONFIDENCE
- **real4-notice-13**: expected `2025-06-17 Notice of Civil Penalty Assessment and Order to City of Stanfield.pdf`, got `2024-07-25 Notice of Civil Penalty Assessment and Order from Department of Environmental Quality.pdf` · review: LOW_CONFIDENCE
- **real4-notice-14**: expected `2025-10-28 Notice of Civil Penalty Assessment and Order to Erickson Incorporated.pdf`, got `2023-02-15 Document from Department of Environmental Quality.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-15**: expected `2025-07-22 Notice of Civil Penalty Assessment and Order to Columbia Plywood Corporation.pdf`, got `2025-07-22 Notice of Civil Penalty Assessm Ent a from Department of Environmental Quality.pdf` · **filed without review**
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `2023-10-05 Invoice from Sam Brownell.pdf` · trap: party Sam Brownell (requester, a person at the customer company) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-invoice-05**: expected `2021-02-08 Invoice from Flock Safety.pdf`, got `2021-02-08 Document from Flock Group Inc.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Document from WTTG.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `2024-11-03 Document from WCCO.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-13**: expected `2010-04-28 Statement from Barclaycard Commercial.pdf`, got `2010-04-28 Statement.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-office-01**: expected `2018-06-21 Letter to Duquesne Light Company.docx`, got `2018-06-21 Letter to Commonwealth of Pennsylvania.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Letter of Notification to PPL Electric Utilities Corporation.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-05**: expected `2012-03-29 Freedom of Information Act Response to Daily Telegraph.doc`, got `2012-03-29 Letter to Armed Forces Pay Review Body.doc` · trap: party Armed Forces Pay Review Body (body whose evidence was requested) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-06**: expected `2021-02-01 Freedom of Information Request Response from Highways England.odt`, got `2021-02-01 Letter to Historical Railways Estate.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-03-08 Letter to Office of the Advocate General.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-09**: expected `2019-09-16 Freedom of Information Act Response from Office of Manpower Economics.docx`, got `2019-09-16 Letter to Office of Manpower Economics.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Email to De la Motte, Leann.msg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-12**: expected `2026-10-19 Public Information Statement from National Weather Service Wichita KS.txt`, got `2026-10-19 Public Information Statement for Liberal Airport - A.txt` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
