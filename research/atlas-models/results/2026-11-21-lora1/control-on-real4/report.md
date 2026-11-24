# InternBench: live run

- **Run:** 2026-11-21T09:36:41Z · commit `ae77a5a7d2`
- **Machine:** INTEL(R) XEON(R) PLATINUM 8573C, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `97291f77f59a`
- **Corpus:** 55 documents · 55 completed · gold `284c464532`
- **Wall time:** 763.46 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 16/52 (30.8%) |
| Document type | 54/55 (98.2%) |
| Date | 36/52 (69.2%) |
| Date role (when the reviewed date was chosen) | 14/36 (38.9%) |
| Parties | 30/55 (54.5%) |
| Relation word (when parties were named) | 40/55 (72.7%) |
| Party in the right role (when parties were named) | 4/23 (17.4%) |
| Ready / review routing | 5/38 (13.2%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 43/55 (78.2%) |
| Description specificity | 69.7% (mean of 55) |
| Evidence recall (model's quotes) | 18.5% (mean of 55) |
| Digest recall (facts reaching the digest) | 93.6% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 91.2% (mean of 55) |
| Sent to review | 100.0% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 38.5% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 78.2% | 55 |
| Review rate | 100.0% | 55 |
| Evidence recall | 18.5% | 55 |
| Total latency p50 | 8.33 s | 55 |
| Total latency p95 | 79.41 s | 55 |
| Generation latency p50 | 3.81 s | 55 |
| Generation latency p95 | 5.06 s | 55 |
| Generated tokens p50 | 98 | 55 |
| Generated tokens p95 | 135 | 55 |
| Prompt tokens p50 | 1143 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 52 |
| Trap date chosen | 6 of 55 |
| Forbidden party named | 14 of 55 |
| Spurious parties named | 25 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 154 (0.0%) |
| Right name sent to review anyway | 12 of 33 |
| Sent to review for an unsupported fact | 43 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 7/13 | 8/13 | 12/14 | 1/13 | 100.0% | 8.07 s | 10.91 s |
| letter | 25 | 5/24 | 16/24 | 9/25 | 2/12 | 100.0% | 7.59 s | 79.41 s |
| notice | 14 | 3/13 | 10/13 | 7/14 | 1/12 | 100.0% | 9.30 s | 19.66 s |
| statement | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 100.0% | 7.63 s | 96.49 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 32 | 12/31 | 24/31 | 19/32 | 1/26 | 100.0% | 8.07 s | 79.41 s |
| office | 11 | 2/9 | 6/9 | 4/11 | 2/3 | 100.0% | 5.91 s | 11.47 s |
| scan | 10 | 2/10 | 4/10 | 5/10 | 2/9 | 100.0% | 18.35 s | 96.49 s |
| text | 2 | 0/2 | 2/2 | 2/2 | – | 100.0% | 7.63 s | 8.73 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 3/18 | 13/18 | 7/20 | 2/10 | 100.0% | 5.98 s | 8.73 s |
| 2-4 | 16 | 6/15 | 10/15 | 10/16 | 2/11 | 100.0% | 7.57 s | 13.60 s |
| 5-9 | 6 | 2/6 | 3/6 | 5/6 | 0/6 | 100.0% | 9.26 s | 11.87 s |
| 10-24 | 10 | 4/10 | 7/10 | 7/10 | 0/9 | 100.0% | 12.55 s | 30.84 s |
| 25-49 | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 100.0% | 79.41 s | 96.49 s |
| 100+ | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 95.34 s | 95.34 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 5/13 | 10/13 | 8/13 | 1/11 | 0 of 13 | 18.63 s | 96.49 s | 3.76 s | 4.98 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 0/3 | 3/3 | 0/3 | 1 |
| addressee_redacted | 6 | 0/6 | 4/6 | 2/6 | 3 |
| agency_billing | 8 | 4/8 | 5/8 | 8/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachments | 15 | 4/15 | 12/15 | 8/15 | 4 |
| body_recounts_events | 1 | – | – | 1/1 | 0 |
| born_digital | 32 | 12/31 | 24/31 | 19/32 | 11 |
| brand_vs_legal_name | 3 | 3/3 | 3/3 | 3/3 | 0 |
| broadcast_invoice | 8 | 4/8 | 5/8 | 8/8 | 1 |
| broken_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption | 4 | 0/4 | 4/4 | 0/4 | 3 |
| card_statement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 2 | 0/2 | 2/2 | 0/2 | 2 |
| certified_mail | 4 | 1/4 | 2/4 | 3/4 | 2 |
| citation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 44 | 15/44 | 30/44 | 28/44 | 14 |
| contract_numbers | 1 | 0/1 | 1/1 | 0/1 | 0 |
| court_filing | 4 | 0/4 | 4/4 | 0/4 | 3 |
| cover_letter | 8 | 2/8 | 5/8 | 5/8 | 3 |
| customer_first | 1 | 0/1 | 0/1 | 0/1 | 1 |
| customer_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_stamp | 3 | 0/3 | 0/3 | 0/3 | 3 |
| deadline_in_body | 7 | 3/7 | 5/7 | 5/7 | 2 |
| doc | 3 | 1/2 | 2/2 | 2/3 | 0 |
| docket_number | 3 | 0/3 | 3/3 | 0/3 | 1 |
| docx | 4 | 0/4 | 2/4 | 0/4 | 2 |
| doubled_ocr_text | 4 | 1/4 | 2/4 | 3/4 | 2 |
| due_date | 4 | 3/4 | 3/4 | 4/4 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| embedded_second_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enclosure_list | 1 | – | – | 1/1 | 0 |
| esignature | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fee_invoice | 2 | 0/1 | 0/1 | 0/2 | 2 |
| file_name_date_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| filed_stamp | 4 | 0/4 | 4/4 | 0/4 | 3 |
| foi_release | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_response | 15 | 1/14 | 7/14 | 5/15 | 7 |
| forecaster_initials | 1 | 0/1 | 1/1 | 1/1 | 0 |
| form | 2 | 2/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| image_only | 2 | 1/2 | 1/2 | 1/2 | 1 |
| inspection_report | 4 | 1/4 | 3/4 | 2/4 | 1 |
| internal_email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| invoice | 14 | 7/13 | 8/13 | 12/14 | 3 |
| issuer_in_logo | 1 | 0/1 | 0/1 | 0/1 | 1 |
| journal_number | 1 | – | – | 0/1 | 1 |
| journalist_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 25 | 5/24 | 16/24 | 9/25 | 11 |
| letterhead_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 9 | 5/9 | 6/9 | 9/9 | 1 |
| long_document | 2 | 0/2 | 2/2 | 0/2 | 0 |
| long_subject | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_dates | 1 | – | – | 1/1 | 0 |
| metadata_date_trap | 3 | 0/2 | 2/2 | 0/3 | 1 |
| military_date_format | 1 | 0/1 | 1/1 | 0/1 | 1 |
| modern | 55 | 16/52 | 36/52 | 30/55 | 20 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 0 |
| multiple_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_documents_in_file | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 19 | 5/19 | 13/19 | 7/19 | 8 |
| new_zealand | 4 | 0/4 | 4/4 | 1/4 | 2 |
| no_addressee | 2 | 0/1 | 0/1 | 1/2 | 0 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 1 |
| no_letterhead_text | 2 | 0/2 | 1/2 | 0/2 | 3 |
| no_signer_name | 1 | 0/1 | 0/1 | 1/1 | 0 |
| nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 14 | 3/13 | 10/13 | 7/14 | 6 |
| notice_of_appeal | 4 | 0/4 | 4/4 | 0/4 | 3 |
| notice_of_violation | 2 | 0/2 | 1/2 | 1/2 | 0 |
| numeric_date_dmy | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_text_layer | 8 | 1/8 | 3/8 | 4/8 | 5 |
| odt | 1 | 0/1 | 0/1 | 1/1 | 0 |
| office | 11 | 2/9 | 6/9 | 4/11 | 3 |
| on_behalf_of_client | 2 | 0/2 | 2/2 | 0/2 | 1 |
| ordinal_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| pay_gov | 1 | 0/1 | 0/1 | 0/1 | 1 |
| payment_page | 2 | 2/2 | 2/2 | 2/2 | 0 |
| penalty_notice | 6 | 3/6 | 4/6 | 5/6 | 2 |
| person_in_official_capacity | 3 | 0/3 | 3/3 | 0/3 | 2 |
| plaintiff_appeals | 1 | 0/1 | 1/1 | 0/1 | 0 |
| political_file | 8 | 4/8 | 5/8 | 8/8 | 1 |
| preliminary_data | 1 | 0/1 | 1/1 | 1/1 | 0 |
| proration_note | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 1/3 | 1/3 | 1/3 | 2 |
| re_line_references_document | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted_signer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| reference_number | 9 | 0/8 | 4/8 | 4/9 | 3 |
| referral | 1 | 0/1 | 0/1 | 0/1 | 0 |
| remit_to_bank | 3 | 2/3 | 2/3 | 3/3 | 0 |
| rtf | 1 | – | – | 0/1 | 1 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 1 |
| scanned | 10 | 2/10 | 4/10 | 5/10 | 6 |
| service_period | 11 | 6/11 | 7/11 | 11/11 | 1 |
| short | 5 | 0/4 | 3/4 | 0/5 | 4 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 6 | 2/6 | 3/6 | 3/6 | 4 |
| statement | 2 | 1/2 | 2/2 | 2/2 | 0 |
| system_generated | 1 | – | – | 0/1 | 1 |
| tabular_body | 1 | 0/1 | 1/1 | 1/1 | 0 |
| text | 2 | 0/2 | 2/2 | 2/2 | 0 |
| title_line | 2 | 0/2 | 2/2 | 1/2 | 0 |
| two_digit_year | 8 | 4/8 | 5/8 | 8/8 | 1 |
| txt | 2 | 0/2 | 2/2 | 2/2 | 0 |
| uk | 7 | 2/5 | 2/5 | 4/7 | 2 |
| undated | 3 | – | – | 1/3 | 2 |
| us | 8 | 4/8 | 5/8 | 8/8 | 1 |
| us_city | 3 | 3/3 | 3/3 | 3/3 | 0 |
| us_federal | 16 | 3/15 | 9/15 | 7/16 | 7 |
| us_federal_court | 4 | 0/4 | 4/4 | 0/4 | 3 |
| us_state | 12 | 3/12 | 8/12 | 6/12 | 5 |
| utc_vs_local | 2 | 0/2 | 2/2 | 2/2 | 0 |
| vendor_invoice | 4 | 3/4 | 3/4 | 4/4 | 0 |
| weather_product | 2 | 0/2 | 2/2 | 2/2 | 0 |
| wmo_header | 2 | 0/2 | 2/2 | 2/2 | 0 |
| zero_amount | 1 | – | – | 0/1 | 1 |
| zone_list | 1 | 0/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 108 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 24 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 8.33 s | 79.41 s | 96.49 s |
| `extraction_wall_ms` | 55 | 13.8 ms | 63.41 s | 86.58 s |
| `worker_total_ms` | 55 | 12.4 ms | 63.41 s | 86.57 s |
| `worker_snapshot_ms` | 55 | 0.25 ms | 0.72 ms | 3.5 ms |
| `worker_parse_ms` | 55 | 4.6 ms | 111.6 ms | 324.6 ms |
| `worker_analysis_ms` | 55 | 0.87 ms | 74.6 ms | 123.1 ms |
| `worker_render_ms` | 55 | 0.00 ms | 5.01 s | 12.67 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 124.20 s | 171.30 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 6.32 s | 9.66 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 114.53 s | 164.65 s |
| `worker_vision_ms` | 55 | 0.00 ms | 0.00 ms | 30.0 ms |
| `analyze_wall_ms` | 55 | 8.06 s | 13.81 s | 17.85 s |
| `distill_ms` | 55 | 2.5 ms | 69.6 ms | 82.3 ms |
| `prompt_ms` | 55 | 0.04 ms | 0.10 ms | 0.12 ms |
| `inference_ms` | 55 | 8.04 s | 13.70 s | 17.69 s |
| `prefill_ms` | 55 | 4.43 s | 10.10 s | 13.17 s |
| `generation_ms` | 55 | 3.81 s | 5.06 s | 5.34 s |
| `validation_ms` | 55 | 7.5 ms | 40.4 ms | 70.2 ms |
| `naming_ms` | 55 | 0.28 ms | 4.4 ms | 5.4 ms |
| `index_ms` | 55 | 2.2 ms | 65.2 ms | 75.7 ms |
| `retrieval_ms` | 55 | 0.04 ms | 4.2 ms | 5.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 257.3 tok/s | 264.1 tok/s | 264.6 tok/s |
| `generation_tok_per_s` | 55 | 26.0 tok/s | 27.9 tok/s | 29.5 tok/s |
| `prompt_tokens` | 55 | 1143 | 2503 | 3121 |
| `cached_tokens` | 55 | 477 | 477 | 546 |
| `generated_tokens` | 55 | 98 | 135 | 140 |
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
| fast | 18 | 2.6 ms | 325.7 ms | 2.3 ms | 325.4 ms | 324.6 ms | 0.45 ms | 0.00 ms | 0.00 ms |
| layout | 24 | 12.5 ms | 41.2 ms | 11.9 ms | 39.5 ms | 33.6 ms | 14.3 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 1.18 s | 1.56 s | 1.18 s | 1.56 s | 15.5 ms | 0.87 ms | 120.9 ms | 1.39 s |
| ocr | 11 | 5.83 s | 86.58 s | 5.83 s | 86.57 s | 113.0 ms | 123.1 ms | 12.67 s | 171.30 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 3.9 ms | 325.7 ms | 3.5 ms | 325.4 ms | 14.7 ms | 1.2 ms | 0.00 ms | 0.00 ms |
| 2-4 | 16 | 7.0 ms | 5.18 s | 6.3 ms | 5.18 s | 27.9 ms | 4.8 ms | 515.2 ms | 7.98 s |
| 5-9 | 6 | 23.3 ms | 39.0 ms | 20.9 ms | 35.4 ms | 22.2 ms | 14.5 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 52.9 ms | 20.09 s | 51.0 ms | 20.09 s | 113.0 ms | 123.1 ms | 1.37 s | 38.99 s |
| 25-49 | 2 | 63.41 s | 86.58 s | 63.41 s | 86.57 s | 3.7 ms | 14.8 ms | 8.91 s | 171.30 s |
| 100+ | 1 | 77.49 s | 77.49 s | 77.48 s | 77.48 s | 29.3 ms | 15.1 ms | 12.67 s | 153.78 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 7.6 ms | 39.0 ms | 6.4 ms | 35.4 ms | 22.2 ms | 14.5 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 14.0 ms | 63.41 s | 13.6 ms | 63.41 s | 29.3 ms | 3.9 ms | 5.01 s | 124.20 s |
| notice | 14 | 16.9 ms | 6.32 s | 15.9 ms | 6.31 s | 113.0 ms | 123.1 ms | 782.2 ms | 11.62 s |
| statement | 2 | 0.75 ms | 86.58 s | 0.34 ms | 86.57 s | 1.3 ms | 14.8 ms | 8.91 s | 171.30 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 233 MB | 1281 MB | real4-letter-13 |
| Model server | 873 MB | 873 MB | real4-office-13 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (36)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report from Paula Gerfen.pdf` · trap: party Paula Gerfen (addressee, an officer of the licensee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Minister for Social Development.pdf` · trap: party Minister for Social Development (minister who transferred the request) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter from Jess Hewat.pdf` · trap: party Jess Hewat (manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-05**: expected `2024-08-23 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-08-23 Letter to Ministry for Pacific Peoples.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-06**: expected `2024-10-04 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-10-04 Letter to s9(2)(a).pdf` · review: TYPE_MISSING, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `letter to Mr. Malamud.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-08**: expected `2015-04-30 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-04-01 Letter to Solicitor of Labor.pdf` · trap: date 2015-04-01 (date of the request); party Solicitor of Labor (appeal authority mentioned) · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-09-08 Letter to Carl Malamud.pdf` · trap: party Carl Malamud (addressee and signer of the attached request) · review: TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-10**: expected `2015-06-11 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Letter to Mr. Carl Malamud Public.Resource.Org.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-13 letter to Kansas Judicial Council.pdf` · trap: date 2021-05-13 (date of the letter being answered); party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real4-letter-13**: expected `2016-05-18 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2016-05-18 Letter to Twanna L. Duncan.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-07-05 Letter to Timothy M. Brown.pdf` · trap: party Timothy M. Brown (chief who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-01**: expected `2026-01-02 Notice of Appeal between State of California and U.S. Department of Transportation.pdf`, got `2026-01-02 Notice of Appeal for Charles C. Calenda.pdf` · trap: party Charles C. Calenda (United States Attorney) · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-02**: expected `2026-02-17 Notice of Appeal between City of Philadelphia and United States Department of the Interior.pdf`, got `2026-02-17 Notice of Appeal for Brett A. Shumate.pdf` · trap: party Brett A. Shumate (Assistant Attorney General listed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for Brett A. Shumate.pdf` · trap: party Brett A. Shumate (Assistant Attorney General listed) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-04**: expected `2026-05-01 Notice of Appeal between United States of America and Tim Walz.pdf`, got `2026-05-01 Notice of Appeal between United States of America and CIVIL No. 25-cv-02668-KMM-DTS.pdf` · review: LOW_CONFIDENCE
- **real4-notice-05**: expected `2025-05-23 Notice of Violation for POINT Biopharma USA Inc.pdf`, got `2025-05-21 Letter to POINT Biopharma USA Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-notice-06**: expected `2025-12-15 Notice of Violation for Tennessee Valley Authority.pdf`, got `2025-12-15 Assessment Followup Letter from NRC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-12**: expected `2025-09-04 Notice of Civil Penalty Assessment and Order to Crown Enterprises, LLC.pdf`, got `2025-08-29 Letter to Crown Enterprises, LLC.pdf` · trap: date 2025-08-29 (date DEQ learned of the service error) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-13**: expected `2025-06-17 Notice of Civil Penalty Assessment and Order to City of Stanfield.pdf`, got `2025-06-17 Letter to Tina Kotek.pdf` · trap: party Tina Kotek (Governor named on the letterhead) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-14**: expected `2025-10-28 Notice of Civil Penalty Assessment and Order to Erickson Incorporated.pdf`, got `2023-02-15 Letter from Department of Environmental Quality.pdf` · review: TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `Requester Invoice from Sam Brownell.pdf` · trap: party Sam Brownell (requester, a person at the customer company) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-03**: expected `2018-10-23 Invoice from PredPol, Inc.pdf`, got `Invoice from PredPol, Inc.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice from WTTG.pdf` · review: DATE_IS_PERIOD, LOW_CONFIDENCE
- **real4-invoice-11**: expected `2024-06-30 Invoice from WCCO-TV.pdf`, got `2024-06-30 Invoice from WCCO-TV Advertiser.pdf` · review: DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `Invoice from WCCO-TV Advertiser.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-office-01**: expected `2018-06-21 Letter to Duquesne Light Company.docx`, got `2018-06-21 Letter to DJ.docx` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Letter to Rosemary Chiavetta.docx` · trap: party Rosemary Chiavetta (Secretary who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-03**: expected `2021-11-29 Letter to Post and Schell.doc`, got `2021-11-29 Letter to Commonwealth of Pennsylvania.doc` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-06**: expected `2021-02-01 Freedom of Information Request Response from Highways England.odt`, got `Letter to Historical Railways Estate.odt` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-02-18 Letter - Freedom of Information Request.docx` · trap: date 2016-02-18 (date of the request e-mail) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-09**: expected `2019-09-16 Freedom of Information Act Response from Office of Manpower Economics.docx`, got `Letter to Your request for information dated 9 September, 2019- Freedom of Information Act.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Email to De la Motte, Leann.msg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-12**: expected `2026-10-19 Public Information Statement from National Weather Service Wichita KS.txt`, got `2026-10-19 Document - National Weather Service Wichita KS 710 PM CDT Mon Oct 19 2026.txt` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-13**: expected `2026-10-19 Special Weather Statement from National Weather Service Jackson KY.txt`, got `2026-10-19 Special Weather Statement.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
