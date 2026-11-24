# InternBench: live run

- **Run:** 2026-11-21T04:57:25Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 54 documents · 54 completed · gold `2c580c87e2`
- **Wall time:** 890.50 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 22/52 (42.3%) |
| Document type | 44/54 (81.5%) |
| Date | 40/53 (75.5%) |
| Date role (when the reviewed date was chosen) | 12/39 (30.8%) |
| Parties | 34/54 (63.0%) |
| Relation word (when parties were named) | 35/49 (71.4%) |
| Party in the right role (when parties were named) | 12/22 (54.5%) |
| Ready / review routing | 7/39 (17.9%) |
| Description states nothing false | 54/54 (100.0%) |
| Description is specific | 35/54 (64.8%) |
| Description specificity | 62.4% (mean of 54) |
| Evidence recall (model's quotes) | 45.7% (mean of 54) |
| Digest recall (facts reaching the digest) | 86.7% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 86.7% (mean of 54) |
| Sent to review | 90.7% of 54 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 50.0% | 54 |
| Review rate | 90.7% | 54 |
| Evidence recall | 45.7% | 54 |
| Total latency p50 | 13.93 s | 54 |
| Total latency p95 | 34.14 s | 54 |
| Generation latency p50 | 6.55 s | 54 |
| Generation latency p95 | 8.33 s | 54 |
| Generated tokens p50 | 94 | 54 |
| Generated tokens p95 | 120 | 54 |
| Prompt tokens p50 | 1171 | 54 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 52 |
| Trap date chosen | 6 of 54 |
| Forbidden party named | 8 of 54 |
| Spurious parties named | 11 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 117 (0.0%) |
| Right name sent to review anyway | 17 of 37 |
| Sent to review for an unsupported fact | 27 of 54 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 13 | 4/12 | 9/13 | 6/13 | 1/9 | 100.0% | 11.64 s | 18.51 s |
| letter | 20 | 7/19 | 15/19 | 14/20 | 2/13 | 95.0% | 16.95 s | 37.74 s |
| notice | 18 | 11/18 | 16/18 | 14/18 | 4/17 | 77.8% | 15.37 s | 34.14 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 5.22 s | 5.41 s |
| statement | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 18.31 s | 18.31 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 1/7 | 2/8 | 3/8 | 1/3 | 100.0% | 9.06 s | 11.64 s |
| native | 16 | 7/16 | 12/16 | 10/16 | 1/12 | 93.8% | 12.83 s | 22.87 s |
| office | 14 | 7/14 | 12/14 | 9/14 | 2/11 | 85.7% | 15.29 s | 19.84 s |
| scan | 10 | 4/10 | 9/10 | 7/10 | 1/8 | 90.0% | 18.73 s | 64.54 s |
| text | 6 | 3/5 | 5/5 | 5/6 | 2/5 | 83.3% | 14.04 s | 26.78 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 7/20 | 13/21 | 12/22 | 3/14 | 95.5% | 10.73 s | 24.27 s |
| 2-4 | 24 | 11/24 | 20/24 | 14/24 | 3/18 | 87.5% | 15.29 s | 20.83 s |
| 5-9 | 5 | 3/5 | 5/5 | 5/5 | 1/5 | 80.0% | 19.23 s | 37.74 s |
| 10-24 | 3 | 1/3 | 2/3 | 3/3 | 0/2 | 100.0% | 34.14 s | 64.54 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 1/3 | 2/3 | 3/3 | 0/2 | 0 of 3 | 34.14 s | 64.54 s | 7.71 s | 7.92 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_counsel | 3 | 1/2 | 2/2 | 2/3 | 2 |
| addressed_to_government | 3 | 0/3 | 3/3 | 3/3 | 0 |
| adopted_vs_released | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agency_billing | 2 | 2/2 | 2/2 | 2/2 | 0 |
| agency_published_sample | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agent_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| aging_table | 1 | 0/1 | 0/1 | 0/1 | 1 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appended_statements | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 2 | 0/2 | 0/2 | 1/2 | 2 |
| as_of_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| attachment_note | 2 | 0/2 | 2/2 | 1/2 | 1 |
| automated_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| bankruptcy | 3 | 3/3 | 3/3 | 3/3 | 0 |
| blank_date | 1 | – | 0/1 | 0/1 | 1 |
| born_digital | 15 | 7/15 | 11/15 | 9/15 | 6 |
| business_review_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| c_o_customer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 1/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_list | 2 | 0/2 | 0/2 | 1/2 | 2 |
| company_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 39 | 16/38 | 30/38 | 26/39 | 10 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| court_filing | 11 | 8/11 | 10/11 | 8/11 | 1 |
| da_number | 3 | 2/2 | 2/2 | 3/3 | 1 |
| damaged | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dataset_footer_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 3 | 0/3 | 3/3 | 1/3 | 1 |
| date_stamp | 1 | 0/1 | 0/1 | 0/1 | 1 |
| dd_mm_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dual_salutation | 1 | 0/1 | 0/1 | 0/1 | 0 |
| due_date_trap | 2 | 0/2 | 2/2 | 0/2 | 0 |
| ecf_header | 5 | 4/5 | 4/5 | 4/5 | 1 |
| ecf_header_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email_addresses | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printout | 1 | 0/1 | 1/1 | 0/1 | 1 |
| eml | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enron | 7 | 1/7 | 5/7 | 2/7 | 3 |
| exhibit_attached | 2 | 2/2 | 2/2 | 2/2 | 0 |
| faint | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footer_codes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 3 | 1/3 | 3/3 | 3/3 | 0 |
| footnotes_inline | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| former_company_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| government | 7 | 3/7 | 6/7 | 7/7 | 1 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_regulator | 3 | 3/3 | 3/3 | 3/3 | 0 |
| government_sender | 8 | 3/7 | 5/7 | 5/8 | 5 |
| handwritten | 7 | 1/6 | 2/7 | 3/7 | 2 |
| handwritten_signature | 5 | 3/5 | 5/5 | 3/5 | 0 |
| hearing_date_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historical | 5 | 0/4 | 0/5 | 2/5 | 2 |
| illegible_labels | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 8 | 1/7 | 2/8 | 3/8 | 2 |
| in_re_caption | 3 | 3/3 | 3/3 | 3/3 | 0 |
| inconsistent_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| invoice | 13 | 4/12 | 9/13 | 6/13 | 4 |
| invoice_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice_reference | 1 | 0/1 | 0/1 | 1/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 1/1 | 0 |
| issuer_in_remit_to | 1 | 0/1 | 1/1 | 0/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 1/1 | 0 |
| law_firm_letterhead | 2 | 0/2 | 2/2 | 1/2 | 0 |
| letter | 21 | 8/20 | 16/20 | 15/21 | 6 |
| letter_in_spreadsheet | 1 | 0/1 | 0/1 | 1/1 | 0 |
| letter_to_judge | 1 | 0/1 | 1/1 | 0/1 | 0 |
| letterhead_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 1/4 | 3/4 | 4/4 | 1 |
| long_lines | 1 | – | – | 1/1 | 1 |
| many_attorneys | 3 | 3/3 | 3/3 | 3/3 | 0 |
| many_dates | 3 | 1/3 | 2/3 | 1/3 | 1 |
| many_deadlines | 1 | 1/1 | 1/1 | 1/1 | 0 |
| memo_endorsed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| metadata_date_trap | 5 | 1/5 | 4/5 | 1/5 | 2 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 1 |
| missing_date | 1 | – | – | 1/1 | 1 |
| missing_letterhead | 1 | – | – | 1/1 | 1 |
| missing_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| modern | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multi_page | 3 | 1/3 | 3/3 | 2/3 | 1 |
| multi_sheet | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_addressees | 1 | 0/1 | 0/1 | 0/1 | 1 |
| multiple_defendants | 4 | 2/4 | 4/4 | 2/4 | 0 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| needs_review | 1 | – | 0/1 | 0/1 | 1 |
| no_explicit_title | 4 | 0/4 | 0/4 | 2/4 | 1 |
| no_invoice_date_field | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_typed_signature | 1 | 0/1 | 1/1 | 1/1 | 0 |
| non_ascii_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| non_utf8_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 18 | 11/18 | 16/18 | 14/18 | 3 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odd_text_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 1 | 1/1 | 1/1 | 1/1 | 0 |
| office_format | 8 | 6/8 | 8/8 | 7/8 | 1 |
| office_holder_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| one_page | 2 | 0/2 | 1/2 | 0/2 | 1 |
| ordinal_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| organisation_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_mention | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| payment_stub | 1 | 0/1 | 1/1 | 1/1 | 0 |
| penalty | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_only_date | 1 | – | 0/1 | 0/1 | 1 |
| person_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| placeholder_data | 1 | 0/1 | 1/1 | 1/1 | 0 |
| placeholder_defendants | 1 | 0/1 | 1/1 | 0/1 | 0 |
| plain_text | 5 | 3/4 | 4/4 | 5/5 | 1 |
| published_sample | 1 | 0/1 | 1/1 | 1/1 | 0 |
| re_line_names_client | 1 | 0/1 | 1/1 | 0/1 | 1 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| redacted | 1 | 0/1 | 0/1 | 0/1 | 0 |
| references_other_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 1/3 | 2/3 | 3/3 | 1 |
| release_date_equals_letter_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| request_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| response_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| rtf | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 12 | 5/12 | 11/12 | 8/12 | 1 |
| secretariat_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 1 |
| service_period_trap | 2 | 1/2 | 2/2 | 1/2 | 0 |
| short | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_trap | 9 | 6/9 | 8/9 | 8/9 | 1 |
| signed_for | 1 | 0/1 | 0/1 | 0/1 | 1 |
| spreadsheet | 6 | 1/6 | 4/6 | 2/6 | 2 |
| staff_names | 2 | 0/2 | 1/2 | 0/2 | 2 |
| stamp | 6 | 3/6 | 4/6 | 4/6 | 1 |
| statement | 1 | 0/1 | 0/1 | 0/1 | 1 |
| tabular | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tax | 1 | 0/1 | 1/1 | 1/1 | 0 |
| text_date_cell | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_stamp | 2 | 2/2 | 2/2 | 2/2 | 0 |
| timezone_headers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| titled_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 0/1 | 0 |
| two_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| uk | 3 | 0/3 | 1/3 | 1/3 | 2 |
| uk_date_format | 2 | 0/2 | 1/2 | 0/2 | 1 |
| unsigned_department_signoff | 1 | 0/1 | 0/1 | 0/1 | 0 |
| untitled_document | 1 | 0/1 | 0/1 | 1/1 | 0 |
| utf8_bom | 1 | – | – | 1/1 | 1 |
| warning_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| word_doc_legacy | 2 | 1/2 | 2/2 | 2/2 | 0 |
| word_docx | 4 | 3/4 | 4/4 | 3/4 | 1 |
| xls | 6 | 1/6 | 4/6 | 2/6 | 2 |

## Routes

- **Pages:** fast 66 · layout 17 · ocr 67
- **Documents by route class:** fast 27 · layout 9 · ocr 18

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 54 | 13.93 s | 34.14 s | 64.54 s |
| `extraction_wall_ms` | 54 | 4.6 ms | 15.65 s | 49.57 s |
| `worker_total_ms` | 54 | 4.2 ms | 15.65 s | 49.57 s |
| `worker_snapshot_ms` | 54 | 0.20 ms | 1.3 ms | 3.4 ms |
| `worker_parse_ms` | 54 | 1.9 ms | 31.8 ms | 1.81 s |
| `worker_analysis_ms` | 54 | 0.18 ms | 2.4 ms | 4.0 ms |
| `worker_render_ms` | 54 | 0.00 ms | 1.05 s | 2.62 s |
| `worker_image_decode_ms` | 54 | 0.00 ms | 54.7 ms | 431.5 ms |
| `worker_ocr_ms` | 54 | 0.00 ms | 30.26 s | 97.26 s |
| `worker_ocr_encode_ms` | 54 | 0.00 ms | 2.27 s | 7.68 s |
| `worker_ocr_engine_ms` | 54 | 0.00 ms | 27.73 s | 89.33 s |
| `worker_vision_ms` | 54 | 0.00 ms | 5.8 ms | 28.1 ms |
| `analyze_wall_ms` | 54 | 13.93 s | 22.83 s | 26.78 s |
| `distill_ms` | 54 | 2.0 ms | 30.0 ms | 50.4 ms |
| `prompt_ms` | 54 | 0.03 ms | 0.05 ms | 0.05 ms |
| `inference_ms` | 54 | 13.92 s | 22.77 s | 26.68 s |
| `prefill_ms` | 54 | 7.38 s | 14.84 s | 17.65 s |
| `generation_ms` | 54 | 6.55 s | 8.33 s | 9.02 s |
| `validation_ms` | 54 | 5.6 ms | 26.6 ms | 63.6 ms |
| `naming_ms` | 54 | 0.23 ms | 3.2 ms | 5.9 ms |
| `index_ms` | 54 | 1.8 ms | 27.4 ms | 42.7 ms |
| `retrieval_ms` | 54 | 0.04 ms | 2.5 ms | 6.6 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 54 | 158.5 tok/s | 161.3 tok/s | 161.8 tok/s |
| `generation_tok_per_s` | 54 | 14.7 tok/s | 15.1 tok/s | 15.1 tok/s |
| `prompt_tokens` | 54 | 1171 | 2322 | 2754 |
| `cached_tokens` | 54 | 477 | 497 | 497 |
| `generated_tokens` | 54 | 94 | 120 | 130 |
| `estimated_prompt_tokens` | 54 | 1649 | 3249 | 3525 |
| `model_requests` | 54 | 1 | 1 | 1 |
| `redistillations` | 54 | 0 | 0 | 0 |
| `source_characters` | 54 | 3187 | 55143 | 91065 |
| `digest_characters` | 54 | 2899 | 8058 | 8523 |
| `prompt_characters` | 54 | 4948 | 10112 | 10577 |
| `worker_ocr_pages` | 54 | 0 | 9 | 23 |
| `worker_ocr_passes` | 54 | 0 | 9 | 23 |
| `worker_orientation_passes` | 54 | 0 | 9 | 23 |
| `worker_rendered_pixels` | 54 | 0 | 75735000 | 193545000 |
| `index_units` | 54 | 35 | 196 | 385 |
| `context_units` | 54 | 32 | 62 | 71 |
| `context_tokens` | 54 | 1062 | 2659 | 2936 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 27 | 2.2 ms | 50.5 ms | 1.6 ms | 50.0 ms | 48.9 ms | 0.59 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 5.6 ms | 18.3 ms | 5.2 ms | 17.6 ms | 14.3 ms | 3.6 ms | 0.00 ms | 0.00 ms |
| ocr | 18 | 2.11 s | 49.57 s | 2.11 s | 49.57 s | 9.5 ms | 2.4 ms | 2.62 s | 97.26 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 2.3 ms | 1.78 s | 1.9 ms | 1.78 s | 9.2 ms | 0.28 ms | 0.00 ms | 1.34 s |
| 2-4 | 24 | 5.0 ms | 4.92 s | 4.6 ms | 4.92 s | 48.9 ms | 1.8 ms | 328.4 ms | 8.26 s |
| 5-9 | 5 | 4.5 ms | 21.67 s | 4.0 ms | 21.67 s | 9.6 ms | 2.3 ms | 1.05 s | 42.61 s |
| 10-24 | 3 | 15.65 s | 49.57 s | 15.65 s | 49.57 s | 31.8 ms | 4.0 ms | 2.62 s | 97.26 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 13 | 10.0 ms | 2.11 s | 9.1 ms | 2.11 s | 9.5 ms | 3.6 ms | 0.00 ms | 2.05 s |
| letter | 20 | 4.4 ms | 21.67 s | 3.9 ms | 21.67 s | 48.9 ms | 1.8 ms | 1.05 s | 42.61 s |
| notice | 18 | 4.4 ms | 15.65 s | 3.9 ms | 15.65 s | 31.8 ms | 4.0 ms | 1.83 s | 30.26 s |
| receipt | 2 | 572.3 ms | 604.8 ms | 571.9 ms | 604.5 ms | 7.4 ms | 0.06 ms | 0.00 ms | 538.7 ms |
| statement | 1 | 1.5 ms | 1.5 ms | 0.92 ms | 0.92 ms | 0.70 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 576 MB | 1288 MB | real-letter-03 |
| Model server | 1578 MB | 1595 MB | real-office-15 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (30)

- **real-letter-02**: expected `2012-06-19 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-06-19 Letter to Antitrust Division.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-03**: expected `2012-11-30 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-11-30 Letter - IPXI Holdings, LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `Notice to Peter Van Tuyn.pdf` · trap: party Peter Van Tuyn (addressee person (counsel)) · review: TYPE_UNSUPPORTED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-08 Letter from Toledo Refining Company LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-01-31 Letter to TPWR Developer, LLC.pdf` · trap: date 2025-01-31 (date of TPWR's SEP Completion Report) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-09**: expected `2020-04-16 Business Review Letter to Association of Independent Commercial Producers.pdf`, got `2020-04-16 Letter from Makan Delrahim.pdf` · trap: party Makan Delrahim (Assistant Attorney General named in the letterhead, who signs) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Letter from Alistair Thompson.pdf` · trap: party Alistair Thompson (CMA director named in the From line, who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-11**: expected `2018-06-20 Decision Letter to Reach plc.pdf`, got `Letter to Northern and Shell.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-12**: expected `2024-05-21 Letter to Explain the Need for Redactions.pdf`, got `2024-05-21 Letter to Explain the Need for Redactions to Tod's S.p.A.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-02**: expected `2009-10-13 Notice of Appeal between Sompo Japan Insurance Company of America and Norfolk Southern Railway Company.pdf`, got `2009-10-13 Notice of Appeal for Norfolk Southern Railway Company.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-04**: expected `2009-11-02 Notice of Dismissal between Luxo AS and The Walt Disney Company.pdf`, got `2009-11-02 Notice of Dismissal for The Walt Disney Company, Pixar.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `2024-06-10 Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · trap: date 2024-06-10 (date of the final judgment or order being appealed) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real-notice-12**: expected `2018-01-08 CP161 Notice for ABC-XYZ Company.pdf`, got `2018-01-08 Notice from ABC-XYZ Company.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `2025-10-20 Final enforcement notice for Euro Car Parks Limited.pdf` · trap: date 2025-10-20 (date of the CMA's provisional notice) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-14**: expected `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN, Inc.txt`, got `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document - ABWOODWORTH.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document from Jacoby Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles from A Deming, Palmer & Co.jpg` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-04**: expected `1859-01-13 Receipt from Leesburg Washingtonian.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `1996-08-28 Invoice.tif` · review: PARTY_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-invoice-11**: expected `2010-11-29 Invoice from C-SPAN Archives.pdf`, got `2010-11-29 Invoice from Carl Malamud.pdf` · trap: party Carl Malamud (customer's contact) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-06**: expected `2017-01-04 Cover Letter to Public Accounts Committee.doc`, got `2017-01-04 Report from Public Accounts Committee.doc` · **filed without review**
- **real-office-08**: expected `2020-09-10 Letter to Labour Party.docx`, got `2020-09-10 Letter to Angela Rayner MP.docx` · trap: party Angela Rayner (party chair addressed by name) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-office-09**: expected `2001-02-08 Invoice from California Power Exchange.xls`, got `2001-02-08 Invoice from EPMI.xls` · review: LOW_CONFIDENCE
- **real-office-10**: expected `2001-04-18 Invoice from Transwestern Pipeline Company.xls`, got `2001-04-18 Invoice from EOG RESOURCES, INC.xls` · trap: party EOG Resources, Inc. (customer alone) · review: LOW_CONFIDENCE
- **real-office-11**: expected `2001-04-23 Invoice Dispute Letter to Enron North America.xls`, got `Invoice from Enron North America.xls` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-13**: expected `2001-09-14 Invoice from Enron North America.xls`, got `2001-09-14 Invoice.xls` · review: PARTY_MISSING, LOW_CONFIDENCE
- **real-office-14**: expected `2001-11-26 Outstanding Invoices Statement from Locke Liddell & Sapp LLP.xls`, got `2000-05-11 Invoice.xls` · trap: date 2000-05-11 (earliest individual invoice date in the table) · review: PARTY_MISSING, LOW_CONFIDENCE
- **real-office-15**: expected `2002-03-20 Ship Notification for Enron Corp.eml`, got `2002-03-20 Email to Richard Randall Lillian Vernon Corporation.eml` · trap: party Richard Randall (consignee contact person) · review: PARTY_UNSUPPORTED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
