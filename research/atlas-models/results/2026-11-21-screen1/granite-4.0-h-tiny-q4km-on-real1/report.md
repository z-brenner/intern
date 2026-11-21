# InternBench: live run

- **Run:** 2026-11-21T04:49:37Z · commit `a51447d321`
- **Machine:** AMD EPYC 9V74 80-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 58 documents · 58 completed · gold `3a6d11e305`
- **Wall time:** 976.04 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 21/56 (37.5%) |
| Document type | 46/58 (79.3%) |
| Date | 44/57 (77.2%) |
| Date role (when the reviewed date was chosen) | 21/43 (48.8%) |
| Parties | 36/58 (62.1%) |
| Relation word (when parties were named) | 48/58 (82.8%) |
| Party in the right role (when parties were named) | 14/30 (46.7%) |
| Ready / review routing | 8/43 (18.6%) |
| Description states nothing false | 58/58 (100.0%) |
| Description is specific | 42/58 (72.4%) |
| Description specificity | 67.2% (mean of 58) |
| Evidence recall (model's quotes) | 34.5% (mean of 58) |
| Digest recall (facts reaching the digest) | 86.7% (mean of 58) |
| Prompt recall (facts reaching the prompt sent) | 86.7% (mean of 58) |
| Sent to review | 86.2% of 58 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 0.0% | 4 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 60.3% | 58 |
| Review rate | 86.2% | 58 |
| Evidence recall | 34.5% | 58 |
| Total latency p50 | 13.89 s | 58 |
| Total latency p95 | 35.90 s | 58 |
| Generation latency p50 | 3.71 s | 58 |
| Generation latency p95 | 4.98 s | 58 |
| Generated tokens p50 | 88 | 58 |
| Generated tokens p95 | 119 | 58 |
| Prompt tokens p50 | 1029 | 58 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 4 of 56 |
| Trap date chosen | 8 of 58 |
| Forbidden party named | 13 of 58 |
| Spurious parties named | 20 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 159 (0.0%) |
| Right name sent to review anyway | 17 of 41 |
| Sent to review for an unsupported fact | 35 of 58 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 7/16 | 12/17 | 10/17 | 4/13 | 82.3% | 11.29 s | 30.34 s |
| letter | 20 | 4/19 | 16/19 | 11/20 | 3/13 | 80.0% | 15.40 s | 42.68 s |
| notice | 18 | 10/18 | 15/18 | 14/18 | 1/17 | 94.4% | 15.21 s | 35.90 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 5.25 s | 5.92 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 18.29 s | 18.29 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 9 | 1/8 | 3/9 | 3/9 | 2/4 | 88.9% | 8.00 s | 11.90 s |
| native | 17 | 8/17 | 12/17 | 13/17 | 1/13 | 88.2% | 12.88 s | 30.34 s |
| office | 14 | 6/14 | 14/14 | 8/14 | 1/11 | 85.7% | 15.21 s | 20.08 s |
| scan | 12 | 4/12 | 10/12 | 8/12 | 2/10 | 83.3% | 20.16 s | 73.65 s |
| text | 6 | 2/5 | 5/5 | 4/6 | 2/5 | 83.3% | 14.71 s | 26.25 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 7/20 | 14/21 | 12/22 | 4/14 | 86.4% | 9.79 s | 25.31 s |
| 2-4 | 27 | 12/27 | 22/27 | 17/27 | 4/21 | 81.5% | 15.21 s | 20.67 s |
| 5-9 | 5 | 2/5 | 5/5 | 3/5 | 0/5 | 100.0% | 20.08 s | 42.68 s |
| 10-24 | 4 | 0/4 | 3/4 | 4/4 | 0/3 | 100.0% | 30.34 s | 73.65 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 4 | 0/4 | 3/4 | 4/4 | 0/3 | 0 of 4 | 30.34 s | 73.65 s | 3.71 s | 4.90 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_counsel | 3 | 2/2 | 2/2 | 3/3 | 1 |
| addressed_to_government | 3 | 0/3 | 2/3 | 3/3 | 1 |
| adopted_vs_released | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_billing | 4 | 3/4 | 3/4 | 3/4 | 2 |
| agency_in_account_with | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agency_published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agent_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| aging_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annex_table | 1 | 0/1 | 1/1 | 0/1 | 1 |
| appended_statements | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 2 | 0/2 | 1/2 | 1/2 | 1 |
| as_of_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| attachment_note | 2 | 0/2 | 2/2 | 0/2 | 2 |
| automated_notice | 1 | 0/1 | 1/1 | 1/1 | 0 |
| bankruptcy | 3 | 2/3 | 2/3 | 3/3 | 1 |
| blank_date | 1 | – | 0/1 | 1/1 | 0 |
| born_digital | 16 | 7/16 | 11/16 | 12/16 | 7 |
| business_review_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| c_o_customer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_list | 2 | 0/2 | 1/2 | 1/2 | 1 |
| company_letterhead | 1 | 0/1 | 0/1 | 1/1 | 1 |
| competing_dates | 43 | 14/42 | 32/42 | 25/43 | 20 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 11 | 6/11 | 9/11 | 8/11 | 3 |
| da_number | 3 | 1/2 | 2/2 | 2/3 | 2 |
| damaged | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dataset_footer_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 3 | 1/3 | 3/3 | 2/3 | 1 |
| date_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dd_mm_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dual_salutation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| due_date_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| ecf_header | 5 | 4/5 | 4/5 | 4/5 | 1 |
| ecf_header_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| email | 1 | 0/1 | 1/1 | 1/1 | 0 |
| email_addresses | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printout | 1 | 0/1 | 1/1 | 0/1 | 1 |
| eml | 1 | 0/1 | 1/1 | 1/1 | 0 |
| enron | 7 | 3/7 | 7/7 | 6/7 | 1 |
| exhibit_attached | 2 | 0/2 | 1/2 | 2/2 | 1 |
| faint | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footer_codes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 3 | 0/3 | 3/3 | 2/3 | 1 |
| footnotes_inline | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| former_company_name | 1 | 0/1 | 0/1 | 1/1 | 0 |
| government | 7 | 3/7 | 6/7 | 4/7 | 3 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_regulator | 3 | 2/3 | 3/3 | 2/3 | 1 |
| government_sender | 8 | 3/7 | 6/7 | 5/8 | 4 |
| handwritten | 7 | 1/6 | 2/7 | 3/7 | 1 |
| handwritten_signature | 5 | 2/5 | 5/5 | 3/5 | 1 |
| hearing_date_banner | 1 | 0/1 | 0/1 | 1/1 | 1 |
| historical | 5 | 0/4 | 0/5 | 2/5 | 1 |
| illegible_labels | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 9 | 1/8 | 3/9 | 3/9 | 2 |
| in_re_caption | 3 | 2/3 | 2/3 | 3/3 | 1 |
| inconsistent_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 0/1 | 1/1 | 0/1 | 1 |
| invoice | 17 | 7/16 | 12/17 | 10/17 | 6 |
| invoice_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice_reference | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 1/1 | 0 |
| issuer_in_remit_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 1/1 | 0 |
| law_firm_letterhead | 2 | 0/2 | 2/2 | 1/2 | 1 |
| letter | 21 | 5/20 | 17/20 | 12/21 | 10 |
| letter_in_spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_to_judge | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| long_document | 4 | 1/4 | 3/4 | 3/4 | 2 |
| long_lines | 1 | – | – | 1/1 | 1 |
| many_attorneys | 3 | 3/3 | 3/3 | 3/3 | 0 |
| many_dates | 6 | 2/6 | 5/6 | 4/6 | 2 |
| many_deadlines | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo_endorsed | 1 | 0/1 | 1/1 | 0/1 | 1 |
| metadata_date_trap | 5 | 3/5 | 5/5 | 4/5 | 1 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 1 |
| missing_date | 1 | – | – | 1/1 | 1 |
| missing_letterhead | 1 | – | – | 1/1 | 1 |
| missing_logo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| modern | 4 | 3/4 | 3/4 | 3/4 | 2 |
| multi_frame | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multi_page | 6 | 2/6 | 5/6 | 4/6 | 3 |
| multi_sheet | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_addressees | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_defendants | 4 | 1/4 | 4/4 | 2/4 | 1 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| needs_review | 1 | – | 0/1 | 1/1 | 0 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_title | 4 | 0/4 | 0/4 | 1/4 | 1 |
| no_invoice_date_field | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_typed_signature | 1 | 0/1 | 1/1 | 1/1 | 0 |
| non_ascii_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| non_utf8_encoding | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notice | 18 | 10/18 | 15/18 | 14/18 | 5 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odd_text_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 1 | 0/1 | 1/1 | 0/1 | 0 |
| office_format | 8 | 3/8 | 8/8 | 3/8 | 4 |
| office_holder_addressee | 1 | 0/1 | 1/1 | 0/1 | 0 |
| one_page | 2 | 0/2 | 1/2 | 0/2 | 1 |
| ordinal_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| organisation_sender | 1 | 0/1 | 1/1 | 0/1 | 1 |
| parent_company_mention | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| payment_stub | 1 | 1/1 | 1/1 | 1/1 | 0 |
| penalty | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_only_date | 1 | – | 0/1 | 1/1 | 0 |
| person_in_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholder_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholder_defendants | 1 | 0/1 | 1/1 | 0/1 | 0 |
| plain_text | 5 | 2/4 | 4/4 | 3/5 | 3 |
| published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line_names_client | 1 | 1/1 | 1/1 | 1/1 | 0 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| references_other_letter | 1 | 0/1 | 0/1 | 1/1 | 1 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 1/3 | 2/3 | 2/3 | 2 |
| release_date_equals_letter_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| request_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| response_letter | 2 | 0/2 | 1/2 | 1/2 | 2 |
| rtf | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 15 | 5/15 | 13/15 | 9/15 | 5 |
| secretariat_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 1 |
| service_period_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| short | 2 | 1/2 | 2/2 | 1/2 | 1 |
| signatory_trap | 9 | 3/9 | 9/9 | 4/9 | 4 |
| signed_for | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 6 | 3/6 | 6/6 | 5/6 | 1 |
| staff_names | 2 | 0/2 | 2/2 | 1/2 | 1 |
| stamp | 7 | 2/7 | 5/7 | 3/7 | 3 |
| statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| stylised_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| tabular | 2 | 1/2 | 1/2 | 1/2 | 2 |
| tax | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text_date_cell | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_stamp | 2 | 2/2 | 2/2 | 2/2 | 0 |
| timezone_headers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| titled_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| two_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_trap | 1 | 0/1 | 0/1 | 1/1 | 1 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| uk | 3 | 0/3 | 1/3 | 2/3 | 2 |
| uk_date_format | 2 | 0/2 | 1/2 | 1/2 | 1 |
| unsigned_department_signoff | 1 | 0/1 | 0/1 | 1/1 | 0 |
| untitled_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| utf8_bom | 1 | – | – | 1/1 | 1 |
| warning_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 2 | 1/2 | 2/2 | 2/2 | 0 |
| word_doc_legacy | 2 | 1/2 | 2/2 | 1/2 | 1 |
| word_docx | 4 | 1/4 | 4/4 | 1/4 | 3 |
| xls | 6 | 3/6 | 6/6 | 5/6 | 1 |

## Routes

- **Pages:** fast 66 · layout 18 · ocr 75 · ocr_regions 10
- **Documents by route class:** fast 27 · layout 9 · ocr 21 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 58 | 13.89 s | 35.90 s | 73.65 s |
| `extraction_wall_ms` | 58 | 6.2 ms | 16.37 s | 55.35 s |
| `worker_total_ms` | 58 | 5.8 ms | 16.37 s | 55.35 s |
| `worker_snapshot_ms` | 58 | 0.20 ms | 1.2 ms | 3.4 ms |
| `worker_parse_ms` | 58 | 2.3 ms | 34.3 ms | 64.1 ms |
| `worker_analysis_ms` | 58 | 0.20 ms | 3.6 ms | 16.1 ms |
| `worker_render_ms` | 58 | 0.00 ms | 1.36 s | 2.74 s |
| `worker_image_decode_ms` | 58 | 0.00 ms | 61.8 ms | 449.4 ms |
| `worker_ocr_ms` | 58 | 0.00 ms | 32.14 s | 108.63 s |
| `worker_ocr_encode_ms` | 58 | 0.00 ms | 2.58 s | 8.44 s |
| `worker_ocr_engine_ms` | 58 | 0.00 ms | 30.03 s | 99.92 s |
| `worker_vision_ms` | 58 | 0.00 ms | 5.1 ms | 33.7 ms |
| `analyze_wall_ms` | 58 | 12.82 s | 23.27 s | 26.25 s |
| `distill_ms` | 58 | 1.9 ms | 32.4 ms | 49.8 ms |
| `prompt_ms` | 58 | 0.03 ms | 0.05 ms | 0.05 ms |
| `inference_ms` | 58 | 12.80 s | 23.21 s | 26.16 s |
| `prefill_ms` | 58 | 9.23 s | 19.46 s | 22.00 s |
| `generation_ms` | 58 | 3.71 s | 4.98 s | 5.13 s |
| `validation_ms` | 58 | 6.6 ms | 27.6 ms | 53.0 ms |
| `naming_ms` | 58 | 0.25 ms | 3.1 ms | 5.8 ms |
| `index_ms` | 58 | 1.8 ms | 28.5 ms | 42.8 ms |
| `retrieval_ms` | 58 | 0.04 ms | 2.4 ms | 6.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 58 | 108.8 tok/s | 110.9 tok/s | 115.1 tok/s |
| `generation_tok_per_s` | 58 | 24.1 tok/s | 25.0 tok/s | 25.4 tok/s |
| `prompt_tokens` | 58 | 1029 | 2122 | 2396 |
| `cached_tokens` | 58 | 469 | 469 | 469 |
| `generated_tokens` | 58 | 88 | 119 | 121 |
| `estimated_prompt_tokens` | 58 | 1656 | 3249 | 3525 |
| `model_requests` | 58 | 1 | 1 | 1 |
| `redistillations` | 58 | 0 | 0 | 0 |
| `source_characters` | 58 | 3209 | 55143 | 91065 |
| `digest_characters` | 58 | 2879 | 8058 | 8523 |
| `prompt_characters` | 58 | 4928 | 10112 | 10577 |
| `worker_ocr_pages` | 58 | 0 | 10 | 23 |
| `worker_ocr_passes` | 58 | 0 | 10 | 23 |
| `worker_orientation_passes` | 58 | 0 | 10 | 23 |
| `worker_rendered_pixels` | 58 | 0 | 84150000 | 193545000 |
| `index_units` | 58 | 37 | 307 | 781 |
| `context_units` | 58 | 32 | 62 | 71 |
| `context_tokens` | 58 | 1068 | 2659 | 2936 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 27 | 2.5 ms | 58.8 ms | 2.0 ms | 58.5 ms | 57.4 ms | 0.58 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 6.2 ms | 20.9 ms | 5.8 ms | 20.3 ms | 16.9 ms | 3.6 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 15.91 s | 15.91 s | 15.91 s | 15.91 s | 16.9 ms | 16.1 ms | 980.8 ms | 31.15 s |
| ocr | 21 | 2.47 s | 24.50 s | 2.47 s | 24.50 s | 18.6 ms | 2.3 ms | 1.93 s | 48.20 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 2.9 ms | 1.62 s | 2.6 ms | 1.62 s | 15.6 ms | 0.29 ms | 0.00 ms | 1.29 s |
| 2-4 | 27 | 9.2 ms | 5.76 s | 8.5 ms | 5.76 s | 57.4 ms | 2.5 ms | 553.9 ms | 10.35 s |
| 5-9 | 5 | 4.6 ms | 24.50 s | 4.1 ms | 24.50 s | 10.5 ms | 2.3 ms | 1.36 s | 48.20 s |
| 10-24 | 4 | 15.91 s | 55.35 s | 15.91 s | 55.35 s | 34.3 ms | 16.1 ms | 2.74 s | 108.63 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 863.1 ms | 15.91 s | 862.8 ms | 15.91 s | 22.2 ms | 16.1 ms | 1.34 s | 31.15 s |
| letter | 20 | 4.3 ms | 24.50 s | 3.9 ms | 24.50 s | 57.4 ms | 1.8 ms | 1.36 s | 48.20 s |
| notice | 18 | 4.4 ms | 16.37 s | 4.1 ms | 16.37 s | 34.3 ms | 4.1 ms | 1.93 s | 32.14 s |
| receipt | 2 | 475.1 ms | 528.1 ms | 474.8 ms | 527.8 ms | 10.2 ms | 0.06 ms | 0.00 ms | 453.8 ms |
| statement | 1 | 1.9 ms | 1.9 ms | 1.3 ms | 1.3 ms | 1.1 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 566 MB | 1460 MB | real-invoice-13 |
| Model server | 4431 MB | 4431 MB | real-office-15 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (35)

- **real-letter-02**: expected `2012-06-19 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-06-19 Letter to Antitrust Division.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-03**: expected `2012-11-30 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-11-30 Letter to Department of Justice.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `2011-04-27 EPA Approval of Closure Plan for United States Environmental Protection Agency, Region 10.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-02 Warning Letter from Toledo Refining Company LLC.pdf` · trap: date 2018-03-02 (date of the PHMSA Warning Letter it answers) · review: LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-01-31 Letter to TPWR Developer, LLC.pdf` · trap: date 2025-01-31 (date of TPWR's SEP Completion Report) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Cement Market Data Order from Alistair Thompson.pdf` · trap: party Alistair Thompson (CMA director named in the From line, who signs) · **filed without review**
- **real-letter-11**: expected `2018-06-20 Decision Letter to Reach plc.pdf`, got `Letter to Reach Plc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-12**: expected `2024-05-21 Letter to Explain the Need for Redactions.pdf`, got `2024-05-21 Letter to Explain the Need for Redactions from Richard S. Mandel.pdf` · trap: party Richard S. Mandel (attorney who signs) · **filed without review**
- **real-letter-14**: expected `2013-08-09 Decision Letter to Convergence Entertainment and Communications, LLC.txt`, got `2013-08-09 Letter to Barbara A. Kreisman.txt` · trap: party Barbara A. Kreisman (Video Division chief who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real-notice-02**: expected `2009-10-13 Notice of Appeal between Sompo Japan Insurance Company of America and Norfolk Southern Railway Company.pdf`, got `2009-10-13 Notice of Appeal for Keenan Cohen & Howard P.C.pdf` · trap: party Keenan Cohen & Howard P.C. (filer's counsel) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-03**: expected `2009-08-31 Notice of Removal between American Tower Management, LLC and Trinity Broadcasting of New York, Inc.pdf`, got `2009-09-01 Document between American Tower Management, LLC and Trinity Broadcasting of New York, Inc.pdf` · review: TYPE_UNSUPPORTED
- **real-notice-04**: expected `2009-11-02 Notice of Dismissal between Luxo AS and The Walt Disney Company.pdf`, got `2009-11-02 Notice of Dismissal for The Walt Disney Company, Pixar.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `2024-06-10 Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · trap: date 2024-06-10 (date of the final judgment or order being appealed) · review: TYPE_INFERRED, DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real-notice-10**: expected `2011-10-05 Notice of Hearing for GSC Group, Inc.pdf`, got `2011-10-24 Notice of Hearing for GSC GROUP, INC.pdf` · trap: date 2011-10-24 (hearing date (banner at top of page 1)) · review: LOW_CONFIDENCE
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `2025-10-20 Final Enforcement Notice for Euro Car Parks Limited.pdf` · trap: date 2025-10-20 (date of the CMA's provisional notice) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-14**: expected `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN, Inc.txt`, got `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN Holding Company, Inc.txt` · trap: party ESPN Holding Company, Inc. (80% owner) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document - E.B.WOODWORTH.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document from Dacoby Brothers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles from Deming Palmer & Co.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-04**: expected `1859-01-13 Receipt from Leesburg Washingtonian.jpg`, got `Document from Ceesburg delashingtonian. Dr.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document from Snyder Drsgh.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `1996-08-28 Invoice from Peilip Morris USA.tif` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-08**: expected `1997-01-01 Invoice from Circulation Expertí, Ltd.tif`, got `1997-01-01 Invoice from Philip Morris USA.tif` · trap: party Philip Morris USA (customer) · **filed without review**
- **real-invoice-10**: expected `2012-09-09 Invoice from KCNC TV.pdf`, got `2012-09-09 Document from CBS Television Stations.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-11**: expected `2010-11-29 Invoice from C-SPAN Archives.pdf`, got `2010-11-29 Invoice from PUBLIC.RESOURCE.ORG.pdf` · trap: party Public.Resource.Org (customer (Bill To)) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-12**: expected `2012-10-28 Official Billing Invoice from WPXI.pdf`, got `2012-10-29 Invoice from Main Street Media Group.pdf` · trap: date 2012-10-29 ('Gen. Date' and 'Officially printed on' date); party Main Street Media Group (agency / billing address) · review: LOW_CONFIDENCE
- **real-office-01**: expected `2024-01-03 License Renewal Letter to Common Frequency, Inc.docx`, got `2024-01-03 Certified Mail Return Receipt Requested from Darrick Servis, CFO.docx` · trap: party Darrick Servis (CFO named in the address block) · review: LOW_CONFIDENCE
- **real-office-05**: expected `2023-12-14 Letter to Lord Chancellor and Secretary of State for Justice.odt`, got `2023-12-14 Letter to The Rt Hon Lord Arbuthnot of Edrom.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-06**: expected `2017-01-04 Cover Letter to Public Accounts Committee.doc`, got `2017-01-04 Letter to Lesley Bainsfair.doc` · trap: party Lesley Bainsfair (Head of Secretariat who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real-office-07**: expected `2020-10-26 Response Letter to Committee on Standards in Public Life.docx`, got `2020-10-26 Letter to Amelia Womack.docx` · trap: party Amelia Womack (Deputy Leader who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real-office-08**: expected `2020-09-10 Letter to Labour Party.docx`, got `2020-09-10 Letter to Angela Rayner MP.docx` · trap: party Angela Rayner (party chair addressed by name) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real-office-10**: expected `2001-04-18 Invoice from Transwestern Pipeline Company.xls`, got `2001-04-18 Invoice from EOG RESOURCES, INC.xls` · trap: party EOG Resources, Inc. (customer alone) · review: LOW_CONFIDENCE
- **real-office-11**: expected `2001-04-23 Invoice Dispute Letter to Enron North America.xls`, got `2001-04-23 Invoice from Enron North America.xls` · **filed without review**
- **real-office-14**: expected `2001-11-26 Outstanding Invoices Statement from Locke Liddell & Sapp LLP.xls`, got `2001-11-26 Document from Locke Liddell & Sapp LLP.xls` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-15**: expected `2002-03-20 Ship Notification for Enron Corp.eml`, got `2002-03-20 UPS Ship Notification from UPS.eml` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
