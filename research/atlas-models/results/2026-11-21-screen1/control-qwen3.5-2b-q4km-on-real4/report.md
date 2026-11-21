# InternBench: live run

- **Run:** 2026-11-21T04:50:30Z · commit `a51447d321`
- **Machine:** INTEL(R) XEON(R) PLATINUM 8573C, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 55 documents · 55 completed · gold `284c464532`
- **Wall time:** 1119.60 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 22/52 (42.3%) |
| Document type | 51/55 (92.7%) |
| Date | 42/52 (80.8%) |
| Date role (when the reviewed date was chosen) | 13/41 (31.7%) |
| Parties | 34/55 (61.8%) |
| Relation word (when parties were named) | 33/52 (63.5%) |
| Party in the right role (when parties were named) | 7/21 (33.3%) |
| Ready / review routing | 8/38 (21.1%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 43/55 (78.2%) |
| Description specificity | 68.5% (mean of 55) |
| Evidence recall (model's quotes) | 44.9% (mean of 55) |
| Digest recall (facts reaching the digest) | 93.6% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 91.2% (mean of 55) |
| Sent to review | 94.5% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 53.8% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 54.5% | 55 |
| Review rate | 94.5% | 55 |
| Evidence recall | 44.9% | 55 |
| Total latency p50 | 14.55 s | 55 |
| Total latency p95 | 92.58 s | 55 |
| Generation latency p50 | 6.14 s | 55 |
| Generation latency p95 | 8.14 s | 55 |
| Generated tokens p50 | 85 | 55 |
| Generated tokens p95 | 108 | 55 |
| Prompt tokens p50 | 1143 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 52 |
| Trap date chosen | 6 of 55 |
| Forbidden party named | 12 of 55 |
| Spurious parties named | 16 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 145 (0.0%) |
| Right name sent to review anyway | 14 of 33 |
| Sent to review for an unsupported fact | 30 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 8/13 | 10/13 | 11/14 | 3/13 | 85.7% | 14.22 s | 17.97 s |
| letter | 25 | 5/24 | 18/24 | 12/25 | 2/12 | 100.0% | 14.35 s | 92.58 s |
| notice | 14 | 7/13 | 12/13 | 9/14 | 2/12 | 92.9% | 15.98 s | 31.99 s |
| statement | 2 | 2/2 | 2/2 | 2/2 | 1/1 | 100.0% | 8.72 s | 106.40 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 32 | 15/31 | 25/31 | 21/32 | 4/26 | 90.6% | 14.50 s | 92.58 s |
| office | 11 | 2/9 | 7/9 | 5/11 | 2/3 | 100.0% | 9.71 s | 20.08 s |
| scan | 10 | 3/10 | 8/10 | 6/10 | 2/9 | 100.0% | 28.45 s | 106.40 s |
| text | 2 | 2/2 | 2/2 | 2/2 | – | 100.0% | 8.72 s | 15.98 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 7/18 | 16/18 | 9/20 | 2/10 | 100.0% | 9.71 s | 15.98 s |
| 2-4 | 16 | 6/15 | 11/15 | 10/16 | 2/11 | 100.0% | 14.22 s | 20.08 s |
| 5-9 | 6 | 2/6 | 4/6 | 4/6 | 2/6 | 66.7% | 15.32 s | 20.67 s |
| 10-24 | 10 | 6/10 | 9/10 | 9/10 | 1/9 | 90.0% | 21.65 s | 38.81 s |
| 25-49 | 2 | 1/2 | 1/2 | 1/2 | 1/2 | 100.0% | 92.58 s | 106.40 s |
| 100+ | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 105.17 s | 105.17 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 7/13 | 11/13 | 11/13 | 2/11 | 0 of 13 | 30.75 s | 106.40 s | 6.71 s | 9.04 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 1/3 | 3/3 | 1/3 | 1 |
| addressee_redacted | 6 | 1/6 | 5/6 | 3/6 | 3 |
| agency_billing | 8 | 4/8 | 6/8 | 6/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attachments | 15 | 7/15 | 13/15 | 11/15 | 3 |
| body_recounts_events | 1 | – | – | 1/1 | 1 |
| born_digital | 32 | 15/31 | 25/31 | 21/32 | 8 |
| brand_vs_legal_name | 3 | 3/3 | 3/3 | 3/3 | 0 |
| broadcast_invoice | 8 | 4/8 | 6/8 | 6/8 | 1 |
| broken_text_layer | 1 | 0/1 | 0/1 | 1/1 | 1 |
| caption | 4 | 1/4 | 4/4 | 1/4 | 1 |
| card_statement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 2 | 0/2 | 2/2 | 0/2 | 1 |
| certified_mail | 4 | 2/4 | 4/4 | 3/4 | 1 |
| citation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 44 | 21/44 | 36/44 | 31/44 | 11 |
| contract_numbers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 4 | 1/4 | 4/4 | 1/4 | 1 |
| cover_letter | 8 | 4/8 | 7/8 | 6/8 | 2 |
| customer_first | 1 | 0/1 | 0/1 | 0/1 | 0 |
| customer_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_stamp | 3 | 0/3 | 1/3 | 0/3 | 2 |
| deadline_in_body | 7 | 4/7 | 7/7 | 6/7 | 1 |
| doc | 3 | 1/2 | 2/2 | 2/3 | 1 |
| docket_number | 3 | 1/3 | 3/3 | 1/3 | 1 |
| docx | 4 | 1/4 | 3/4 | 3/4 | 2 |
| doubled_ocr_text | 4 | 2/4 | 4/4 | 3/4 | 1 |
| due_date | 4 | 4/4 | 4/4 | 4/4 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 0/2 | 2/2 | 0/2 | 2 |
| embedded_second_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| enclosure_list | 1 | – | – | 1/1 | 1 |
| esignature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| fee_invoice | 2 | 0/1 | 0/1 | 1/2 | 1 |
| file_name_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| filed_stamp | 4 | 1/4 | 4/4 | 1/4 | 1 |
| foi_release | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_response | 15 | 2/14 | 9/14 | 8/15 | 7 |
| forecaster_initials | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 2/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 0/1 | 1/1 | 0/1 | 1 |
| illegible_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| image_only | 2 | 1/2 | 1/2 | 1/2 | 1 |
| inspection_report | 4 | 2/4 | 3/4 | 3/4 | 1 |
| internal_email | 2 | 0/2 | 2/2 | 0/2 | 2 |
| invoice | 14 | 8/13 | 10/13 | 11/14 | 2 |
| issuer_in_logo | 1 | 0/1 | 0/1 | 0/1 | 0 |
| journal_number | 1 | – | – | 0/1 | 1 |
| journalist_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 25 | 5/24 | 18/24 | 12/25 | 13 |
| letterhead_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 9 | 5/9 | 7/9 | 7/9 | 1 |
| long_document | 2 | 0/2 | 1/2 | 1/2 | 0 |
| long_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_dates | 1 | – | – | 1/1 | 1 |
| metadata_date_trap | 3 | 0/2 | 1/2 | 1/3 | 1 |
| military_date_format | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 55 | 22/52 | 42/52 | 34/55 | 18 |
| msg | 2 | 0/2 | 2/2 | 0/2 | 2 |
| multiple_addressees | 1 | 0/1 | 0/1 | 1/1 | 1 |
| multiple_documents_in_file | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 19 | 5/19 | 14/19 | 9/19 | 7 |
| new_zealand | 4 | 1/4 | 4/4 | 2/4 | 2 |
| no_addressee | 2 | 0/1 | 0/1 | 2/2 | 2 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 1 |
| no_letterhead_text | 2 | 0/2 | 2/2 | 0/2 | 2 |
| no_signer_name | 1 | 0/1 | 0/1 | 0/1 | 1 |
| nonprofit | 1 | 0/1 | 0/1 | 1/1 | 1 |
| notice | 14 | 7/13 | 12/13 | 9/14 | 3 |
| notice_of_appeal | 4 | 1/4 | 4/4 | 1/4 | 1 |
| notice_of_violation | 2 | 1/2 | 1/2 | 2/2 | 0 |
| numeric_date_dmy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 8 | 2/8 | 7/8 | 5/8 | 2 |
| odt | 1 | 0/1 | 0/1 | 0/1 | 1 |
| office | 11 | 2/9 | 7/9 | 5/11 | 7 |
| on_behalf_of_client | 2 | 0/2 | 2/2 | 0/2 | 1 |
| ordinal_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| pay_gov | 1 | 0/1 | 0/1 | 0/1 | 0 |
| payment_page | 2 | 2/2 | 2/2 | 2/2 | 0 |
| penalty_notice | 6 | 4/6 | 6/6 | 5/6 | 1 |
| person_in_official_capacity | 3 | 1/3 | 3/3 | 1/3 | 0 |
| plaintiff_appeals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| political_file | 8 | 4/8 | 6/8 | 6/8 | 1 |
| preliminary_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| proration_note | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 1/3 | 2/3 | 1/3 | 1 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted_signer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_number | 9 | 1/8 | 5/8 | 5/9 | 6 |
| referral | 1 | 0/1 | 0/1 | 1/1 | 1 |
| remit_to_bank | 3 | 3/3 | 3/3 | 3/3 | 0 |
| rtf | 1 | – | – | 0/1 | 1 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 10 | 3/10 | 8/10 | 6/10 | 3 |
| service_period | 11 | 7/11 | 9/11 | 9/11 | 1 |
| short | 5 | 1/4 | 4/4 | 1/5 | 3 |
| signature_block_org | 1 | 0/1 | 1/1 | 0/1 | 1 |
| signature_in_caps | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 6 | 2/6 | 5/6 | 3/6 | 3 |
| statement | 2 | 2/2 | 2/2 | 2/2 | 0 |
| system_generated | 1 | – | – | 1/1 | 1 |
| tabular_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 2 | 2/2 | 2/2 | 2/2 | 0 |
| title_line | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_digit_year | 8 | 4/8 | 6/8 | 6/8 | 1 |
| txt | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk | 7 | 2/5 | 3/5 | 5/7 | 4 |
| undated | 3 | – | – | 2/3 | 3 |
| us | 8 | 4/8 | 6/8 | 6/8 | 1 |
| us_city | 3 | 3/3 | 3/3 | 3/3 | 0 |
| us_federal | 16 | 6/15 | 10/15 | 10/16 | 4 |
| us_federal_court | 4 | 1/4 | 4/4 | 1/4 | 1 |
| us_state | 12 | 5/12 | 12/12 | 6/12 | 5 |
| utc_vs_local | 2 | 2/2 | 2/2 | 2/2 | 0 |
| vendor_invoice | 4 | 4/4 | 4/4 | 4/4 | 0 |
| weather_product | 2 | 2/2 | 2/2 | 2/2 | 0 |
| wmo_header | 2 | 2/2 | 2/2 | 2/2 | 0 |
| zero_amount | 1 | – | – | 1/1 | 1 |
| zone_list | 1 | 1/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 108 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 24 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 14.55 s | 92.58 s | 106.40 s |
| `extraction_wall_ms` | 55 | 14.2 ms | 62.82 s | 89.46 s |
| `worker_total_ms` | 55 | 12.6 ms | 62.81 s | 89.45 s |
| `worker_snapshot_ms` | 55 | 0.25 ms | 0.92 ms | 3.7 ms |
| `worker_parse_ms` | 55 | 4.6 ms | 112.4 ms | 230.1 ms |
| `worker_analysis_ms` | 55 | 0.86 ms | 72.9 ms | 124.1 ms |
| `worker_render_ms` | 55 | 0.00 ms | 5.09 s | 12.68 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 122.73 s | 177.17 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 6.30 s | 9.76 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 112.63 s | 170.53 s |
| `worker_vision_ms` | 55 | 0.00 ms | 0.00 ms | 35.9 ms |
| `analyze_wall_ms` | 55 | 14.22 s | 26.43 s | 29.76 s |
| `distill_ms` | 55 | 2.5 ms | 69.9 ms | 87.7 ms |
| `prompt_ms` | 55 | 0.05 ms | 0.10 ms | 0.12 ms |
| `inference_ms` | 55 | 14.21 s | 26.33 s | 29.69 s |
| `prefill_ms` | 55 | 8.30 s | 18.68 s | 23.24 s |
| `generation_ms` | 55 | 6.14 s | 8.14 s | 9.04 s |
| `validation_ms` | 55 | 7.5 ms | 40.7 ms | 54.3 ms |
| `naming_ms` | 55 | 0.25 ms | 4.4 ms | 5.5 ms |
| `index_ms` | 55 | 2.2 ms | 65.4 ms | 76.5 ms |
| `retrieval_ms` | 55 | 0.05 ms | 4.1 ms | 5.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 135.4 tok/s | 141.1 tok/s | 141.3 tok/s |
| `generation_tok_per_s` | 55 | 13.8 tok/s | 14.3 tok/s | 14.4 tok/s |
| `prompt_tokens` | 55 | 1143 | 2503 | 3121 |
| `cached_tokens` | 55 | 477 | 477 | 546 |
| `generated_tokens` | 55 | 85 | 108 | 120 |
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
| fast | 18 | 2.4 ms | 231.3 ms | 2.0 ms | 230.9 ms | 230.1 ms | 0.42 ms | 0.00 ms | 0.00 ms |
| layout | 24 | 13.2 ms | 47.7 ms | 12.5 ms | 43.8 ms | 33.4 ms | 14.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 1.17 s | 1.57 s | 1.17 s | 1.57 s | 14.8 ms | 0.86 ms | 122.2 ms | 1.40 s |
| ocr | 11 | 6.37 s | 89.46 s | 6.36 s | 89.45 s | 113.6 ms | 124.1 ms | 12.68 s | 177.17 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 3.8 ms | 231.3 ms | 3.3 ms | 230.9 ms | 15.0 ms | 1.2 ms | 0.00 ms | 0.00 ms |
| 2-4 | 16 | 7.0 ms | 5.31 s | 6.5 ms | 5.31 s | 26.2 ms | 4.9 ms | 480.5 ms | 8.24 s |
| 5-9 | 6 | 25.1 ms | 47.7 ms | 22.3 ms | 43.8 ms | 27.2 ms | 15.8 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 53.0 ms | 19.77 s | 51.0 ms | 19.77 s | 113.6 ms | 124.1 ms | 1.37 s | 38.42 s |
| 25-49 | 2 | 62.82 s | 89.46 s | 62.81 s | 89.45 s | 3.6 ms | 14.8 ms | 8.96 s | 177.17 s |
| 100+ | 1 | 76.99 s | 76.99 s | 76.98 s | 76.98 s | 29.8 ms | 15.1 ms | 12.68 s | 152.76 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 7.1 ms | 47.7 ms | 6.5 ms | 43.8 ms | 27.2 ms | 15.8 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 14.6 ms | 62.82 s | 14.2 ms | 62.81 s | 29.8 ms | 3.9 ms | 5.09 s | 122.73 s |
| notice | 14 | 17.1 ms | 6.61 s | 16.0 ms | 6.60 s | 113.6 ms | 124.1 ms | 784.8 ms | 12.21 s |
| statement | 2 | 0.73 ms | 89.46 s | 0.40 ms | 89.45 s | 1.4 ms | 14.8 ms | 8.96 s | 177.17 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 226 MB | 1256 MB | real4-letter-13 |
| Model server | 1594 MB | 1594 MB | real4-office-13 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (30)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report from Paula Gerfen.pdf` · trap: party Paula Gerfen (addressee, an officer of the licensee) · review: LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Laura Berntsen.pdf` · trap: party Laura Berntsen (acting manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter to Te Whatu Ora - Health NZ.pdf` · trap: party Te Whatu Ora - Health NZ (subject of the request) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-06**: expected `2024-10-04 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-10-04 Response to an Official Information Act Request Ref - Ministry for Pacific Peoples.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, PARSER_WARNING
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Letter.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-08**: expected `2015-04-30 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-04-30 Letter to Amanda L. Edens.pdf` · trap: party Amanda L. Edens (director who signs) · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-09-08 Application and Approval for Off-duty Employment for PUBLIC.RESOURCE.ORG.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real4-letter-10**: expected `2015-06-11 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-06-11 Letter to Federal Highway Administration.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter to Marla Luckert.pdf` · trap: party Marla Luckert (chief justice who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-12**: expected `2021-03-15 Letter to Office of Management and Budget.pdf`, got `2021-01-20 Letter to Office of Management and Budget.pdf` · trap: date 2021-01-20 (date of the Presidential Memorandum cited) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real4-letter-13**: expected `2016-05-18 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2016-01-20 Letter to Mary Saunders.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Letter to Carl Malamud.pdf` · trap: party Carl Malamud (addressee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-01**: expected `2026-01-02 Notice of Appeal between State of California and U.S. Department of Transportation.pdf`, got `2026-01-02 Notice of Appeal for United States.pdf` · trap: party United States Court of Appeals for the First Circuit (court appealed to) · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-02**: expected `2026-02-17 Notice of Appeal between City of Philadelphia and United States Department of the Interior.pdf`, got `2026-02-17 Notice of Appeal for City of Philadelphia.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for Joe Neguse.pdf` · review: PARTY_UNSUPPORTED
- **real4-notice-05**: expected `2025-05-23 Notice of Violation for POINT Biopharma USA Inc.pdf`, got `2025-05-21 Notice of Violation for Point Biopharma USA Inc.pdf` · review: LOW_CONFIDENCE
- **real4-notice-12**: expected `2025-09-04 Notice of Civil Penalty Assessment and Order to Crown Enterprises, LLC.pdf`, got `2025-08-26 Notice of Civil Penalty Assessment and Order for Crown Enterprises LLC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-13**: expected `2025-06-17 Notice of Civil Penalty Assessment and Order to City of Stanfield.pdf`, got `2025-06-17 Notice of Civil Penalty Assessment and Order for Tina Kotek.pdf` · trap: party Tina Kotek (Governor named on the letterhead) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `Requester Invoice.pdf` · review: DATE_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice from WTTG.pdf` · review: DATE_IS_PERIOD, LOW_CONFIDENCE
- **real4-invoice-11**: expected `2024-06-30 Invoice from WCCO-TV.pdf`, got `2024-06-30 Invoice - America Leads Inc.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `2024-11-10 Invoice.pdf` · review: PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Receipt from Christopher T. Wright.docx` · trap: party Christopher T. Wright (addressee, an attorney at the firm) · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-03**: expected `2021-11-29 Letter to Post and Schell.doc`, got `2021-11-29 Letter to Commonwealth of Pennsylvania.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-06**: expected `2021-02-01 Freedom of Information Request Response from Highways England.odt`, got `2021-01-02 Letter to Freedom of Information Request.odt` · trap: date 2021-01-02 (date of the request) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-03-08 Letter to OAG.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-09**: expected `2019-09-16 Freedom of Information Act Response from Office of Manpower Economics.docx`, got `2019-09-09 Letter to Office of Manpower Economics.docx` · trap: date 2019-09-09 (date of the request) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Agreement with Bruce Betts.msg` · trap: party Bruce Betts (sender) · review: TYPE_MISSING, PARTY_MISSING, LOW_CONFIDENCE
- **real4-office-11**: expected `2016-08-01 Email to California Building Standards Commission.msg`, got `2016-08-01 Email - Alexander Hunter.msg` · trap: party Alexander Hunter (sender) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
