# InternBench: live run

- **Run:** 2026-11-21T04:53:52Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 53 documents · 53 completed · gold `174e2d20ea`
- **Wall time:** 979.59 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 22/50 (44.0%) |
| Document type | 48/53 (90.6%) |
| Date | 40/50 (80.0%) |
| Date role (when the reviewed date was chosen) | 16/40 (40.0%) |
| Parties | 34/53 (64.1%) |
| Relation word (when parties were named) | 34/49 (69.4%) |
| Party in the right role (when parties were named) | 6/20 (30.0%) |
| Ready / review routing | 8/36 (22.2%) |
| Description states nothing false | 52/53 (98.1%) |
| Description is specific | 45/53 (84.9%) |
| Description specificity | 74.2% (mean of 53) |
| Evidence recall (model's quotes) | 33.3% (mean of 53) |
| Digest recall (facts reaching the digest) | 93.4% (mean of 53) |
| Prompt recall (facts reaching the prompt sent) | 90.9% (mean of 53) |
| Sent to review | 92.5% of 53 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 61.5% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 69.8% | 53 |
| Review rate | 92.5% | 53 |
| Evidence recall | 33.3% | 53 |
| Total latency p50 | 13.23 s | 53 |
| Total latency p95 | 79.40 s | 53 |
| Generation latency p50 | 4.80 s | 53 |
| Generation latency p95 | 6.21 s | 53 |
| Generated tokens p50 | 89 | 53 |
| Generated tokens p95 | 111 | 53 |
| Prompt tokens p50 | 999 | 53 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 3 of 50 |
| Trap date chosen | 5 of 53 |
| Forbidden party named | 11 of 53 |
| Spurious parties named | 14 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 171 (0.6%) |
| Right name sent to review anyway | 13 of 31 |
| Sent to review for an unsupported fact | 37 of 53 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 12 | 6/11 | 8/11 | 7/12 | 2/11 | 91.7% | 11.93 s | 14.44 s |
| letter | 25 | 6/24 | 18/24 | 15/25 | 3/12 | 92.0% | 12.73 s | 79.40 s |
| notice | 14 | 9/13 | 13/13 | 10/14 | 2/12 | 92.9% | 15.69 s | 27.93 s |
| statement | 2 | 1/2 | 1/2 | 2/2 | 1/1 | 100.0% | 8.85 s | 90.78 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 30 | 14/29 | 26/29 | 17/30 | 3/24 | 93.3% | 13.41 s | 79.40 s |
| office | 11 | 3/9 | 7/9 | 8/11 | 3/3 | 81.8% | 8.88 s | 19.07 s |
| scan | 10 | 3/10 | 5/10 | 7/10 | 2/9 | 100.0% | 26.55 s | 90.78 s |
| text | 2 | 2/2 | 2/2 | 2/2 | – | 100.0% | 8.85 s | 12.40 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 6/17 | 15/17 | 10/19 | 3/9 | 89.5% | 8.70 s | 14.08 s |
| 2-4 | 15 | 6/14 | 11/14 | 10/15 | 3/10 | 93.3% | 12.73 s | 19.07 s |
| 5-9 | 6 | 2/6 | 3/6 | 4/6 | 0/6 | 100.0% | 13.80 s | 21.74 s |
| 10-24 | 10 | 7/10 | 9/10 | 8/10 | 1/9 | 90.0% | 19.47 s | 35.94 s |
| 25-49 | 2 | 0/2 | 1/2 | 1/2 | 1/2 | 100.0% | 79.40 s | 90.78 s |
| 100+ | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 94.90 s | 94.90 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 8/13 | 11/13 | 10/13 | 2/11 | 1 of 13 | 26.69 s | 94.90 s | 5.90 s | 6.45 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 0/3 | 3/3 | 2/3 | 1 |
| addressee_redacted | 6 | 2/6 | 5/6 | 3/6 | 3 |
| agency_billing | 8 | 5/8 | 5/8 | 6/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| attachments | 15 | 8/15 | 14/15 | 12/15 | 2 |
| body_recounts_events | 1 | – | – | 1/1 | 1 |
| born_digital | 30 | 14/29 | 26/29 | 17/30 | 9 |
| brand_vs_legal_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| broadcast_invoice | 8 | 5/8 | 5/8 | 6/8 | 1 |
| broken_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption | 4 | 2/4 | 4/4 | 2/4 | 1 |
| card_statement | 1 | 0/1 | 0/1 | 1/1 | 0 |
| certificate_of_service | 2 | 1/2 | 2/2 | 1/2 | 1 |
| certified_mail | 4 | 3/4 | 4/4 | 4/4 | 0 |
| citation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 42 | 21/42 | 33/42 | 28/42 | 12 |
| contract_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| court_filing | 4 | 2/4 | 4/4 | 2/4 | 1 |
| cover_letter | 8 | 5/8 | 8/8 | 7/8 | 1 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_redacted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_stamp | 3 | 0/3 | 0/3 | 2/3 | 1 |
| deadline_in_body | 7 | 6/7 | 7/7 | 7/7 | 0 |
| doc | 3 | 0/2 | 1/2 | 3/3 | 2 |
| docket_number | 3 | 0/3 | 3/3 | 2/3 | 1 |
| docx | 4 | 1/4 | 3/4 | 2/4 | 2 |
| doubled_ocr_text | 4 | 3/4 | 4/4 | 4/4 | 0 |
| due_date | 2 | 1/2 | 2/2 | 1/2 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 1/2 | 2/2 | 2/2 | 0 |
| embedded_second_letter | 1 | 0/1 | 0/1 | 0/1 | 0 |
| enclosure_list | 1 | – | – | 1/1 | 1 |
| esignature | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fee_invoice | 2 | 0/1 | 1/1 | 0/2 | 1 |
| file_name_date_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| filed_stamp | 4 | 2/4 | 4/4 | 2/4 | 1 |
| foi_release | 1 | 0/1 | 0/1 | 1/1 | 0 |
| foi_response | 15 | 3/14 | 8/14 | 8/15 | 8 |
| forecaster_initials | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 2/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| image_only | 2 | 0/2 | 0/2 | 2/2 | 0 |
| inspection_report | 4 | 2/4 | 4/4 | 3/4 | 1 |
| internal_email | 2 | 1/2 | 2/2 | 2/2 | 0 |
| invoice | 12 | 6/11 | 8/11 | 7/12 | 2 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| journal_number | 1 | – | – | 0/1 | 1 |
| journalist_addressee | 1 | 0/1 | 0/1 | 1/1 | 1 |
| letter | 25 | 6/24 | 18/24 | 15/25 | 11 |
| letterhead_names | 1 | 0/1 | 1/1 | 0/1 | 1 |
| line_item_dates | 9 | 5/9 | 5/9 | 7/9 | 1 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 1 |
| long_subject | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | – | – | 1/1 | 1 |
| metadata_date_trap | 3 | 0/2 | 2/2 | 1/3 | 2 |
| military_date_format | 1 | 0/1 | 0/1 | 0/1 | 0 |
| modern | 53 | 22/50 | 40/50 | 34/53 | 16 |
| msg | 2 | 1/2 | 2/2 | 2/2 | 0 |
| multiple_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_documents_in_file | 1 | 0/1 | 0/1 | 1/1 | 0 |
| named_person_at_org | 19 | 4/19 | 15/19 | 10/19 | 8 |
| new_zealand | 4 | 1/4 | 4/4 | 2/4 | 2 |
| no_addressee | 2 | 1/1 | 1/1 | 2/2 | 1 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 1 |
| no_letterhead_text | 2 | 0/2 | 2/2 | 0/2 | 2 |
| no_signer_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 14 | 9/13 | 13/13 | 10/14 | 3 |
| notice_of_appeal | 4 | 2/4 | 4/4 | 2/4 | 1 |
| notice_of_violation | 2 | 1/2 | 2/2 | 1/2 | 1 |
| numeric_date_dmy | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_text_layer | 8 | 3/8 | 5/8 | 5/8 | 2 |
| odt | 1 | 1/1 | 1/1 | 1/1 | 0 |
| office | 11 | 3/9 | 7/9 | 8/11 | 5 |
| on_behalf_of_client | 2 | 0/2 | 2/2 | 1/2 | 1 |
| ordinal_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pay_gov | 1 | 0/1 | 1/1 | 0/1 | 0 |
| payment_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 6 | 5/6 | 6/6 | 6/6 | 0 |
| person_in_official_capacity | 3 | 2/3 | 3/3 | 2/3 | 0 |
| plaintiff_appeals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| political_file | 8 | 5/8 | 5/8 | 6/8 | 1 |
| preliminary_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 0/3 | 1/3 | 1/3 | 2 |
| re_line_references_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| redacted_signer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| reference_number | 9 | 3/8 | 7/8 | 6/9 | 4 |
| referral | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_to_bank | 2 | 1/2 | 2/2 | 1/2 | 0 |
| rtf | 1 | – | – | 0/1 | 1 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 10 | 3/10 | 5/10 | 7/10 | 2 |
| service_period | 10 | 6/10 | 7/10 | 7/10 | 1 |
| short | 5 | 0/4 | 4/4 | 2/5 | 3 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 0/1 | 0/1 | 1/1 | 1 |
| signature_on_later_page | 6 | 1/6 | 4/6 | 2/6 | 4 |
| statement | 2 | 1/2 | 1/2 | 2/2 | 0 |
| system_generated | 1 | – | – | 0/1 | 1 |
| tabular_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 2 | 2/2 | 2/2 | 2/2 | 0 |
| title_line | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_digit_year | 8 | 5/8 | 5/8 | 6/8 | 1 |
| txt | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk | 7 | 2/5 | 2/5 | 5/7 | 4 |
| undated | 3 | – | – | 1/3 | 3 |
| us | 8 | 5/8 | 5/8 | 6/8 | 1 |
| us_city | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_federal | 16 | 6/15 | 11/15 | 9/16 | 5 |
| us_federal_court | 4 | 2/4 | 4/4 | 2/4 | 1 |
| us_state | 12 | 4/12 | 12/12 | 8/12 | 3 |
| utc_vs_local | 2 | 2/2 | 2/2 | 2/2 | 0 |
| vendor_invoice | 2 | 1/2 | 2/2 | 1/2 | 0 |
| weather_product | 2 | 2/2 | 2/2 | 2/2 | 0 |
| wmo_header | 2 | 2/2 | 2/2 | 2/2 | 0 |
| zero_amount | 1 | – | – | 0/1 | 1 |
| zone_list | 1 | 1/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 105 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 22 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 53 | 13.23 s | 79.40 s | 94.90 s |
| `extraction_wall_ms` | 53 | 11.7 ms | 52.57 s | 75.07 s |
| `worker_total_ms` | 53 | 11.2 ms | 52.57 s | 75.07 s |
| `worker_snapshot_ms` | 53 | 0.23 ms | 0.72 ms | 3.0 ms |
| `worker_parse_ms` | 53 | 3.8 ms | 86.9 ms | 2.03 s |
| `worker_analysis_ms` | 53 | 0.71 ms | 52.9 ms | 83.4 ms |
| `worker_render_ms` | 53 | 0.00 ms | 3.91 s | 10.34 s |
| `worker_image_decode_ms` | 53 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 53 | 0.00 ms | 103.00 s | 148.88 s |
| `worker_ocr_encode_ms` | 53 | 0.00 ms | 5.30 s | 7.85 s |
| `worker_ocr_engine_ms` | 53 | 0.00 ms | 94.90 s | 143.33 s |
| `worker_vision_ms` | 53 | 0.00 ms | 0.00 ms | 20.2 ms |
| `analyze_wall_ms` | 53 | 12.73 s | 23.84 s | 27.34 s |
| `distill_ms` | 53 | 2.0 ms | 53.2 ms | 64.3 ms |
| `prompt_ms` | 53 | 0.04 ms | 0.07 ms | 0.09 ms |
| `inference_ms` | 53 | 12.72 s | 23.73 s | 27.24 s |
| `prefill_ms` | 53 | 8.31 s | 17.38 s | 21.26 s |
| `generation_ms` | 53 | 4.80 s | 6.21 s | 6.45 s |
| `validation_ms` | 53 | 5.2 ms | 26.7 ms | 50.1 ms |
| `naming_ms` | 53 | 0.20 ms | 3.4 ms | 4.3 ms |
| `index_ms` | 53 | 1.7 ms | 49.7 ms | 58.9 ms |
| `retrieval_ms` | 53 | 0.04 ms | 3.3 ms | 4.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 53 | 119.5 tok/s | 122.7 tok/s | 124.4 tok/s |
| `generation_tok_per_s` | 53 | 18.5 tok/s | 19.0 tok/s | 19.2 tok/s |
| `prompt_tokens` | 53 | 999 | 2125 | 2588 |
| `cached_tokens` | 53 | 469 | 469 | 510 |
| `generated_tokens` | 53 | 89 | 111 | 114 |
| `estimated_prompt_tokens` | 53 | 1604 | 2878 | 3398 |
| `model_requests` | 53 | 1 | 1 | 1 |
| `redistillations` | 53 | 0 | 0 | 0 |
| `source_characters` | 53 | 3149 | 75251 | 87914 |
| `digest_characters` | 53 | 2910 | 6325 | 7031 |
| `prompt_characters` | 53 | 4964 | 8380 | 9087 |
| `worker_ocr_pages` | 53 | 0 | 29 | 66 |
| `worker_ocr_passes` | 53 | 0 | 29 | 66 |
| `worker_orientation_passes` | 53 | 0 | 29 | 66 |
| `worker_rendered_pixels` | 53 | 0 | 252367280 | 574406515 |
| `index_units` | 53 | 37 | 1694 | 1903 |
| `context_units` | 53 | 34 | 98 | 122 |
| `context_tokens` | 53 | 1015 | 2288 | 2806 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 18 | 2.2 ms | 2.04 s | 1.7 ms | 2.04 s | 2.03 s | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 22 | 11.7 ms | 31.5 ms | 11.2 ms | 30.4 ms | 25.8 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 1.00 s | 1.30 s | 1.00 s | 1.30 s | 12.0 ms | 0.71 ms | 100.6 ms | 1.16 s |
| ocr | 11 | 5.20 s | 75.07 s | 5.19 s | 75.07 s | 88.8 ms | 83.4 ms | 10.34 s | 148.88 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 3.3 ms | 2.04 s | 2.8 ms | 2.04 s | 2.03 s | 1.0 ms | 87.7 ms | 898.6 ms |
| 2-4 | 15 | 5.1 ms | 4.23 s | 4.7 ms | 4.23 s | 21.5 ms | 3.8 ms | 411.7 ms | 6.60 s |
| 5-9 | 6 | 17.7 ms | 30.3 ms | 16.0 ms | 27.8 ms | 16.7 ms | 11.3 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 40.8 ms | 16.63 s | 39.5 ms | 16.63 s | 88.8 ms | 83.4 ms | 1.19 s | 32.17 s |
| 25-49 | 2 | 52.57 s | 75.07 s | 52.57 s | 75.07 s | 3.8 ms | 11.4 ms | 7.34 s | 148.88 s |
| 100+ | 1 | 67.55 s | 67.55 s | 67.55 s | 67.55 s | 23.5 ms | 11.5 ms | 10.34 s | 134.06 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 12 | 6.2 ms | 30.3 ms | 5.7 ms | 27.8 ms | 16.7 ms | 11.3 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 11.7 ms | 52.57 s | 11.2 ms | 52.57 s | 23.5 ms | 3.1 ms | 3.91 s | 103.00 s |
| notice | 14 | 13.1 ms | 5.38 s | 12.4 ms | 5.38 s | 88.8 ms | 83.4 ms | 629.5 ms | 9.96 s |
| statement | 2 | 0.97 ms | 75.07 s | 0.30 ms | 75.07 s | 1.1 ms | 11.4 ms | 7.34 s | 148.88 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 237 MB | 1275 MB | real4-letter-13 |
| Model server | 4428 MB | 4480 MB | real4-letter-05 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (28)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report for Pacific Gas and Electric Company.pdf` · review: TYPE_INFERRED, LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Laura Berntsen.pdf` · trap: party Laura Berntsen (acting manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter from Jess Hewat.pdf` · trap: party Jess Hewat (manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real4-letter-05**: expected `2024-08-23 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-08-23 Response to an Official Information Act Request - Ministry for Pacific Peoples.pdf` · review: PARTY_UNSUPPORTED
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Document - PUBLIC.RESOURCE.ORG.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-08**: expected `2015-04-30 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Letter from Solicitor of Labor.pdf` · trap: party Solicitor of Labor (appeal authority mentioned) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-06-29 Memorandum for Aflcmcxz from 88 ABWJAC.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-10**: expected `2015-06-11 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-06-11 Letter from Thomas D. Everett.pdf` · trap: party Thomas D. Everett (director who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter to Kansas Judicial Council.pdf` · trap: party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-13**: expected `2016-05-18 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2016-05-18 Letter to Bobbie Parsons.pdf` · trap: party Bobbie Parsons (FOIA officer who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-07-05 Letter from U.S. Coast Guard.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real4-letter-15**: expected `2021-08-27 Letter to Public.Resource.Org.pdf`, got `2021-08-27 Letter from Greg Woosley.pdf` · trap: party Greg Woosley (general counsel who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-01**: expected `2026-01-02 Notice of Appeal between State of California and U.S. Department of Transportation.pdf`, got `2026-01-02 Notice of Appeal for Charles C. Calenda.pdf` · trap: party Charles C. Calenda (United States Attorney) · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for Joe Neguse.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-06**: expected `2025-12-15 Notice of Violation for Tennessee Valley Authority.pdf`, got `2025-12-15 Notice of Violation to Delson Erb.pdf` · trap: party Delson Erb (addressee, licensee officer) · **filed without review**
- **real4-notice-14**: expected `2025-10-28 Notice of Civil Penalty Assessment and Order to Erickson Incorporated.pdf`, got `2025-10-28 Document from Erickson Incorporated.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `2023-10-05 Requester Invoice.pdf` · review: DATE_AMBIGUOUS, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-03**: expected `2018-10-23 Invoice from PredPol, Inc.pdf`, got `2018-10-23 Invoice.pdf` · review: PARTY_MISSING
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice.pdf` · review: DATE_IS_PERIOD, PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `Estimate from WCCO-TV Advertiser.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-13**: expected `2010-04-28 Statement from Barclaycard Commercial.pdf`, got `Statement from Barclaycard Commercial.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from American Petroleum Institute (P) DC 7317.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-office-01**: expected `2018-06-21 Letter to Duquesne Light Company.docx`, got `2018-06-21 Letter of Notification to Duquesne Light Company.docx` · **filed without review**
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Letter of Notification for Christopher T. Wright Esquire.docx` · trap: party Christopher T. Wright (addressee, an attorney at the firm) · review: PARTY_UNSUPPORTED
- **real4-office-03**: expected `2021-11-29 Letter to Post and Schell.doc`, got `2021-11-29 Letter of Notification to Post and Schell.doc` · **filed without review**
- **real4-office-05**: expected `2012-03-29 Freedom of Information Act Response to Daily Telegraph.doc`, got `2012-03-15 Letter to Daily Telegraph.doc` · trap: date 2012-03-15 (date of the request letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-02-18 Letter.docx` · trap: date 2016-02-18 (date of the request e-mail) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DESCRIPTION_INVALID
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Email to Leann.delaMotte@dgs.msg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
