# InternBench: live run

- **Run:** 2026-11-21T04:50:18Z · commit `a51447d321`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 58 documents · 58 completed · gold `3a6d11e305`
- **Wall time:** 834.29 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 14/56 (25.0%) |
| Document type | 45/58 (77.6%) |
| Date | 38/57 (66.7%) |
| Date role (when the reviewed date was chosen) | 8/38 (21.1%) |
| Parties | 33/58 (56.9%) |
| Relation word (when parties were named) | 42/55 (76.4%) |
| Party in the right role (when parties were named) | 8/31 (25.8%) |
| Ready / review routing | 2/43 (4.7%) |
| Description states nothing false | 58/58 (100.0%) |
| Description is specific | 35/58 (60.3%) |
| Description specificity | 58.6% (mean of 58) |
| Evidence recall (model's quotes) | 17.8% (mean of 58) |
| Digest recall (facts reaching the digest) | 86.7% (mean of 58) |
| Prompt recall (facts reaching the prompt sent) | 86.7% (mean of 58) |
| Sent to review | 100.0% of 58 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 0.0% | 4 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 74.1% | 58 |
| Review rate | 100.0% | 58 |
| Evidence recall | 17.8% | 58 |
| Total latency p50 | 11.28 s | 58 |
| Total latency p95 | 34.40 s | 58 |
| Generation latency p50 | 3.18 s | 58 |
| Generation latency p95 | 4.09 s | 58 |
| Generated tokens p50 | 98 | 58 |
| Generated tokens p95 | 123 | 58 |
| Prompt tokens p50 | 1174 | 58 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 56 |
| Trap date chosen | 4 of 58 |
| Forbidden party named | 9 of 58 |
| Spurious parties named | 21 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 137 (0.0%) |
| Right name sent to review anyway | 14 of 41 |
| Sent to review for an unsupported fact | 43 of 58 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 2/16 | 8/17 | 7/17 | 1/13 | 100.0% | 9.48 s | 34.40 s |
| letter | 20 | 3/19 | 13/19 | 11/20 | 1/13 | 100.0% | 11.99 s | 42.84 s |
| notice | 18 | 9/18 | 16/18 | 14/18 | 0/17 | 100.0% | 11.38 s | 33.41 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 4.06 s | 5.42 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 17.00 s | 17.00 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 9 | 1/8 | 1/9 | 3/9 | 1/4 | 100.0% | 7.14 s | 11.66 s |
| native | 17 | 5/17 | 11/17 | 11/17 | 0/13 | 100.0% | 10.59 s | 34.40 s |
| office | 14 | 3/14 | 12/14 | 7/14 | 0/11 | 100.0% | 11.38 s | 17.00 s |
| scan | 12 | 3/12 | 9/12 | 9/12 | 0/10 | 100.0% | 17.82 s | 78.02 s |
| text | 6 | 2/5 | 5/5 | 3/6 | 1/5 | 100.0% | 10.21 s | 20.75 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3/20 | 12/21 | 7/22 | 2/14 | 100.0% | 8.47 s | 20.34 s |
| 2-4 | 27 | 9/27 | 20/27 | 18/27 | 0/21 | 100.0% | 11.38 s | 18.95 s |
| 5-9 | 5 | 2/5 | 4/5 | 4/5 | 0/5 | 100.0% | 15.27 s | 42.84 s |
| 10-24 | 4 | 0/4 | 2/4 | 4/4 | 0/3 | 100.0% | 33.41 s | 78.02 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 4 | 0/4 | 2/4 | 4/4 | 0/3 | 0 of 4 | 33.41 s | 78.02 s | 2.65 s | 4.04 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_counsel | 3 | 0/2 | 2/2 | 0/3 | 4 |
| addressed_to_government | 3 | 0/3 | 3/3 | 3/3 | 0 |
| adopted_vs_released | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_billing | 4 | 1/4 | 2/4 | 2/4 | 1 |
| agency_in_account_with | 1 | 0/1 | 0/1 | 1/1 | 0 |
| agency_published_sample | 1 | 0/1 | 1/1 | 0/1 | 0 |
| agent_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| aging_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annex_table | 1 | 0/1 | 0/1 | 1/1 | 0 |
| appended_statements | 1 | 0/1 | 1/1 | 1/1 | 0 |
| approval_letter | 2 | 0/2 | 0/2 | 1/2 | 1 |
| as_of_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| attachment_note | 2 | 0/2 | 2/2 | 1/2 | 1 |
| automated_notice | 1 | 0/1 | 1/1 | 0/1 | 0 |
| bankruptcy | 3 | 3/3 | 3/3 | 3/3 | 0 |
| blank_date | 1 | – | 0/1 | 0/1 | 1 |
| born_digital | 16 | 5/16 | 10/16 | 11/16 | 5 |
| business_review_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| c_o_customer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 1/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_list | 2 | 0/2 | 0/2 | 1/2 | 1 |
| company_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 43 | 10/42 | 26/42 | 26/43 | 9 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| court_filing | 11 | 6/11 | 10/11 | 9/11 | 0 |
| da_number | 3 | 1/2 | 2/2 | 1/3 | 3 |
| damaged | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dataset_footer_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 3 | 0/3 | 2/3 | 1/3 | 2 |
| date_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dd_mm_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| deadline_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dual_salutation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| due_date_trap | 2 | 0/2 | 2/2 | 0/2 | 1 |
| ecf_header | 5 | 3/5 | 4/5 | 4/5 | 0 |
| ecf_header_stamp | 1 | 0/1 | 0/1 | 0/1 | 1 |
| email | 1 | 0/1 | 1/1 | 0/1 | 0 |
| email_addresses | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eml | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enron | 7 | 0/7 | 7/7 | 1/7 | 1 |
| exhibit_attached | 2 | 1/2 | 2/2 | 2/2 | 0 |
| faint | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 0/1 | 0/1 | 1/1 | 0 |
| footer_codes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 3 | 1/3 | 3/3 | 3/3 | 0 |
| footnotes_inline | 1 | 0/1 | 1/1 | 0/1 | 1 |
| form | 2 | 0/2 | 0/2 | 0/2 | 0 |
| former_company_name | 1 | 0/1 | 0/1 | 1/1 | 0 |
| government | 7 | 1/7 | 5/7 | 5/7 | 1 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 0/1 | 0/1 | 0 |
| government_regulator | 3 | 3/3 | 3/3 | 3/3 | 0 |
| government_sender | 8 | 2/7 | 5/7 | 3/8 | 6 |
| handwritten | 7 | 0/6 | 0/7 | 2/7 | 2 |
| handwritten_signature | 5 | 2/5 | 5/5 | 4/5 | 0 |
| hearing_date_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historical | 5 | 0/4 | 0/5 | 1/5 | 2 |
| illegible_labels | 1 | 0/1 | 0/1 | 0/1 | 0 |
| image | 9 | 1/8 | 1/9 | 3/9 | 2 |
| in_re_caption | 3 | 3/3 | 3/3 | 3/3 | 0 |
| inconsistent_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 17 | 2/16 | 8/17 | 7/17 | 4 |
| invoice_number_looks_like_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| invoice_reference | 1 | 0/1 | 1/1 | 0/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 0/1 | 0 |
| issuer_in_remit_to | 1 | 0/1 | 1/1 | 0/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| law_firm_letterhead | 2 | 0/2 | 1/2 | 1/2 | 1 |
| letter | 21 | 4/20 | 14/20 | 12/21 | 9 |
| letter_in_spreadsheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| letter_to_judge | 1 | 0/1 | 0/1 | 0/1 | 1 |
| letterhead_issuer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 1/4 | 3/4 | 4/4 | 0 |
| long_lines | 1 | – | – | 0/1 | 2 |
| many_attorneys | 3 | 2/3 | 3/3 | 3/3 | 0 |
| many_dates | 6 | 0/6 | 2/6 | 3/6 | 1 |
| many_deadlines | 1 | 1/1 | 1/1 | 1/1 | 0 |
| memo_endorsed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| metadata_date_trap | 5 | 0/5 | 5/5 | 1/5 | 1 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 1 |
| missing_date | 1 | – | – | 0/1 | 2 |
| missing_letterhead | 1 | – | – | 0/1 | 2 |
| missing_logo | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 1/4 | 2/4 | 2/4 | 1 |
| multi_frame | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_page | 6 | 0/6 | 3/6 | 4/6 | 1 |
| multi_sheet | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_addressees | 1 | 0/1 | 0/1 | 0/1 | 0 |
| multiple_defendants | 4 | 0/4 | 4/4 | 3/4 | 0 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| needs_review | 1 | – | 0/1 | 0/1 | 1 |
| network_logo | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_explicit_title | 4 | 0/4 | 0/4 | 1/4 | 1 |
| no_invoice_date_field | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_typed_signature | 1 | 0/1 | 1/1 | 1/1 | 0 |
| non_ascii_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| non_utf8_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 18 | 9/18 | 16/18 | 14/18 | 0 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odd_text_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 1 | 0/1 | 1/1 | 0/1 | 1 |
| office_format | 8 | 3/8 | 6/8 | 6/8 | 2 |
| office_holder_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| one_page | 2 | 0/2 | 1/2 | 1/2 | 0 |
| ordinal_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| organisation_sender | 1 | 0/1 | 0/1 | 1/1 | 0 |
| parent_company_mention | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| payment_stub | 1 | 0/1 | 1/1 | 0/1 | 0 |
| penalty | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_only_date | 1 | – | 0/1 | 0/1 | 1 |
| person_in_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| placeholder_data | 1 | 0/1 | 1/1 | 0/1 | 0 |
| placeholder_defendants | 1 | 0/1 | 1/1 | 1/1 | 0 |
| plain_text | 5 | 2/4 | 4/4 | 3/5 | 3 |
| published_sample | 1 | 0/1 | 1/1 | 0/1 | 0 |
| re_line_names_client | 1 | 0/1 | 1/1 | 0/1 | 1 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| redacted | 1 | 0/1 | 0/1 | 1/1 | 0 |
| references_other_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 1/3 | 2/3 | 3/3 | 0 |
| release_date_equals_letter_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| request_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| response_letter | 2 | 0/2 | 1/2 | 2/2 | 0 |
| rtf | 1 | 0/1 | 0/1 | 1/1 | 0 |
| scanned | 15 | 4/15 | 10/15 | 11/15 | 0 |
| secretariat_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 1 |
| service_period_trap | 2 | 0/2 | 2/2 | 0/2 | 1 |
| short | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_trap | 9 | 3/9 | 7/9 | 6/9 | 2 |
| signed_for | 1 | 0/1 | 0/1 | 0/1 | 0 |
| spreadsheet | 6 | 0/6 | 6/6 | 1/6 | 1 |
| staff_names | 2 | 0/2 | 2/2 | 1/2 | 0 |
| stamp | 7 | 2/7 | 4/7 | 5/7 | 1 |
| statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| stylised_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tabular | 2 | 0/2 | 1/2 | 0/2 | 1 |
| tax | 1 | 0/1 | 1/1 | 0/1 | 0 |
| text_date_cell | 1 | 0/1 | 1/1 | 0/1 | 0 |
| third_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_stamp | 2 | 1/2 | 2/2 | 2/2 | 0 |
| timezone_headers | 1 | 0/1 | 1/1 | 0/1 | 0 |
| titled_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 0/1 | 0/1 | 1/1 | 0 |
| two_senders | 1 | 0/1 | 1/1 | 0/1 | 1 |
| type_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 0/1 | 0/1 | 1/1 | 0 |
| uk | 3 | 0/3 | 1/3 | 2/3 | 1 |
| uk_date_format | 2 | 0/2 | 1/2 | 1/2 | 1 |
| unsigned_department_signoff | 1 | 0/1 | 0/1 | 1/1 | 0 |
| untitled_document | 1 | 0/1 | 1/1 | 0/1 | 0 |
| utf8_bom | 1 | – | – | 0/1 | 2 |
| warning_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 2 | 1/2 | 1/2 | 2/2 | 0 |
| word_doc_legacy | 2 | 1/2 | 2/2 | 2/2 | 0 |
| word_docx | 4 | 2/4 | 3/4 | 3/4 | 1 |
| xls | 6 | 0/6 | 6/6 | 1/6 | 1 |

## Routes

- **Pages:** fast 66 · layout 18 · ocr 75 · ocr_regions 10
- **Documents by route class:** fast 27 · layout 9 · ocr 21 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 58 | 11.28 s | 34.40 s | 78.02 s |
| `extraction_wall_ms` | 58 | 7.3 ms | 20.81 s | 65.21 s |
| `worker_total_ms` | 58 | 6.8 ms | 20.80 s | 65.20 s |
| `worker_snapshot_ms` | 58 | 0.25 ms | 1.5 ms | 4.7 ms |
| `worker_parse_ms` | 58 | 2.7 ms | 44.6 ms | 171.4 ms |
| `worker_analysis_ms` | 58 | 0.23 ms | 4.9 ms | 19.9 ms |
| `worker_render_ms` | 58 | 0.00 ms | 1.60 s | 3.33 s |
| `worker_image_decode_ms` | 58 | 0.00 ms | 78.4 ms | 541.3 ms |
| `worker_ocr_ms` | 58 | 0.00 ms | 40.71 s | 127.85 s |
| `worker_ocr_encode_ms` | 58 | 0.00 ms | 2.81 s | 8.63 s |
| `worker_ocr_engine_ms` | 58 | 0.00 ms | 39.48 s | 118.86 s |
| `worker_vision_ms` | 58 | 0.00 ms | 5.8 ms | 36.4 ms |
| `analyze_wall_ms` | 58 | 10.46 s | 17.00 s | 20.75 s |
| `distill_ms` | 58 | 2.4 ms | 40.8 ms | 62.2 ms |
| `prompt_ms` | 58 | 0.04 ms | 0.06 ms | 0.08 ms |
| `inference_ms` | 58 | 10.45 s | 16.98 s | 20.63 s |
| `prefill_ms` | 58 | 7.09 s | 14.28 s | 17.22 s |
| `generation_ms` | 58 | 3.18 s | 4.09 s | 6.13 s |
| `validation_ms` | 58 | 7.1 ms | 26.0 ms | 65.9 ms |
| `naming_ms` | 58 | 0.27 ms | 3.9 ms | 7.3 ms |
| `index_ms` | 58 | 2.2 ms | 35.5 ms | 53.5 ms |
| `retrieval_ms` | 58 | 0.04 ms | 2.9 ms | 8.0 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 58 | 163.8 tok/s | 167.3 tok/s | 168.9 tok/s |
| `generation_tok_per_s` | 58 | 30.6 tok/s | 32.3 tok/s | 33.0 tok/s |
| `prompt_tokens` | 58 | 1174 | 2322 | 2754 |
| `cached_tokens` | 58 | 477 | 497 | 497 |
| `generated_tokens` | 58 | 98 | 123 | 188 |
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
| fast | 27 | 2.8 ms | 66.8 ms | 2.4 ms | 66.4 ms | 65.0 ms | 0.69 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 7.3 ms | 25.9 ms | 6.8 ms | 25.2 ms | 21.3 ms | 4.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 20.81 s | 20.81 s | 20.80 s | 20.80 s | 22.7 ms | 19.9 ms | 1.24 s | 40.71 s |
| ocr | 21 | 2.92 s | 28.94 s | 2.92 s | 28.94 s | 23.2 ms | 3.1 ms | 2.42 s | 56.92 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 2.8 ms | 2.12 s | 2.5 ms | 2.11 s | 19.4 ms | 0.37 ms | 0.00 ms | 1.67 s |
| 2-4 | 27 | 12.7 ms | 7.33 s | 11.5 ms | 7.32 s | 65.0 ms | 3.1 ms | 687.3 ms | 12.87 s |
| 5-9 | 5 | 6.1 ms | 28.94 s | 5.5 ms | 28.94 s | 13.3 ms | 2.8 ms | 1.51 s | 56.92 s |
| 10-24 | 4 | 19.07 s | 65.21 s | 19.07 s | 65.20 s | 44.6 ms | 19.9 ms | 3.33 s | 127.85 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 1.11 s | 20.81 s | 1.11 s | 20.80 s | 26.9 ms | 19.9 ms | 1.60 s | 40.71 s |
| letter | 20 | 5.4 ms | 28.94 s | 4.9 ms | 28.94 s | 65.0 ms | 2.2 ms | 1.51 s | 56.92 s |
| notice | 18 | 5.3 ms | 19.07 s | 5.1 ms | 19.07 s | 44.6 ms | 4.9 ms | 2.42 s | 37.18 s |
| receipt | 2 | 622.3 ms | 701.1 ms | 621.9 ms | 700.7 ms | 11.3 ms | 0.09 ms | 0.00 ms | 607.0 ms |
| statement | 1 | 2.7 ms | 2.7 ms | 1.9 ms | 1.9 ms | 1.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 581 MB | 1462 MB | real-invoice-13 |
| Model server | 856 MB | 876 MB | real-office-15 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (42)

- **real-letter-02**: expected `2012-06-19 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-06-19 Letter to Morgan, Lewis & Bockius LLP.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-03**: expected `2012-11-30 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-11-30 Letter to IPXI Holdings, LLC.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `Letter from United States Environmental Protection Agency.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-08 Letter from Toledo Refining Company LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-01-31 Letter to TPWR Developer, LLC.pdf` · trap: date 2025-01-31 (date of TPWR's SEP Completion Report) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-09**: expected `2020-04-16 Business Review Letter to Association of Independent Commercial Producers.pdf`, got `2020-04-16 Letter from Makan Delrahim.pdf` · trap: party Makan Delrahim (Assistant Attorney General named in the letterhead, who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Letter from Alistair Thompson.pdf` · trap: party Alistair Thompson (CMA director named in the From line, who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-11**: expected `2018-06-20 Decision Letter to Reach plc.pdf`, got `Letter to Reach plc.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-12**: expected `2024-05-21 Letter to Explain the Need for Redactions.pdf`, got `2024-05-23 Letter to Explain the Need for Redactions.pdf` · trap: date 2024-05-23 (ECF filing stamp and date of the judge's memo endorsement) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-13**: expected `2021-11-12 Response Letter to China Telecom (Americas) Corporation.txt`, got `2021-11-12 Letter to Patrick Webre.txt` · trap: party Patrick Webre (signing bureau chief) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-02**: expected `2009-10-13 Notice of Appeal between Sompo Japan Insurance Company of America and Norfolk Southern Railway Company.pdf`, got `2009-10-13 Certificate of Service for Jenkintown, PA 19046.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real-notice-03**: expected `2009-08-31 Notice of Removal between American Tower Management, LLC and Trinity Broadcasting of New York, Inc.pdf`, got `2009-08-31 Notice of Entry between American Tower Management, LLC and Trinity Broadcasting of New York, Inc.pdf` · review: LOW_CONFIDENCE
- **real-notice-04**: expected `2009-11-02 Notice of Dismissal between Luxo AS and The Walt Disney Company.pdf`, got `2009-11-02 Notice of Dismissal between LUXO AS and The Walt Disney Company, Pixar.pdf` · review: LOW_CONFIDENCE
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · review: TYPE_INFERRED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-08**: expected `2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Google Inc.pdf`, got `2008-01-24 Document between PA ADVISORS, LLC and WPP Group plc.pdf` · review: TYPE_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-12**: expected `2018-01-08 CP161 Notice for ABC-XYZ Company.pdf`, got `2018-01-08 Notice for Department of the Treasury.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `Letter to Euro Car Parks Limited.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-15**: expected `2024-11-21 Notice of Apparent Liability for Forfeiture for Eken Group Limited.txt`, got `2024-11-21 Notice of Apparent Liability for Forfeiture from Eken Group Limited.txt` · review: LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document from Concord, N18.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document from Jacoby Brothers Wholesale Department.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles from ntor, 511 Sansome nt.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-04**: expected `1859-01-13 Receipt from Leesburg Washingtonian.jpg`, got `Document from NV19.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document from Snyder DRSGH.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-06**: expected `1992-12-15 Invoice from Perception Research Services, Inc.tif`, got `Invoice from Perception Research Services, Inc.tif` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `Invoice from Vernon Computer Rentals & Leaging.tif` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-10**: expected `2012-09-09 Invoice from KCNC TV.pdf`, got `Invoice from CBS Television Stations.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-11**: expected `2010-11-29 Invoice from C-SPAN Archives.pdf`, got `2010-11-29 Invoice from orders@c-spanarchives.org.pdf` · review: LOW_CONFIDENCE
- **real-invoice-12**: expected `2012-10-28 Official Billing Invoice from WPXI.pdf`, got `Invoice from P.O. Box 640132.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-13**: expected `2012-10-14 Invoice from WPTV.pdf`, got `Invoice from WPTV.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-14**: expected `2025-03-30 Invoice from WRAL.pdf`, got `2025-03-30 Document from Sue Duke Energy.pdf` · trap: party Duke Energy (named in the ad product 'Sue Duke Energy') · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-04**: expected `2011-07-11 Letter to Ofcom.rtf`, got `Letter to OFCOM.rtf` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-05**: expected `2023-12-14 Letter to Lord Chancellor and Secretary of State for Justice.odt`, got `2023-12-14 Letter to Chris Hodges.odt` · trap: party Chris Hodges (Chair of the Board who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-06**: expected `2017-01-04 Cover Letter to Public Accounts Committee.doc`, got `2017-01-04 Letter to Committee on Standards in Public Life.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-office-07**: expected `2020-10-26 Response Letter to Committee on Standards in Public Life.docx`, got `Letter to Committee on Standards in Public Life.docx` · review: TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-08**: expected `2020-09-10 Letter to Labour Party.docx`, got `2020-09-10 Letter to Angela Rayner MP.docx` · trap: party Angela Rayner (party chair addressed by name) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-office-09**: expected `2001-02-08 Invoice from California Power Exchange.xls`, got `2001-02-08 Invoice from 1850 Gateway Blvd.xls` · review: LOW_CONFIDENCE
- **real-office-10**: expected `2001-04-18 Invoice from Transwestern Pipeline Company.xls`, got `2001-04-18 Invoice.xls` · review: PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real-office-11**: expected `2001-04-23 Invoice Dispute Letter to Enron North America.xls`, got `2001-04-23 Invoice from 713 646-8875.xls` · review: LOW_CONFIDENCE
- **real-office-12**: expected `2000-03-31 Invoice from Varibus Corporation.xls`, got `2000-03-31 Invoice from PH# (504)576-4152.xls` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-office-13**: expected `2001-09-14 Invoice from Enron North America.xls`, got `2001-09-14 Document - NorthWestern Generation I, LLC.xls` · trap: party NorthWestern Generation I, LLC (customer alone) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- … and 2 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
