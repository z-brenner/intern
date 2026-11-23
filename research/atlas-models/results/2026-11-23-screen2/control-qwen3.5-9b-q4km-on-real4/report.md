# InternBench: live run

- **Run:** 2026-11-23T16:54:17Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 55 documents · 55 completed · gold `284c464532`
- **Wall time:** 4434.88 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 33/52 (63.5%) |
| Document type | 52/55 (94.5%) |
| Date | 46/52 (88.5%) |
| Date role (when the reviewed date was chosen) | 19/46 (41.3%) |
| Parties | 46/55 (83.6%) |
| Relation word (when parties were named) | 45/53 (84.9%) |
| Party in the right role (when parties were named) | 12/25 (48.0%) |
| Ready / review routing | 10/38 (26.3%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 45/55 (81.8%) |
| Description specificity | 75.1% (mean of 55) |
| Evidence recall (model's quotes) | 66.7% (mean of 55) |
| Digest recall (facts reaching the digest) | 93.6% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 91.2% (mean of 55) |
| Sent to review | 87.3% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 61.5% | 13 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 54.5% | 55 |
| Review rate | 87.3% | 55 |
| Evidence recall | 66.7% | 55 |
| Total latency p50 | 69.63 s | 55 |
| Total latency p95 | 198.91 s | 55 |
| Generation latency p50 | 24.46 s | 55 |
| Generation latency p95 | 33.26 s | 55 |
| Generated tokens p50 | 90 | 55 |
| Generated tokens p95 | 118 | 55 |
| Prompt tokens p50 | 1143 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 52 |
| Trap date chosen | 5 of 55 |
| Forbidden party named | 6 of 55 |
| Spurious parties named | 6 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 155 (0.0%) |
| Right name sent to review anyway | 18 of 33 |
| Sent to review for an unsupported fact | 30 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 8/13 | 11/13 | 13/14 | 3/13 | 85.7% | 66.15 s | 86.64 s |
| letter | 25 | 13/24 | 20/24 | 19/25 | 3/12 | 96.0% | 67.97 s | 224.76 s |
| notice | 14 | 10/13 | 13/13 | 12/14 | 3/12 | 78.6% | 84.47 s | 131.28 s |
| statement | 2 | 2/2 | 2/2 | 2/2 | 1/1 | 50.0% | 43.82 s | 198.91 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native | 32 | 19/31 | 28/31 | 27/32 | 5/26 | 87.5% | 68.49 s | 224.76 s |
| office | 11 | 7/9 | 9/9 | 9/11 | 2/3 | 100.0% | 48.20 s | 92.80 s |
| scan | 10 | 5/10 | 7/10 | 8/10 | 3/9 | 90.0% | 121.66 s | 198.91 s |
| text | 2 | 2/2 | 2/2 | 2/2 | – | 0.0% | 43.82 s | 70.38 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 10/18 | 17/18 | 13/20 | 2/10 | 90.0% | 45.02 s | 78.51 s |
| 2-4 | 16 | 13/15 | 14/15 | 14/16 | 2/11 | 100.0% | 62.82 s | 92.80 s |
| 5-9 | 6 | 2/6 | 4/6 | 6/6 | 3/6 | 50.0% | 69.79 s | 121.24 s |
| 10-24 | 10 | 6/10 | 9/10 | 10/10 | 2/9 | 80.0% | 108.78 s | 131.28 s |
| 25-49 | 2 | 2/2 | 2/2 | 2/2 | 1/2 | 100.0% | 198.91 s | 224.76 s |
| 100+ | 1 | 0/1 | 0/1 | 1/1 | – | 100.0% | 261.18 s | 261.18 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 13 | 8/13 | 11/13 | 13/13 | 3/11 | 0 of 13 | 125.87 s | 261.18 s | 25.60 s | 33.26 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| acknowledgement | 3 | 2/3 | 3/3 | 3/3 | 0 |
| addressee_redacted | 6 | 3/6 | 5/6 | 4/6 | 3 |
| agency_billing | 8 | 4/8 | 6/8 | 8/8 | 1 |
| attachment_title_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attachments | 15 | 8/15 | 13/15 | 14/15 | 3 |
| body_recounts_events | 1 | – | – | 1/1 | 1 |
| born_digital | 32 | 19/31 | 28/31 | 27/32 | 6 |
| brand_vs_legal_name | 3 | 3/3 | 3/3 | 3/3 | 0 |
| broadcast_invoice | 8 | 4/8 | 6/8 | 8/8 | 1 |
| broken_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption | 4 | 3/4 | 4/4 | 3/4 | 0 |
| card_statement | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 2 | 2/2 | 2/2 | 2/2 | 0 |
| certified_mail | 4 | 2/4 | 4/4 | 4/4 | 0 |
| citation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| competing_dates | 44 | 30/44 | 39/44 | 40/44 | 6 |
| contract_numbers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 4 | 3/4 | 4/4 | 3/4 | 0 |
| cover_letter | 8 | 4/8 | 8/8 | 8/8 | 0 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_stamp | 3 | 1/3 | 1/3 | 1/3 | 1 |
| deadline_in_body | 7 | 4/7 | 6/7 | 7/7 | 1 |
| doc | 3 | 2/2 | 2/2 | 3/3 | 1 |
| docket_number | 3 | 2/3 | 3/3 | 3/3 | 0 |
| docx | 4 | 3/4 | 4/4 | 4/4 | 0 |
| doubled_ocr_text | 4 | 2/4 | 4/4 | 4/4 | 0 |
| due_date | 4 | 4/4 | 4/4 | 4/4 | 0 |
| duplicate_marking | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| embedded_second_letter | 1 | 0/1 | 0/1 | 1/1 | 1 |
| enclosure_list | 1 | – | – | 1/1 | 1 |
| esignature | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fee_invoice | 2 | 0/1 | 1/1 | 1/2 | 1 |
| file_name_date_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filed_stamp | 4 | 3/4 | 4/4 | 3/4 | 0 |
| foi_release | 1 | 1/1 | 1/1 | 1/1 | 0 |
| foi_response | 15 | 8/14 | 10/14 | 11/15 | 6 |
| forecaster_initials | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 2/2 | 2/2 | 2/2 | 0 |
| group_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| illegible_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| image_only | 2 | 1/2 | 1/2 | 1/2 | 1 |
| inspection_report | 4 | 2/4 | 4/4 | 4/4 | 0 |
| internal_email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| invoice | 14 | 8/13 | 11/13 | 13/14 | 2 |
| issuer_in_logo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| journal_number | 1 | – | – | 0/1 | 1 |
| journalist_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 25 | 13/24 | 20/24 | 19/25 | 8 |
| letterhead_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 9 | 5/9 | 7/9 | 9/9 | 1 |
| long_document | 2 | 1/2 | 1/2 | 2/2 | 1 |
| long_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_dates | 1 | – | – | 1/1 | 1 |
| metadata_date_trap | 3 | 2/2 | 2/2 | 3/3 | 1 |
| military_date_format | 1 | 0/1 | 0/1 | 1/1 | 1 |
| modern | 55 | 33/52 | 46/52 | 46/55 | 11 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 1 |
| multiple_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_documents_in_file | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 19 | 11/19 | 17/19 | 15/19 | 3 |
| new_zealand | 4 | 1/4 | 3/4 | 2/4 | 3 |
| no_addressee | 2 | 1/1 | 1/1 | 2/2 | 1 |
| no_issuer_name_in_header | 1 | – | – | 0/1 | 1 |
| no_letterhead_text | 2 | 0/2 | 2/2 | 1/2 | 1 |
| no_signer_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| nonprofit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 14 | 10/13 | 13/13 | 12/14 | 1 |
| notice_of_appeal | 4 | 3/4 | 4/4 | 3/4 | 0 |
| notice_of_violation | 2 | 2/2 | 2/2 | 2/2 | 0 |
| numeric_date_dmy | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 8 | 4/8 | 6/8 | 7/8 | 1 |
| odt | 1 | 1/1 | 1/1 | 1/1 | 0 |
| office | 11 | 7/9 | 9/9 | 9/11 | 3 |
| on_behalf_of_client | 2 | 1/2 | 2/2 | 2/2 | 0 |
| ordinal_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| pay_gov | 1 | 0/1 | 1/1 | 0/1 | 0 |
| payment_page | 2 | 2/2 | 2/2 | 2/2 | 0 |
| penalty_notice | 6 | 4/6 | 6/6 | 6/6 | 0 |
| person_in_official_capacity | 3 | 2/3 | 3/3 | 2/3 | 0 |
| plaintiff_appeals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| political_file | 8 | 4/8 | 6/8 | 8/8 | 1 |
| preliminary_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| proration_note | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line | 3 | 2/3 | 2/3 | 2/3 | 0 |
| re_line_references_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted_signer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| reference_number | 9 | 4/8 | 6/8 | 6/9 | 5 |
| referral | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_to_bank | 3 | 3/3 | 3/3 | 3/3 | 0 |
| rtf | 1 | – | – | 0/1 | 1 |
| same_day_order | 1 | 0/1 | 1/1 | 0/1 | 0 |
| scanned | 10 | 5/10 | 7/10 | 8/10 | 2 |
| service_period | 11 | 7/11 | 9/11 | 11/11 | 1 |
| short | 5 | 2/4 | 4/4 | 3/5 | 2 |
| signature_block_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_in_caps | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_on_later_page | 6 | 5/6 | 5/6 | 5/6 | 1 |
| statement | 2 | 2/2 | 2/2 | 2/2 | 0 |
| system_generated | 1 | – | – | 1/1 | 1 |
| tabular_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 2 | 2/2 | 2/2 | 2/2 | 0 |
| title_line | 2 | 1/2 | 1/2 | 2/2 | 1 |
| two_digit_year | 8 | 4/8 | 6/8 | 8/8 | 1 |
| txt | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk | 7 | 5/5 | 5/5 | 6/7 | 2 |
| undated | 3 | – | – | 2/3 | 3 |
| us | 8 | 4/8 | 6/8 | 8/8 | 1 |
| us_city | 3 | 3/3 | 3/3 | 3/3 | 0 |
| us_federal | 16 | 9/15 | 12/15 | 13/16 | 3 |
| us_federal_court | 4 | 3/4 | 4/4 | 3/4 | 0 |
| us_state | 12 | 7/12 | 12/12 | 10/12 | 2 |
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
| `total_ms` | 55 | 69.63 s | 198.91 s | 261.18 s |
| `extraction_wall_ms` | 55 | 13.0 ms | 81.59 s | 102.83 s |
| `worker_total_ms` | 55 | 12.6 ms | 81.59 s | 102.82 s |
| `worker_snapshot_ms` | 55 | 0.23 ms | 0.74 ms | 3.7 ms |
| `worker_parse_ms` | 55 | 4.8 ms | 118.3 ms | 392.9 ms |
| `worker_analysis_ms` | 55 | 0.89 ms | 71.7 ms | 146.8 ms |
| `worker_render_ms` | 55 | 0.00 ms | 6.44 s | 16.22 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 0.00 ms | 0.00 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 159.94 s | 204.02 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 7.82 s | 12.16 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 148.47 s | 195.82 s |
| `worker_vision_ms` | 55 | 0.00 ms | 0.00 ms | 25.5 ms |
| `analyze_wall_ms` | 55 | 69.60 s | 125.08 s | 162.41 s |
| `distill_ms` | 55 | 2.5 ms | 67.9 ms | 81.5 ms |
| `prompt_ms` | 55 | 0.05 ms | 0.10 ms | 0.12 ms |
| `inference_ms` | 55 | 69.54 s | 124.94 s | 162.30 s |
| `prefill_ms` | 55 | 46.80 s | 100.47 s | 130.43 s |
| `generation_ms` | 55 | 24.46 s | 33.26 s | 33.82 s |
| `validation_ms` | 55 | 8.0 ms | 36.0 ms | 65.7 ms |
| `naming_ms` | 55 | 0.25 ms | 4.2 ms | 5.5 ms |
| `index_ms` | 55 | 2.2 ms | 64.0 ms | 75.9 ms |
| `retrieval_ms` | 55 | 0.04 ms | 4.1 ms | 5.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 24.4 tok/s | 25.0 tok/s | 25.4 tok/s |
| `generation_tok_per_s` | 55 | 3.7 tok/s | 3.8 tok/s | 3.8 tok/s |
| `prompt_tokens` | 55 | 1143 | 2503 | 3121 |
| `cached_tokens` | 55 | 477 | 477 | 546 |
| `generated_tokens` | 55 | 90 | 118 | 118 |
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
| fast | 18 | 2.3 ms | 393.9 ms | 1.9 ms | 393.6 ms | 392.9 ms | 0.44 ms | 0.00 ms | 0.00 ms |
| layout | 24 | 12.3 ms | 43.1 ms | 11.5 ms | 41.2 ms | 35.3 ms | 14.3 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 2 | 1.34 s | 2.02 s | 1.33 s | 2.02 s | 17.8 ms | 0.94 ms | 136.5 ms | 1.83 s |
| ocr | 11 | 7.81 s | 102.83 s | 7.80 s | 102.82 s | 119.6 ms | 146.8 ms | 16.22 s | 204.02 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20 | 4.0 ms | 393.9 ms | 3.6 ms | 393.6 ms | 16.9 ms | 1.2 ms | 0.00 ms | 0.00 ms |
| 2-4 | 16 | 6.9 ms | 6.63 s | 6.3 ms | 6.63 s | 31.7 ms | 4.7 ms | 529.9 ms | 10.05 s |
| 5-9 | 6 | 23.9 ms | 41.9 ms | 21.3 ms | 38.0 ms | 22.9 ms | 14.4 ms | 0.00 ms | 0.00 ms |
| 10-24 | 10 | 54.4 ms | 26.61 s | 52.5 ms | 26.61 s | 119.6 ms | 146.8 ms | 1.81 s | 51.69 s |
| 25-49 | 2 | 81.59 s | 102.83 s | 81.59 s | 102.82 s | 3.7 ms | 14.8 ms | 12.56 s | 204.02 s |
| 100+ | 1 | 98.77 s | 98.77 s | 98.76 s | 98.76 s | 29.8 ms | 14.8 ms | 16.22 s | 196.17 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 14 | 7.7 ms | 41.9 ms | 6.5 ms | 38.0 ms | 22.9 ms | 14.4 ms | 0.00 ms | 0.00 ms |
| letter | 25 | 13.0 ms | 81.59 s | 12.7 ms | 81.59 s | 31.7 ms | 3.8 ms | 6.44 s | 159.94 s |
| notice | 14 | 17.3 ms | 7.97 s | 16.2 ms | 7.96 s | 119.6 ms | 146.8 ms | 960.9 ms | 14.90 s |
| statement | 2 | 0.67 ms | 102.83 s | 0.26 ms | 102.82 s | 1.3 ms | 14.8 ms | 12.56 s | 204.02 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 220 MB | 1205 MB | real4-letter-13 |
| Model server | 6086 MB | 6132 MB | real4-letter-05 |
| Benchmark (engine) | 23 MB | 28 MB | real4-office-13 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (19)

- **real4-letter-01**: expected `2024-12-13 Security Baseline Inspection Report to Pacific Gas and Electric Company.pdf`, got `2024-12-13 Inspection Report for Pacific Gas and Electric Company.pdf` · **filed without review**
- **real4-letter-02**: expected `2024-12-13 NRC Inspection Report to Southern California Edison Company.pdf`, got `2024-12-13 Report for Southern California Edison Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real4-letter-03**: expected `2024-07-05 Official Information Act Response from The Treasury.pdf`, got `2024-07-05 Letter to Laura Berntsen.pdf` · trap: party Laura Berntsen (acting manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-04**: expected `2024-09-03 Official Information Act Response from The Treasury.pdf`, got `2024-09-03 Letter from Jess Hewat.pdf` · trap: party Jess Hewat (manager who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-06**: expected `2024-10-04 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf`, got `2024-08-23 Response to an Official Information Act Request from Ministry for Pacific Peoples.pdf` · trap: date 2024-08-23 (date of the request) · review: PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real4-letter-07**: expected `2015-04-27 Freedom of Information Act Request Acknowledgement to Public.Resource.Org.pdf`, got `Freedom of Information Act Request Acknowledgement.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-09**: expected `2015-09-08 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-07-22 Application and Approval for Off-duty Employment for PUBLIC.RESOURCE.ORG.pdf` · trap: date 2015-07-22 (date of the attached FOIA request letter) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-letter-11**: expected `2021-05-14 Letter to Public.Resource.Org.pdf`, got `2021-05-14 Letter from Kansas Judicial Council.pdf` · trap: party Kansas Judicial Council (body the letter will be referred to) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real4-letter-14**: expected `2015-07-15 Freedom of Information Act Response to Public.Resource.Org.pdf`, got `2015-07-05 Letter to Carl Malamud.pdf` · trap: party Carl Malamud (addressee) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-03**: expected `2026-03-02 Notice of Appeal between Joe Neguse and U.S. Immigration and Customs Enforcement.pdf`, got `2026-03-02 Notice of Appeal for JOE NEGUSE.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-12**: expected `2025-09-04 Notice of Civil Penalty Assessment and Order to Crown Enterprises, LLC.pdf`, got `2025-09-04 Notice of Civil Penalty Assessment and Order for Crown Enterprises LLC.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-notice-15**: expected `2025-07-22 Notice of Civil Penalty Assessment and Order to Columbia Plywood Corporation.pdf`, got `2025-07-22 Notice of Civil Penalty Assessment and Order for OTLO Columbia Plywood Corporation.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-invoice-01**: expected `2023-10-05 Invoice from National Credit Union Administration.pdf`, got `2023-10-05 Requester Invoice.pdf` · review: DATE_AMBIGUOUS, PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real4-invoice-10**: expected `2024-01-28 Invoice from WTTG.pdf`, got `2024-01-08 Invoice from WTTG.pdf` · review: DATE_IS_PERIOD, LOW_CONFIDENCE
- **real4-invoice-11**: expected `2024-06-30 Invoice from WCCO-TV.pdf`, got `2024-06-30 Invoice from WCCO-TV Advertiser.pdf` · review: LOW_CONFIDENCE
- **real4-invoice-12**: expected `2024-11-10 Invoice from WCCO-TV.pdf`, got `2024-11-10 Invoice from WCCO-TV Advertiser.pdf` · review: LOW_CONFIDENCE
- **real4-invoice-14**: expected `2024-05-26 Invoice from WTTG.pdf`, got `2024-05-06 Invoice from WTTG.pdf` · trap: date 2024-05-06 (first air date) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real4-office-02**: expected `2016-05-12 Letter to Post & Schell.docx`, got `2016-05-12 Letter of Notification of PPL Electric Utilities from PPL Electric Utilities Corporation.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real4-office-10**: expected `2016-04-06 Email to Department of General Services.msg`, got `2016-04-06 Email from Bruce Betts.msg` · trap: party Bruce Betts (sender) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
