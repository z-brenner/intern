# InternBench: live run

- **Run:** 2026-11-23T17:14:25Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 52 documents · 52 completed · gold `e683b2decf`
- **Wall time:** 1991.02 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 25/49 (51.0%) |
| Document type | 49/52 (94.2%) |
| Date | 43/49 (87.8%) |
| Date role (when the reviewed date was chosen) | 16/43 (37.2%) |
| Parties | 38/52 (73.1%) |
| Relation word (when parties were named) | 39/49 (79.6%) |
| Party in the right role (when parties were named) | 11/21 (52.4%) |
| Ready / review routing | 10/35 (28.6%) |
| Description states nothing false | 52/52 (100.0%) |
| Description is specific | 41/52 (78.8%) |
| Description specificity | 73.1% (mean of 52) |
| Evidence recall (model's quotes) | 68.9% (mean of 52) |
| Digest recall (facts reaching the digest) | 93.3% (mean of 52) |
| Prompt recall (facts reaching the prompt sent) | 90.7% (mean of 52) |
| Sent to review | 88.5% of 52 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 69.2% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 59.6% | 52 |
| Review rate | 88.5% | 52 |
| Evidence recall | 68.9% | 52 |
| Total latency p50 | 31.47 s | 52 |
| Total latency p95 | 114.97 s | 52 |
| Generation latency p50 | 12.88 s | 52 |
| Generation latency p95 | 18.62 s | 52 |
| Generated tokens p50 | 88 | 52 |
| Generated tokens p95 | 126 | 52 |
| Prompt tokens p50 | 1163 | 52 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 49 |
| Trap date chosen | 4 of 52 |
| Forbidden party named | 6 of 52 |
| Spurious parties named | 10 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 161 (0.0%) |
| Right name sent to review anyway | 13 of 30 |
| Sent to review for an unsupported fact | 31 of 52 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 5/10 | 8/10 | 9/11 | 3/10 | 81.8% | 31.00 s | 37.53 s |
| letter | 25 | 10/24 | 20/24 | 15/25 | 2/12 | 100.0% | 31.47 s | 114.97 s |
| notice | 14 | 9/13 | 13/13 | 12/14 | 4/12 | 71.4% | 34.21 s | 59.57 s |
| statement | 2 | 1/2 | 2/2 | 2/2 | 1/1 | 100.0% | 23.62 s | 125.06 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 29 | 15/28 | 26/28 | 22/29 | 5/23 | 86.2% | 31.47 s | 114.97 s |
| office | 11 | 3/9 | 8/9 | 6/11 | 2/3 | 100.0% | 21.43 s | 41.34 s |
| scan | 10 | 5/10 | 7/10 | 8/10 | 3/9 | 90.0% | 55.69 s | 125.06 s |
| text | 2 | 2/2 | 2/2 | 2/2 | – | 50.0% | 23.62 s | 31.27 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 8/17 | 15/17 | 10/19 | 3/9 | 89.5% | 21.43 s | 34.54 s |
| 2-4 | 14 | 6/13 | 12/13 | 9/14 | 3/9 | 92.9% | 31.47 s | 41.34 s |
| 5-9 | 6 | 2/6 | 4/6 | 6/6 | 1/6 | 83.3% | 31.00 s | 50.07 s |
| 10-24 | 10 | 7/10 | 9/10 | 10/10 | 2/9 | 80.0% | 50.01 s | 60.63 s |
| 25-49 | 2 | 1/2 | 2/2 | 2/2 | 1/2 | 100.0% | 114.97 s | 125.06 s |
| 100+ | 1 | 1/1 | 1/1 | 1/1 | – | 100.0% | 135.83 s | 135.83 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 9/13 | 12/13 | 13/13 | 3/11 | 1 of 13 | 56.23 s | 135.83 s | 15.11 s | 22.64 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 1/3 | 3/3 | 1/3 | 0 |
| addressee_redacted | 6 | 2/6 | 5/6 | 3/6 | 3 |
| agency_billing | 8 | 4/8 | 6/8 | 7/8 | 1 |
| attachment_title_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachments | 15 | 11/15 | 14/15 | 15/15 | 0 |
| body_recounts_events | 1 | – | – | 1/1 | 1 |
| born_digital | 29 | 15/28 | 26/28 | 22/29 | 6 |
| brand_vs_legal_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| broadcast_invoice | 8 | 4/8 | 6/8 | 7/8 | 1 |
| broken_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption | 4 | 2/4 | 4/4 | 2/4 | 1 |
| card_statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 2 | 1/2 | 2/2 | 1/2 | 1 |
| certified_mail | 4 | 4/4 | 4/4 | 4/4 | 0 |
| citation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| competing_dates | 41 | 22/41 | 36/41 | 32/41 | 7 |
| contract_numbers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filing | 4 | 2/4 | 4/4 | 2/4 | 1 |
| cover_letter | 8 | 6/8 | 8/8 | 8/8 | 0 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_redacted | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_stamp | 3 | 1/3 | 1/3 | 1/3 | 1 |
| deadline_in_body | 7 | 6/7 | 7/7 | 7/7 | 0 |
| doc | 3 | 0/2 | 2/2 | 1/3 | 1 |
| docket_number | 3 | 1/3 | 3/3 | 1/3 | 0 |
| docx | 4 | 1/4 | 3/4 | 1/4 | 2 |
| doubled_ocr_text | 4 | 4/4 | 4/4 | 4/4 | 0 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 2/2 | 2/2 | 2/2 | 0 |
| embedded_second_letter | 1 | 0/1 | 0/1 | 1/1 | 0 |
| enclosure_list | 1 | – | – | 1/1 | 1 |
| esignature | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fee_invoice | 2 | 0/1 | 1/1 | 1/2 | 1 |
| file_name_date_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filed_stamp | 4 | 2/4 | 4/4 | 2/4 | 1 |
| foi_release | 1 | 0/1 | 1/1 | 1/1 | 0 |
| foi_response | 15 | 4/14 | 10/14 | 8/15 | 6 |
| forecaster_initials | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 1/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 1 |
| inspection_report | 4 | 2/4 | 4/4 | 4/4 | 0 |
| internal_email | 2 | 2/2 | 2/2 | 2/2 | 0 |
| invoice | 11 | 5/10 | 8/10 | 9/11 | 2 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| journal_number | 1 | – | – | 1/1 | 0 |
| journalist_addressee | 1 | 0/1 | 1/1 | 0/1 | 0 |
| letter | 25 | 10/24 | 20/24 | 15/25 | 7 |
| letterhead_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 9 | 4/9 | 7/9 | 8/9 | 1 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| long_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_dates | 1 | – | – | 1/1 | 1 |
| metadata_date_trap | 3 | 1/2 | 2/2 | 2/3 | 1 |
| military_date_format | 1 | 0/1 | 0/1 | 1/1 | 0 |
| modern | 52 | 25/49 | 43/49 | 38/52 | 10 |
| msg | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multiple_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_documents_in_file | 1 | 0/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 19 | 9/19 | 17/19 | 13/19 | 1 |
| new_zealand | 4 | 2/4 | 4/4 | 2/4 | 2 |
| no_addressee | 2 | 0/1 | 1/1 | 1/2 | 2 |
| no_issuer_name_in_header | 1 | – | – | 1/1 | 0 |
| no_letterhead_text | 2 | 0/2 | 2/2 | 0/2 | 1 |
| no_signer_name | 1 | 0/1 | 1/1 | 1/1 | 0 |
| nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 14 | 9/13 | 13/13 | 12/14 | 1 |
| notice_of_appeal | 4 | 2/4 | 4/4 | 2/4 | 1 |
| notice_of_violation | 2 | 1/2 | 2/2 | 2/2 | 0 |
| numeric_date_dmy | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_text_layer | 8 | 5/8 | 6/8 | 7/8 | 0 |
| odt | 1 | 0/1 | 1/1 | 1/1 | 0 |
| office | 11 | 3/9 | 8/9 | 6/11 | 3 |
| on_behalf_of_client | 2 | 0/2 | 2/2 | 0/2 | 0 |
| ordinal_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| pay_gov | 1 | 0/1 | 1/1 | 0/1 | 0 |
| penalty_notice | 6 | 5/6 | 6/6 | 6/6 | 0 |
| person_in_official_capacity | 3 | 1/3 | 3/3 | 1/3 | 1 |
| plaintiff_appeals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| political_file | 8 | 4/8 | 6/8 | 7/8 | 1 |
| preliminary_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 2/3 | 2/3 | 2/3 | 0 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 0/1 | 1/1 | 1/1 | 0 |
| redacted_signer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| reference_number | 9 | 2/8 | 7/8 | 5/9 | 5 |
| referral | 1 | 0/1 | 1/1 | 0/1 | 1 |
| rtf | 1 | – | – | 1/1 | 0 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 10 | 5/10 | 7/10 | 8/10 | 1 |
| service_period | 8 | 4/8 | 6/8 | 7/8 | 1 |
| short | 5 | 1/4 | 4/4 | 2/5 | 1 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 0/1 | 1/1 | 0/1 | 0 |
| signature_on_later_page | 6 | 4/6 | 5/6 | 5/6 | 1 |
| statement | 2 | 1/2 | 2/2 | 2/2 | 0 |
| system_generated | 1 | – | – | 1/1 | 1 |
| tabular_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 2 | 2/2 | 2/2 | 2/2 | 0 |
| title_line | 2 | 2/2 | 2/2 | 2/2 | 0 |
| two_digit_year | 8 | 4/8 | 6/8 | 7/8 | 1 |
| txt | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk | 7 | 0/5 | 4/5 | 4/7 | 3 |
| undated | 3 | – | – | 3/3 | 2 |
| us | 8 | 4/8 | 6/8 | 7/8 | 1 |
| us_city | 1 | 1/1 | 1/1 | 1/1 | 0 |
| us_federal | 16 | 7/15 | 12/15 | 13/16 | 2 |
| us_federal_court | 4 | 2/4 | 4/4 | 2/4 | 1 |
| us_state | 11 | 8/11 | 11/11 | 8/11 | 1 |
| utc_vs_local | 2 | 2/2 | 2/2 | 2/2 | 0 |
| vendor_invoice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| weather_product | 2 | 2/2 | 2/2 | 2/2 | 0 |
| wmo_header | 2 | 2/2 | 2/2 | 2/2 | 0 |
| zero_amount | 1 | – | – | 1/1 | 1 |
| zone_list | 1 | 1/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 154 · layout 103 · ocr 156 · ocr_regions 2
- **Documents by route class:** fast 18 · layout 21 · ocr 11 · ocr_regions 2

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 52 | 31.47 s | 114.97 s | 135.83 s |
| `extraction_wall_ms` | 52 | 11.3 ms | 52.63 s | 76.08 s |
| `worker_total_ms` | 52 | 11.0 ms | 52.63 s | 76.07 s |
| `worker_snapshot_ms` | 52 | 0.24 ms | 0.65 ms | 3.1 ms |
| `worker_parse_ms` | 52 | 3.7 ms | 87.5 ms | 1.02 s |
| `worker_analysis_ms` | 52 | 0.57 ms | 53.4 ms | 90.0 ms |
| `worker_render_ms` | 52 | 0.00 ms | 4.03 s | 10.87 s |
| `worker_image_decode_ms` | 52 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 52 | 0.00 ms | 103.06 s | 150.75 s |
| `worker_ocr_encode_ms` | 52 | 0.00 ms | 5.06 s | 8.13 s |
| `worker_ocr_engine_ms` | 52 | 0.00 ms | 95.22 s | 145.43 s |
| `worker_vision_ms` | 52 | 0.00 ms | 0.00 ms | 20.0 ms |
| `analyze_wall_ms` | 52 | 31.27 s | 55.52 s | 68.63 s |
| `distill_ms` | 52 | 1.9 ms | 53.4 ms | 64.4 ms |
| `prompt_ms` | 52 | 0.04 ms | 0.08 ms | 0.10 ms |
| `inference_ms` | 52 | 31.27 s | 55.41 s | 68.52 s |
| `prefill_ms` | 52 | 19.01 s | 41.07 s | 52.54 s |
| `generation_ms` | 52 | 12.88 s | 18.62 s | 22.64 s |
| `validation_ms` | 52 | 6.7 ms | 33.1 ms | 50.4 ms |
| `naming_ms` | 52 | 0.21 ms | 3.4 ms | 4.3 ms |
| `index_ms` | 52 | 1.8 ms | 50.0 ms | 59.3 ms |
| `retrieval_ms` | 52 | 0.04 ms | 3.3 ms | 4.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 52 | 61.3 tok/s | 62.9 tok/s | 63.1 tok/s |
| `generation_tok_per_s` | 52 | 6.8 tok/s | 7.1 tok/s | 7.1 tok/s |
| `prompt_tokens` | 52 | 1163 | 2503 | 3121 |
| `cached_tokens` | 52 | 477 | 477 | 546 |
| `generated_tokens` | 52 | 88 | 126 | 147 |
| `estimated_prompt_tokens` | 52 | 1604 | 2878 | 3398 |
| `model_requests` | 52 | 1 | 1 | 1 |
| `redistillations` | 52 | 0 | 0 | 0 |
| `source_characters` | 52 | 3149 | 75251 | 87914 |
| `digest_characters` | 52 | 2910 | 6325 | 7031 |
| `prompt_characters` | 52 | 4964 | 8380 | 9087 |
| `worker_ocr_pages` | 52 | 0 | 29 | 66 |
| `worker_ocr_passes` | 52 | 0 | 29 | 66 |
| `worker_orientation_passes` | 52 | 0 | 29 | 66 |
| `worker_rendered_pixels` | 52 | 0 | 252367280 | 574406515 |
| `index_units` | 52 | 37 | 1694 | 1903 |
| `context_units` | 52 | 34 | 98 | 122 |
| `context_tokens` | 52 | 1015 | 2288 | 2806 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 18 | 2.1 ms | 1.02 s | 1.8 ms | 1.02 s | 1.02 s | 0.37 ms | 0.00 ms | 0.00 ms |
| layout | 21 | 13.2 ms | 32.2 ms | 12.5 ms | 31.0 ms | 26.1 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 991.8 ms | 1.33 s | 991.4 ms | 1.33 s | 12.4 ms | 0.74 ms | 102.2 ms | 1.19 s |
| ocr | 11 | 5.10 s | 76.08 s | 5.09 s | 76.07 s | 88.5 ms | 90.0 ms | 10.87 s | 150.75 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 19 | 3.3 ms | 1.02 s | 3.1 ms | 1.02 s | 1.02 s | 1.0 ms | 88.4 ms | 886.2 ms |
| 2-4 | 14 | 4.6 ms | 4.41 s | 4.3 ms | 4.41 s | 22.3 ms | 3.8 ms | 415.4 ms | 6.84 s |
| 5-9 | 6 | 17.7 ms | 30.0 ms | 16.1 ms | 27.5 ms | 16.8 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 40.8 ms | 16.73 s | 39.3 ms | 16.73 s | 88.5 ms | 90.0 ms | 1.11 s | 32.51 s |
| 25-49 | 2 | 52.63 s | 76.08 s | 52.63 s | 76.07 s | 2.8 ms | 11.4 ms | 7.11 s | 150.75 s |
| 100+ | 1 | 67.20 s | 67.20 s | 67.19 s | 67.19 s | 24.0 ms | 11.7 ms | 10.87 s | 133.27 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 10.7 ms | 30.0 ms | 9.6 ms | 27.5 ms | 16.8 ms | 11.2 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 11.3 ms | 52.63 s | 11.0 ms | 52.63 s | 24.0 ms | 3.1 ms | 4.03 s | 103.06 s |
| notice | 14 | 13.2 ms | 5.19 s | 12.5 ms | 5.19 s | 88.5 ms | 90.0 ms | 629.9 ms | 9.55 s |
| statement | 2 | 0.54 ms | 76.08 s | 0.26 ms | 76.07 s | 0.90 ms | 11.4 ms | 7.11 s | 150.75 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 229 MB | 1265 MB | real4-letter-13 |
| Model server | 3268 MB | 3308 MB | real4-letter-05 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (24)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report for Pacific Gas and Electric Company.pdf` · review: LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Laura Berntsen.pdf` · trap: party Laura Berntsen (acting manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter to Jess Hewat.pdf` · trap: party Jess Hewat (manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Freedom of Information Act Request Acknowledgement.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `Application and Approval for Off-duty Employment for PUBLIC.RESOURCE.ORG.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-10**: expected `2015-06-11 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-06-11 Letter to Mr. Carl Malamud Public.Resource.Org.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter to Kansas Judicial Council.pdf` · trap: party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-07-05 Letter to Carl Malamud.pdf` · trap: party Carl Malamud (addressee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-02**: expected `2026-02-17 Notice of Appeal between City of Philadelphia and United States Department of the Interior.pdf`, got `2026-02-17 Notice of Appeal from Jessica Borwon, Acting Director of the National Park Service.pdf` · trap: party Jessica Borwon (acting director named as a defendant in her official capacity) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for JOE NEGUSE.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-06**: expected `2025-12-15 Notice of Violation for Tennessee Valley Authority.pdf`, got `2025-12-15 Assessment Followup Letter Nrc Inspection Report to Tennessee Valley Authority.pdf` · **filed without review**
- **real4-notice-11**: expected `2024-02-20 Citation and Notification of Penalty to Henry Brick Company, Inc.pdf`, got `2024-02-20 Citation and Notification of Penalty.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `2023-10-05 Requester Invoice.pdf` · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice from WTTG.pdf` · review: DATE_IS_PERIOD, LOW_CONFIDENCE
- **real4-invoice-11**: expected `2024-06-30 Invoice from WCCO-TV.pdf`, got `2024-06-30 Invoice from Smart Media Group.pdf` · review: LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `2024-11-10 Invoice from WCCO-TV Advertiser.pdf` · review: LOW_CONFIDENCE
- **real4-invoice-13**: expected `2010-04-28 Statement from Barclaycard Commercial.pdf`, got `2010-04-28 Statement from Barclaycard Commercial Duiciuycuiu.pdf` · review: TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Letter to Mr. Wright.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-03**: expected `2021-11-29 Letter to Post and Schell.doc`, got `2021-11-29 Letter of Notification of PPL Electric Utilities Corporation to Commonwealth Keystone Building.doc` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-05**: expected `2012-03-29 Freedom of Information Act Response to Daily Telegraph.doc`, got `2012-03-29 Letter to Mr Kirkup.doc` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-06**: expected `2021-02-01 Freedom of Information Request Response from Highways England.odt`, got `2021-02-01 Letter to Historical Railways Estate.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real4-office-08**: expected `2016-03-08 Freedom of Information Act Response from Office of the Advocate General.docx`, got `2016-02-18 Letter.docx` · trap: date 2016-02-18 (date of the request e-mail) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real4-office-09**: expected `2019-09-16 Freedom of Information Act Response from Office of Manpower Economics.docx`, got `2019-09-16 Letter to Department for Business, Energy and Industrial Strategy.docx` · trap: party Department for Business, Energy and Industrial Strategy (department the requester is referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
