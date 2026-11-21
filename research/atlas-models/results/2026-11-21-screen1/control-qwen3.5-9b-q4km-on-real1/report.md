# InternBench: live run

- **Run:** 2026-11-21T05:05:10Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 57 documents · 57 completed · gold `20264a93a5`
- **Wall time:** 3410.18 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 34/55 (61.8%) |
| Document type | 48/57 (84.2%) |
| Date | 47/56 (83.9%) |
| Date role (when the reviewed date was chosen) | 15/46 (32.6%) |
| Parties | 44/57 (77.2%) |
| Relation word (when parties were named) | 47/53 (88.7%) |
| Party in the right role (when parties were named) | 18/31 (58.1%) |
| Ready / review routing | 7/43 (16.3%) |
| Description states nothing false | 56/57 (98.2%) |
| Description is specific | 47/57 (82.5%) |
| Description specificity | 69.0% (mean of 57) |
| Evidence recall (model's quotes) | 64.3% (mean of 57) |
| Digest recall (facts reaching the digest) | 88.2% (mean of 57) |
| Prompt recall (facts reaching the prompt sent) | 88.2% (mean of 57) |
| Sent to review | 91.2% of 57 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 4 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 40.4% | 57 |
| Review rate | 91.2% | 57 |
| Evidence recall | 64.3% | 57 |
| Total latency p50 | 57.09 s | 57 |
| Total latency p95 | 96.42 s | 57 |
| Generation latency p50 | 22.93 s | 57 |
| Generation latency p95 | 30.32 s | 57 |
| Generated tokens p50 | 94 | 57 |
| Generated tokens p95 | 122 | 57 |
| Prompt tokens p50 | 1175 | 57 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 55 |
| Trap date chosen | 6 of 57 |
| Forbidden party named | 4 of 57 |
| Spurious parties named | 8 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 138 (0.7%) |
| Right name sent to review anyway | 26 of 41 |
| Sent to review for an unsupported fact | 23 of 57 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 11/16 | 12/17 | 11/17 | 3/13 | 88.2% | 46.64 s | 79.72 s |
| letter | 20 | 9/19 | 18/19 | 16/20 | 1/13 | 100.0% | 57.18 s | 92.60 s |
| notice | 18 | 14/18 | 16/18 | 16/18 | 3/17 | 83.3% | 58.70 s | 96.85 s |
| receipt | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 11.88 s | 11.88 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 87.63 s | 87.63 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 2/7 | 3/8 | 2/8 | 1/4 | 100.0% | 20.16 s | 56.48 s |
| native | 17 | 11/17 | 14/17 | 14/17 | 2/13 | 88.2% | 56.21 s | 87.25 s |
| office | 14 | 11/14 | 14/14 | 13/14 | 1/11 | 92.9% | 58.70 s | 87.63 s |
| scan | 12 | 7/12 | 11/12 | 11/12 | 2/10 | 83.3% | 66.57 s | 125.73 s |
| text | 6 | 3/5 | 5/5 | 4/6 | 1/5 | 100.0% | 53.75 s | 96.42 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 21 | 10/19 | 14/20 | 12/21 | 3/14 | 95.2% | 43.99 s | 94.43 s |
| 2-4 | 27 | 18/27 | 25/27 | 23/27 | 2/21 | 92.6% | 58.30 s | 77.42 s |
| 5-9 | 5 | 4/5 | 5/5 | 5/5 | 1/5 | 80.0% | 73.93 s | 92.60 s |
| 10-24 | 4 | 2/4 | 3/4 | 4/4 | 1/3 | 75.0% | 87.25 s | 125.73 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 4 | 2/4 | 3/4 | 4/4 | 1/3 | 0 of 4 | 87.25 s | 125.73 s | 24.09 s | 35.15 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_counsel | 3 | 1/2 | 2/2 | 2/3 | 1 |
| addressed_to_government | 3 | 0/3 | 3/3 | 3/3 | 0 |
| adopted_vs_released | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agency_billing | 4 | 3/4 | 3/4 | 3/4 | 2 |
| agency_in_account_with | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agent_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| aging_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appended_statements | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 2 | 0/2 | 1/2 | 2/2 | 1 |
| as_of_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| attachment_note | 2 | 1/2 | 2/2 | 1/2 | 1 |
| automated_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| bankruptcy | 3 | 3/3 | 3/3 | 3/3 | 0 |
| blank_date | 1 | – | 0/1 | 0/1 | 0 |
| born_digital | 16 | 10/16 | 13/16 | 13/16 | 4 |
| business_review_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| c_o_customer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_list | 2 | 0/2 | 1/2 | 2/2 | 1 |
| company_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 42 | 25/41 | 35/41 | 32/42 | 10 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 11 | 10/11 | 10/11 | 10/11 | 1 |
| da_number | 3 | 2/2 | 2/2 | 2/3 | 1 |
| dataset_footer_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 3 | 0/3 | 3/3 | 2/3 | 1 |
| date_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| dd_mm_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dual_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| due_date_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| ecf_header | 5 | 4/5 | 4/5 | 4/5 | 1 |
| ecf_header_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email_addresses | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| eml | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enron | 7 | 4/7 | 7/7 | 6/7 | 1 |
| exhibit_attached | 2 | 2/2 | 2/2 | 2/2 | 0 |
| faint | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footer_codes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 3 | 1/3 | 3/3 | 3/3 | 0 |
| footnotes_inline | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| former_company_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government | 7 | 5/7 | 6/7 | 7/7 | 1 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_regulator | 3 | 3/3 | 3/3 | 3/3 | 0 |
| government_sender | 8 | 3/7 | 6/7 | 6/8 | 2 |
| handwritten | 6 | 1/5 | 2/6 | 1/6 | 1 |
| handwritten_signature | 5 | 5/5 | 5/5 | 5/5 | 0 |
| hearing_date_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historical | 4 | 0/3 | 0/4 | 0/4 | 1 |
| illegible_labels | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 8 | 2/7 | 3/8 | 2/8 | 1 |
| in_re_caption | 3 | 3/3 | 3/3 | 3/3 | 0 |
| inconsistent_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| invoice | 17 | 11/16 | 12/17 | 11/17 | 3 |
| invoice_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice_reference | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 0/1 | 0 |
| issuer_in_remit_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| law_firm_letterhead | 2 | 0/2 | 2/2 | 1/2 | 1 |
| letter | 21 | 10/20 | 19/20 | 17/21 | 4 |
| letter_in_spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_to_judge | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 1/4 | 3/4 | 4/4 | 1 |
| long_lines | 1 | – | – | 0/1 | 1 |
| many_attorneys | 3 | 3/3 | 3/3 | 3/3 | 0 |
| many_dates | 6 | 3/6 | 5/6 | 4/6 | 2 |
| many_deadlines | 1 | 1/1 | 1/1 | 1/1 | 0 |
| memo_endorsed | 1 | 0/1 | 1/1 | 0/1 | 1 |
| metadata_date_trap | 5 | 4/5 | 5/5 | 5/5 | 0 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| missing_date | 1 | – | – | 0/1 | 1 |
| missing_letterhead | 1 | – | – | 0/1 | 1 |
| missing_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 3/4 | 3/4 | 3/4 | 2 |
| multi_frame | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_page | 6 | 4/6 | 5/6 | 5/6 | 2 |
| multi_sheet | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 4/4 | 4/4 | 4/4 | 0 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| needs_review | 1 | – | 0/1 | 0/1 | 0 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_title | 4 | 0/4 | 0/4 | 0/4 | 1 |
| no_invoice_date_field | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_typed_signature | 1 | 0/1 | 1/1 | 1/1 | 0 |
| non_ascii_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| non_utf8_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 18 | 14/18 | 16/18 | 16/18 | 3 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odd_text_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 1 | 1/1 | 1/1 | 1/1 | 0 |
| office_format | 8 | 7/8 | 8/8 | 7/8 | 1 |
| office_holder_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| one_page | 2 | 1/2 | 1/2 | 1/2 | 1 |
| ordinal_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| organisation_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_mention | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| payment_stub | 1 | 1/1 | 1/1 | 1/1 | 0 |
| penalty | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_only_date | 1 | – | 0/1 | 0/1 | 0 |
| person_in_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| placeholder_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholder_defendants | 1 | 1/1 | 1/1 | 1/1 | 0 |
| plain_text | 5 | 3/4 | 4/4 | 4/5 | 1 |
| published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line_names_client | 1 | 0/1 | 1/1 | 1/1 | 0 |
| receipt | 1 | 0/1 | 0/1 | 0/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| references_other_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 1/3 | 2/3 | 3/3 | 1 |
| release_date_equals_letter_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| request_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| response_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| rtf | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 15 | 9/15 | 14/15 | 13/15 | 2 |
| secretariat_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| short | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_trap | 9 | 7/9 | 9/9 | 8/9 | 1 |
| signed_for | 1 | 0/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 6 | 4/6 | 6/6 | 6/6 | 0 |
| staff_names | 2 | 1/2 | 2/2 | 2/2 | 0 |
| stamp | 7 | 5/7 | 5/7 | 5/7 | 1 |
| statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| stylised_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tabular | 2 | 1/2 | 1/2 | 1/2 | 2 |
| tax | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text_date_cell | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_stamp | 2 | 2/2 | 2/2 | 2/2 | 0 |
| timezone_headers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| titled_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| uk | 3 | 1/3 | 2/3 | 2/3 | 1 |
| uk_date_format | 2 | 1/2 | 2/2 | 1/2 | 0 |
| unsigned_department_signoff | 1 | 1/1 | 1/1 | 1/1 | 0 |
| untitled_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| utf8_bom | 1 | – | – | 0/1 | 1 |
| warning_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 2 | 2/2 | 2/2 | 2/2 | 0 |
| word_doc_legacy | 2 | 2/2 | 2/2 | 2/2 | 0 |
| word_docx | 4 | 3/4 | 4/4 | 3/4 | 1 |
| xls | 6 | 4/6 | 6/6 | 6/6 | 0 |

## Routes

- **Pages:** fast 66 · layout 18 · ocr 74 · ocr_regions 10
- **Documents by route class:** fast 27 · layout 9 · ocr 20 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 57 | 57.09 s | 96.42 s | 125.73 s |
| `extraction_wall_ms` | 57 | 5.9 ms | 18.22 s | 50.29 s |
| `worker_total_ms` | 57 | 5.4 ms | 18.21 s | 50.28 s |
| `worker_snapshot_ms` | 57 | 0.20 ms | 0.90 ms | 3.5 ms |
| `worker_parse_ms` | 57 | 2.0 ms | 32.2 ms | 1.66 s |
| `worker_analysis_ms` | 57 | 0.23 ms | 3.7 ms | 15.6 ms |
| `worker_render_ms` | 57 | 0.00 ms | 1.19 s | 2.62 s |
| `worker_image_decode_ms` | 57 | 0.00 ms | 54.6 ms | 429.8 ms |
| `worker_ocr_ms` | 57 | 0.00 ms | 35.63 s | 98.76 s |
| `worker_ocr_encode_ms` | 57 | 0.00 ms | 2.49 s | 7.69 s |
| `worker_ocr_engine_ms` | 57 | 0.00 ms | 34.63 s | 90.76 s |
| `worker_vision_ms` | 57 | 0.00 ms | 5.3 ms | 28.5 ms |
| `analyze_wall_ms` | 57 | 55.90 s | 87.62 s | 96.42 s |
| `distill_ms` | 57 | 2.0 ms | 32.9 ms | 50.4 ms |
| `prompt_ms` | 57 | 0.03 ms | 0.05 ms | 0.06 ms |
| `inference_ms` | 57 | 55.88 s | 87.61 s | 96.34 s |
| `prefill_ms` | 57 | 31.84 s | 63.05 s | 73.80 s |
| `generation_ms` | 57 | 22.93 s | 30.32 s | 35.15 s |
| `validation_ms` | 57 | 6.2 ms | 19.8 ms | 47.7 ms |
| `naming_ms` | 57 | 0.25 ms | 3.2 ms | 6.0 ms |
| `index_ms` | 57 | 1.8 ms | 29.0 ms | 42.8 ms |
| `retrieval_ms` | 57 | 0.04 ms | 2.5 ms | 6.6 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 57 | 36.9 tok/s | 37.4 tok/s | 37.6 tok/s |
| `generation_tok_per_s` | 57 | 4.0 tok/s | 4.3 tok/s | 4.4 tok/s |
| `prompt_tokens` | 57 | 1175 | 2322 | 2754 |
| `cached_tokens` | 57 | 477 | 497 | 497 |
| `generated_tokens` | 57 | 94 | 122 | 137 |
| `estimated_prompt_tokens` | 57 | 1668 | 3249 | 3525 |
| `model_requests` | 57 | 1 | 1 | 1 |
| `redistillations` | 57 | 0 | 0 | 0 |
| `source_characters` | 57 | 3231 | 55143 | 91065 |
| `digest_characters` | 57 | 2899 | 8058 | 8523 |
| `prompt_characters` | 57 | 4948 | 10112 | 10577 |
| `worker_ocr_pages` | 57 | 0 | 10 | 23 |
| `worker_ocr_passes` | 57 | 0 | 10 | 23 |
| `worker_orientation_passes` | 57 | 0 | 10 | 23 |
| `worker_rendered_pixels` | 57 | 0 | 84150000 | 193545000 |
| `index_units` | 57 | 39 | 307 | 781 |
| `context_units` | 57 | 32 | 62 | 71 |
| `context_tokens` | 57 | 1080 | 2659 | 2936 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 27 | 2.2 ms | 50.5 ms | 1.8 ms | 50.2 ms | 49.1 ms | 0.60 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 5.9 ms | 19.7 ms | 5.4 ms | 18.9 ms | 15.5 ms | 3.7 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 18.22 s | 18.22 s | 18.21 s | 18.21 s | 15.8 ms | 15.6 ms | 948.6 ms | 35.63 s |
| ocr | 20 | 2.41 s | 22.00 s | 2.41 s | 22.00 s | 11.4 ms | 2.4 ms | 1.86 s | 43.20 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 21 | 2.2 ms | 1.77 s | 1.9 ms | 1.77 s | 10.6 ms | 0.28 ms | 0.00 ms | 1.39 s |
| 2-4 | 27 | 9.2 ms | 6.37 s | 8.3 ms | 6.36 s | 49.1 ms | 2.6 ms | 513.7 ms | 11.14 s |
| 5-9 | 5 | 4.7 ms | 22.00 s | 4.1 ms | 22.00 s | 9.6 ms | 2.3 ms | 1.07 s | 43.20 s |
| 10-24 | 4 | 15.52 s | 50.29 s | 15.52 s | 50.28 s | 32.2 ms | 15.6 ms | 2.62 s | 98.76 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 947.9 ms | 18.22 s | 947.5 ms | 18.21 s | 15.8 ms | 15.6 ms | 1.19 s | 35.63 s |
| letter | 20 | 4.4 ms | 22.00 s | 3.8 ms | 22.00 s | 49.1 ms | 1.8 ms | 1.07 s | 43.20 s |
| notice | 18 | 4.3 ms | 15.52 s | 3.9 ms | 15.52 s | 32.2 ms | 4.1 ms | 1.86 s | 30.07 s |
| receipt | 1 | 605.4 ms | 605.4 ms | 605.1 ms | 605.1 ms | 6.4 ms | 0.06 ms | 0.00 ms | 538.9 ms |
| statement | 1 | 1.6 ms | 1.6 ms | 0.94 ms | 0.94 ms | 0.69 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 563 MB | 1485 MB | real-invoice-13 |
| Model server | 6085 MB | 6135 MB | real-invoice-08 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (21)

- **real-letter-02**: expected `2012-06-19 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-06-19 Letter to Antitrust Division.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-03**: expected `2012-11-30 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-11-30 Letter to Department of Justice.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `2011-04-27 EPA Approval of Closure Plan for Kincaid Project Group. Inc.pdf` · review: LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-08 Letter from Toledo Refining Company LLC.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-03-14 Letter to TPWR Developer, LLC.pdf` · trap: date 2025-03-14 (second amendment of the report) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-09**: expected `2020-04-16 Business Review Letter to Association of Independent Commercial Producers.pdf`, got `2020-04-16 Business Review Letter from U.S. Department of Justice.pdf` · review: LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Letter from CMA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-12**: expected `2024-05-21 Letter to Explain the Need for Redactions.pdf`, got `2024-05-21 Letter to Explain the Need for Redactions to Hon. Jessica G. L. Clarke.pdf` · trap: party Jessica G. L. Clarke (judge to whom it is addressed and who endorses it) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `2024-06-10 Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · trap: date 2024-06-10 (date of the final judgment or order being appealed) · review: DATE_AMBIGUOUS
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `2025-10-20 Document from Competition and Markets Authority.pdf` · trap: date 2025-10-20 (date of the CMA's provisional notice) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-14**: expected `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN, Inc.txt`, got `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN Inc.txt` · review: LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document - WOODWRTH&CO.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles.jpg` · review: TYPE_INFERRED, DATE_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, DESCRIPTION_INVALID, PARSER_WARNING
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `1996-08-28 Document from Vernon Computer Rentals & Leaging.tif` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-12**: expected `2012-10-28 Official Billing Invoice from WPXI.pdf`, got `2012-10-29 Pittsburgh (WPXI) OFFICIAL BILLING INVOICE from Main Street Media Group.pdf` · trap: date 2012-10-29 ('Gen. Date' and 'Officially printed on' date); party Main Street Media Group (agency / billing address) · review: LOW_CONFIDENCE
- **real-office-08**: expected `2020-09-10 Letter to Labour Party.docx`, got `2020-09-10 Letter to Angela Rayner MP.docx` · trap: party Angela Rayner (party chair addressed by name) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-office-11**: expected `2001-04-23 Invoice Dispute Letter to Enron North America.xls`, got `2001-04-23 INVOICE from Enron North America.xls` · review: LOW_CONFIDENCE
- **real-office-14**: expected `2001-11-26 Outstanding Invoices Statement from Locke Liddell & Sapp LLP.xls`, got `2001-11-26 Document from Locke Liddell & Sapp LLP.xls` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-15**: expected `2002-03-20 Ship Notification for Enron Corp.eml`, got `2002-03-20 UPS Ship Notification for Richard Randall Lillian Vernon Corporation.eml` · trap: party Richard Randall (consignee contact person) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
