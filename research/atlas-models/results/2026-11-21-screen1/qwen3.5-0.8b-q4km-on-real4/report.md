# InternBench: live run

- **Run:** 2026-11-21T04:51:49Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 54 documents · 54 completed · gold `1abcda1810`
- **Wall time:** 684.34 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 21/51 (41.2%) |
| Document type | 52/54 (96.3%) |
| Date | 36/51 (70.6%) |
| Date role (when the reviewed date was chosen) | 14/35 (40.0%) |
| Parties | 30/54 (55.6%) |
| Relation word (when parties were named) | 40/51 (78.4%) |
| Party in the right role (when parties were named) | 8/24 (33.3%) |
| Ready / review routing | 5/37 (13.5%) |
| Description states nothing false | 54/54 (100.0%) |
| Description is specific | 40/54 (74.1%) |
| Description specificity | 69.1% (mean of 54) |
| Evidence recall (model's quotes) | 13.9% (mean of 54) |
| Digest recall (facts reaching the digest) | 93.5% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 91.0% (mean of 54) |
| Sent to review | 100.0% of 54 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 61.5% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 66.7% | 54 |
| Review rate | 100.0% | 54 |
| Evidence recall | 13.9% | 54 |
| Total latency p50 | 7.69 s | 54 |
| Total latency p95 | 71.79 s | 54 |
| Generation latency p50 | 3.40 s | 54 |
| Generation latency p95 | 4.67 s | 54 |
| Generated tokens p50 | 96 | 54 |
| Generated tokens p95 | 125 | 54 |
| Prompt tokens p50 | 1143 | 54 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 51 |
| Trap date chosen | 6 of 54 |
| Forbidden party named | 8 of 54 |
| Spurious parties named | 20 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 139 (0.0%) |
| Right name sent to review anyway | 16 of 32 |
| Sent to review for an unsupported fact | 36 of 54 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 13 | 6/12 | 6/12 | 11/13 | 1/12 | 100.0% | 7.17 s | 9.05 s |
| letter | 25 | 9/24 | 19/24 | 12/25 | 2/12 | 100.0% | 7.10 s | 71.79 s |
| notice | 14 | 6/13 | 10/13 | 7/14 | 1/12 | 100.0% | 8.48 s | 18.67 s |
| statement | 2 | 0/2 | 1/2 | 0/2 | 1/1 | 100.0% | 5.71 s | 86.42 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 31 | 12/30 | 22/30 | 19/31 | 1/25 | 100.0% | 7.69 s | 71.79 s |
| office | 11 | 4/9 | 7/9 | 5/11 | 2/3 | 100.0% | 5.06 s | 9.14 s |
| scan | 10 | 5/10 | 7/10 | 6/10 | 2/9 | 100.0% | 17.76 s | 86.42 s |
| text | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 5.71 s | 8.82 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 5/18 | 11/18 | 7/20 | 2/10 | 100.0% | 5.07 s | 8.82 s |
| 2-4 | 15 | 6/14 | 10/14 | 9/15 | 2/10 | 100.0% | 7.10 s | 12.85 s |
| 5-9 | 6 | 2/6 | 3/6 | 5/6 | 0/6 | 100.0% | 7.91 s | 11.06 s |
| 10-24 | 10 | 7/10 | 9/10 | 8/10 | 0/9 | 100.0% | 11.36 s | 27.84 s |
| 25-49 | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 100.0% | 71.79 s | 86.42 s |
| 100+ | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 84.13 s | 84.13 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 8/13 | 12/13 | 9/13 | 1/11 | 0 of 13 | 17.78 s | 86.42 s | 3.94 s | 4.93 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 2/3 | 3/3 | 2/3 | 0 |
| addressee_redacted | 6 | 0/6 | 5/6 | 1/6 | 3 |
| agency_billing | 8 | 4/8 | 4/8 | 8/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachments | 15 | 8/15 | 14/15 | 10/15 | 3 |
| body_recounts_events | 1 | – | – | 1/1 | 0 |
| born_digital | 31 | 12/30 | 22/30 | 19/31 | 10 |
| brand_vs_legal_name | 2 | 2/2 | 2/2 | 2/2 | 0 |
| broadcast_invoice | 8 | 4/8 | 4/8 | 8/8 | 1 |
| broken_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption | 4 | 0/4 | 3/4 | 0/4 | 2 |
| card_statement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| certificate_of_service | 2 | 0/2 | 1/2 | 0/2 | 2 |
| certified_mail | 4 | 4/4 | 4/4 | 4/4 | 0 |
| citation | 2 | 1/2 | 2/2 | 1/2 | 0 |
| competing_dates | 43 | 18/43 | 30/43 | 25/43 | 12 |
| contract_numbers | 1 | 0/1 | 1/1 | 0/1 | 0 |
| court_filing | 4 | 0/4 | 3/4 | 0/4 | 2 |
| cover_letter | 8 | 6/8 | 7/8 | 7/8 | 2 |
| customer_first | 1 | 0/1 | 0/1 | 0/1 | 1 |
| customer_redacted | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_stamp | 3 | 0/3 | 0/3 | 1/3 | 2 |
| deadline_in_body | 7 | 5/7 | 7/7 | 5/7 | 0 |
| doc | 3 | 1/2 | 2/2 | 2/3 | 0 |
| docket_number | 3 | 2/3 | 3/3 | 2/3 | 0 |
| docx | 4 | 2/4 | 2/4 | 2/4 | 1 |
| doubled_ocr_text | 4 | 4/4 | 4/4 | 4/4 | 0 |
| due_date | 3 | 2/3 | 2/3 | 3/3 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| embedded_second_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enclosure_list | 1 | – | – | 1/1 | 0 |
| esignature | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fee_invoice | 2 | 0/1 | 0/1 | 0/2 | 2 |
| file_name_date_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filed_stamp | 4 | 0/4 | 3/4 | 0/4 | 2 |
| foi_release | 1 | 0/1 | 1/1 | 0/1 | 0 |
| foi_response | 15 | 3/14 | 9/14 | 6/15 | 6 |
| forecaster_initials | 1 | 0/1 | 0/1 | 0/1 | 0 |
| form | 2 | 1/2 | 2/2 | 1/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 0 |
| inspection_report | 4 | 2/4 | 3/4 | 3/4 | 2 |
| internal_email | 2 | 1/2 | 2/2 | 1/2 | 0 |
| invoice | 13 | 6/12 | 6/12 | 11/13 | 3 |
| issuer_in_logo | 1 | 0/1 | 0/1 | 0/1 | 1 |
| journal_number | 1 | – | – | 0/1 | 0 |
| journalist_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 25 | 9/24 | 19/24 | 12/25 | 8 |
| letterhead_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 9 | 4/9 | 5/9 | 8/9 | 1 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 0 |
| long_subject | 1 | 0/1 | 0/1 | 1/1 | 1 |
| many_dates | 1 | – | – | 1/1 | 0 |
| metadata_date_trap | 3 | 1/2 | 2/2 | 1/3 | 1 |
| military_date_format | 1 | 0/1 | 1/1 | 0/1 | 1 |
| modern | 54 | 21/51 | 36/51 | 30/54 | 14 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 0 |
| multiple_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_documents_in_file | 1 | 0/1 | 1/1 | 0/1 | 0 |
| named_person_at_org | 19 | 10/19 | 15/19 | 11/19 | 7 |
| new_zealand | 4 | 0/4 | 4/4 | 1/4 | 2 |
| no_addressee | 2 | 0/1 | 0/1 | 1/2 | 0 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 0 |
| no_letterhead_text | 2 | 1/2 | 2/2 | 1/2 | 1 |
| no_signer_name | 1 | 0/1 | 1/1 | 0/1 | 0 |
| nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 14 | 6/13 | 10/13 | 7/14 | 3 |
| notice_of_appeal | 4 | 0/4 | 3/4 | 0/4 | 2 |
| notice_of_violation | 2 | 1/2 | 1/2 | 2/2 | 1 |
| numeric_date_dmy | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_text_layer | 8 | 5/8 | 6/8 | 5/8 | 3 |
| odt | 1 | 0/1 | 1/1 | 0/1 | 0 |
| office | 11 | 4/9 | 7/9 | 5/11 | 1 |
| on_behalf_of_client | 2 | 1/2 | 2/2 | 1/2 | 0 |
| ordinal_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| pay_gov | 1 | 0/1 | 0/1 | 0/1 | 1 |
| payment_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 6 | 5/6 | 6/6 | 5/6 | 0 |
| person_in_official_capacity | 3 | 0/3 | 3/3 | 0/3 | 0 |
| plaintiff_appeals | 1 | 0/1 | 1/1 | 0/1 | 0 |
| political_file | 8 | 4/8 | 4/8 | 8/8 | 1 |
| preliminary_data | 1 | 0/1 | 0/1 | 0/1 | 0 |
| proration_note | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 1/3 | 1/3 | 1/3 | 2 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 0/1 | 1/1 | 0/1 | 0 |
| redacted_signer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| reference_number | 9 | 1/8 | 6/8 | 4/9 | 2 |
| referral | 1 | 0/1 | 0/1 | 0/1 | 0 |
| remit_to_bank | 2 | 1/2 | 1/2 | 2/2 | 0 |
| rtf | 1 | – | – | 0/1 | 0 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 10 | 5/10 | 7/10 | 6/10 | 3 |
| service_period | 10 | 5/10 | 5/10 | 10/10 | 1 |
| short | 5 | 2/4 | 4/4 | 2/5 | 1 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 6 | 3/6 | 3/6 | 4/6 | 4 |
| statement | 2 | 0/2 | 1/2 | 0/2 | 0 |
| system_generated | 1 | – | – | 0/1 | 1 |
| tabular_body | 1 | 0/1 | 0/1 | 0/1 | 0 |
| text | 2 | 0/2 | 0/2 | 0/2 | 0 |
| title_line | 2 | 0/2 | 2/2 | 1/2 | 0 |
| two_digit_year | 8 | 4/8 | 4/8 | 8/8 | 1 |
| txt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| uk | 7 | 1/5 | 3/5 | 2/7 | 1 |
| undated | 3 | – | – | 1/3 | 1 |
| us | 8 | 4/8 | 4/8 | 8/8 | 1 |
| us_city | 2 | 2/2 | 2/2 | 2/2 | 0 |
| us_federal | 16 | 5/15 | 8/15 | 7/16 | 7 |
| us_federal_court | 4 | 0/4 | 3/4 | 0/4 | 2 |
| us_state | 12 | 8/12 | 11/12 | 9/12 | 1 |
| utc_vs_local | 2 | 0/2 | 0/2 | 0/2 | 0 |
| vendor_invoice | 3 | 2/3 | 2/3 | 3/3 | 0 |
| weather_product | 2 | 0/2 | 0/2 | 0/2 | 0 |
| wmo_header | 2 | 0/2 | 0/2 | 0/2 | 0 |
| zero_amount | 1 | – | – | 0/1 | 1 |
| zone_list | 1 | 0/1 | 0/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 106 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 23 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 54 | 7.69 s | 71.79 s | 86.42 s |
| `extraction_wall_ms` | 54 | 10.7 ms | 56.53 s | 76.96 s |
| `worker_total_ms` | 54 | 9.9 ms | 56.53 s | 76.95 s |
| `worker_snapshot_ms` | 54 | 0.23 ms | 0.64 ms | 3.3 ms |
| `worker_parse_ms` | 54 | 3.4 ms | 87.9 ms | 1.53 s |
| `worker_analysis_ms` | 54 | 0.57 ms | 48.9 ms | 91.0 ms |
| `worker_render_ms` | 54 | 0.00 ms | 4.05 s | 10.38 s |
| `worker_image_decode_ms` | 54 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 54 | 0.00 ms | 110.55 s | 152.68 s |
| `worker_ocr_encode_ms` | 54 | 0.00 ms | 4.99 s | 7.87 s |
| `worker_ocr_engine_ms` | 54 | 0.00 ms | 102.52 s | 147.41 s |
| `worker_vision_ms` | 54 | 0.00 ms | 0.00 ms | 28.1 ms |
| `analyze_wall_ms` | 54 | 7.30 s | 13.68 s | 15.86 s |
| `distill_ms` | 54 | 1.9 ms | 53.6 ms | 64.6 ms |
| `prompt_ms` | 54 | 0.04 ms | 0.08 ms | 0.09 ms |
| `inference_ms` | 54 | 7.29 s | 13.60 s | 15.74 s |
| `prefill_ms` | 54 | 3.81 s | 9.53 s | 11.86 s |
| `generation_ms` | 54 | 3.40 s | 4.67 s | 6.16 s |
| `validation_ms` | 54 | 6.4 ms | 28.9 ms | 58.4 ms |
| `naming_ms` | 54 | 0.20 ms | 3.4 ms | 4.3 ms |
| `index_ms` | 54 | 1.7 ms | 50.5 ms | 59.0 ms |
| `retrieval_ms` | 54 | 0.04 ms | 3.3 ms | 4.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 54 | 281.8 tok/s | 311.1 tok/s | 314.5 tok/s |
| `generation_tok_per_s` | 54 | 27.2 tok/s | 30.3 tok/s | 31.0 tok/s |
| `prompt_tokens` | 54 | 1143 | 2503 | 3121 |
| `cached_tokens` | 54 | 477 | 477 | 546 |
| `generated_tokens` | 54 | 96 | 125 | 165 |
| `estimated_prompt_tokens` | 54 | 1596 | 2878 | 3398 |
| `model_requests` | 54 | 1 | 1 | 1 |
| `redistillations` | 54 | 0 | 0 | 0 |
| `source_characters` | 54 | 2982 | 75251 | 87914 |
| `digest_characters` | 54 | 2847 | 6325 | 7031 |
| `prompt_characters` | 54 | 4896 | 8380 | 9087 |
| `worker_ocr_pages` | 54 | 0 | 29 | 66 |
| `worker_ocr_passes` | 54 | 0 | 29 | 66 |
| `worker_orientation_passes` | 54 | 0 | 29 | 66 |
| `worker_rendered_pixels` | 54 | 0 | 252367280 | 574406515 |
| `index_units` | 54 | 34 | 1694 | 1903 |
| `context_units` | 54 | 34 | 98 | 122 |
| `context_tokens` | 54 | 1008 | 2288 | 2806 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 18 | 1.9 ms | 1.53 s | 1.6 ms | 1.53 s | 1.53 s | 0.41 ms | 0.00 ms | 0.00 ms |
| layout | 23 | 10.7 ms | 32.3 ms | 9.9 ms | 31.0 ms | 26.1 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 1.06 s | 1.38 s | 1.05 s | 1.38 s | 11.9 ms | 0.79 ms | 104.3 ms | 1.23 s |
| ocr | 11 | 5.32 s | 76.96 s | 5.32 s | 76.95 s | 88.7 ms | 91.0 ms | 10.38 s | 152.68 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 3.0 ms | 1.06 s | 2.6 ms | 1.05 s | 11.2 ms | 0.99 ms | 0.00 ms | 0.00 ms |
| 2-4 | 15 | 5.4 ms | 4.45 s | 5.0 ms | 4.45 s | 21.4 ms | 3.8 ms | 419.6 ms | 6.92 s |
| 5-9 | 6 | 17.7 ms | 30.4 ms | 16.0 ms | 27.7 ms | 16.8 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 41.7 ms | 17.19 s | 40.1 ms | 17.18 s | 88.7 ms | 91.0 ms | 1.20 s | 33.25 s |
| 25-49 | 2 | 56.53 s | 76.96 s | 56.53 s | 76.95 s | 3.0 ms | 11.4 ms | 7.18 s | 152.68 s |
| 100+ | 1 | 68.27 s | 68.27 s | 68.26 s | 68.26 s | 23.5 ms | 11.7 ms | 10.38 s | 135.51 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 13 | 6.0 ms | 30.4 ms | 5.6 ms | 27.7 ms | 16.8 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 11.3 ms | 56.53 s | 11.0 ms | 56.53 s | 23.5 ms | 3.2 ms | 4.05 s | 110.55 s |
| notice | 14 | 13.7 ms | 5.50 s | 12.8 ms | 5.49 s | 88.7 ms | 91.0 ms | 641.2 ms | 10.16 s |
| statement | 2 | 0.50 ms | 76.96 s | 0.26 ms | 76.95 s | 1.1 ms | 11.4 ms | 7.18 s | 152.68 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 270 MB | 1237 MB | real4-letter-09 |
| Model server | 876 MB | 876 MB | real4-office-13 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (30)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report from Paula Gerfen.pdf` · trap: party Paula Gerfen (addressee, an officer of the licensee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Kaiwhakaoranga Service.pdf` · trap: party Kaiwhakaoranga Service (subject of the request, not a party) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter from Jess Hewat.pdf` · trap: party Jess Hewat (manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-05**: expected `2024-08-23 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-08-23 Letter to Ministry for Pacific Peoples.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-06**: expected `2024-10-04 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-10-04 Letter to s9(2)(a).pdf` · review: TYPE_MISSING, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Letter.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real4-letter-08**: expected `2015-04-30 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-04-01 Letter to Amanda L. Edens.pdf` · trap: date 2015-04-01 (date of the request); party Amanda L. Edens (director who signs) · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-09-08 Letter to Carl Malamud.pdf` · trap: party Carl Malamud (addressee and signer of the attached request) · review: TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter to Kansas Judicial Council.pdf` · trap: party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Letter from U.S. Coast Guard.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-01**: expected `2026-01-02 Notice of Appeal between State of California and U.S. Department of Transportation.pdf`, got `2025-11-04 Notice of Appeal for United States.pdf` · trap: date 2025-11-04 (date of the judgment appealed from); party United States Court of Appeals for the First Circuit (court appealed to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-02**: expected `2026-02-17 Notice of Appeal between City of Philadelphia and United States Department of the Interior.pdf`, got `2026-02-17 Notice of Appeal for Doug Burgum.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for JOE NEGUSE.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-04**: expected `2026-05-01 Notice of Appeal between United States of America and Tim Walz.pdf`, got `2026-05-01 Notice of Appeal between United States of America and CIVIL No. 25-cv-02668-KMM-DTS.pdf` · review: LOW_CONFIDENCE
- **real4-notice-06**: expected `2025-12-15 Notice of Violation for Tennessee Valley Authority.pdf`, got `2025-09-25 Notice of Violation for Tennessee Valley Authority.pdf` · trap: date 2025-09-25 (date of the inspection exit meeting) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real4-notice-11**: expected `2024-02-20 Citation and Notification of Penalty to Henry Brick Company, Inc.pdf`, got `2024-02-20 Citation and Notification of Penalty for Jose A. Gonzalez.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `Requester Invoice from Sam Brownell.pdf` · trap: party Sam Brownell (requester, a person at the customer company) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-03**: expected `2018-10-23 Invoice from PredPol, Inc.pdf`, got `Invoice from PredPol, Inc.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice from WTTG.pdf` · review: DATE_IS_PERIOD, LOW_CONFIDENCE
- **real4-invoice-11**: expected `2024-06-30 Invoice from WCCO-TV.pdf`, got `Invoice from WCCO-TV Advertiser.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `Invoice from WCCO.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-13**: expected `2010-04-28 Statement from Barclaycard Commercial.pdf`, got `2010-04-28 Statement.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, PARTY_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-office-03**: expected `2021-11-29 Letter to Post and Schell.doc`, got `2021-11-29 Letter to Commonwealth Keystone Building.doc` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-06**: expected `2021-02-01 Freedom of Information Request Response from Highways England.odt`, got `2021-02-01 Letter to Freedom of Information Request – Bridge infill letters.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-02-18 Letter from Freedom of Information Request.docx` · trap: date 2016-02-18 (date of the request e-mail) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-09**: expected `2019-09-16 Freedom of Information Act Response from Office of Manpower Economics.docx`, got `Letter from Fleetbank House.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Email to De la Motte, Leann.msg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-12**: expected `2026-10-19 Public Information Statement from National Weather Service Wichita KS.txt`, got `Document from PNSICT.txt` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real4-office-13**: expected `2026-10-19 Special Weather Statement from National Weather Service Jackson KY.txt`, got `Document - Fleming-Montgomery-Bath-Rowan-Estill-Powell-Menifee-Rockcastle- Jackson-Pulaski-.txt` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
