# InternBench: live run

- **Run:** 2026-11-23T16:48:57Z · commit `70ce2164a1`
- **Machine:** AMD EPYC 7763 64-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 58 documents · 58 completed · gold `3a6d11e305`
- **Wall time:** 3397.05 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 31/56 (55.4%) |
| Document type | 47/58 (81.0%) |
| Date | 49/57 (86.0%) |
| Date role (when the reviewed date was chosen) | 21/48 (43.8%) |
| Parties | 43/58 (74.1%) |
| Relation word (when parties were named) | 47/56 (83.9%) |
| Party in the right role (when parties were named) | 14/30 (46.7%) |
| Ready / review routing | 5/43 (11.6%) |
| Description states nothing false | 58/58 (100.0%) |
| Description is specific | 44/58 (75.9%) |
| Description specificity | 66.7% (mean of 58) |
| Evidence recall (model's quotes) | 71.0% (mean of 58) |
| Digest recall (facts reaching the digest) | 86.7% (mean of 58) |
| Prompt recall (facts reaching the prompt sent) | 86.7% (mean of 58) |
| Sent to review | 93.1% of 58 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 4 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 50.0% | 58 |
| Review rate | 93.1% | 58 |
| Evidence recall | 71.0% | 58 |
| Total latency p50 | 52.91 s | 58 |
| Total latency p95 | 107.72 s | 58 |
| Generation latency p50 | 11.37 s | 58 |
| Generation latency p95 | 16.00 s | 58 |
| Generated tokens p50 | 90 | 58 |
| Generated tokens p95 | 130 | 58 |
| Prompt tokens p50 | 1174 | 58 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 56 |
| Trap date chosen | 4 of 58 |
| Forbidden party named | 4 of 58 |
| Spurious parties named | 13 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 151 (0.0%) |
| Right name sent to review anyway | 24 of 41 |
| Sent to review for an unsupported fact | 29 of 58 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 7/16 | 13/17 | 9/17 | 2/13 | 94.1% | 44.76 s | 83.55 s |
| letter | 20 | 8/19 | 18/19 | 18/20 | 1/13 | 95.0% | 53.11 s | 105.63 s |
| notice | 18 | 16/18 | 17/18 | 16/18 | 2/17 | 88.9% | 57.79 s | 109.29 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 13.80 s | 15.89 s |
| statement | 1 | 0/1 | 1/1 | 0/1 | – | 100.0% | 86.93 s | 86.93 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 9 | 2/8 | 3/9 | 3/9 | 1/4 | 100.0% | 27.24 s | 44.76 s |
| native | 17 | 9/17 | 15/17 | 13/17 | 2/13 | 88.2% | 53.10 s | 92.48 s |
| office | 14 | 9/14 | 14/14 | 12/14 | 0/11 | 92.9% | 58.43 s | 86.93 s |
| scan | 12 | 7/12 | 12/12 | 9/12 | 1/10 | 91.7% | 65.14 s | 144.13 s |
| text | 6 | 4/5 | 5/5 | 6/6 | 1/5 | 100.0% | 53.11 s | 109.29 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 9/20 | 15/21 | 13/22 | 2/14 | 95.5% | 34.77 s | 107.72 s |
| 2-4 | 27 | 17/27 | 26/27 | 21/27 | 1/21 | 96.3% | 53.84 s | 79.23 s |
| 5-9 | 5 | 3/5 | 5/5 | 5/5 | 0/5 | 100.0% | 79.27 s | 105.63 s |
| 10-24 | 4 | 2/4 | 3/4 | 4/4 | 2/3 | 50.0% | 92.48 s | 144.13 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 4 | 2/4 | 3/4 | 4/4 | 2/3 | 0 of 4 | 92.48 s | 144.13 s | 10.36 s | 17.80 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_counsel | 3 | 1/2 | 2/2 | 3/3 | 1 |
| addressed_to_government | 3 | 0/3 | 3/3 | 3/3 | 0 |
| adopted_vs_released | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_billing | 4 | 1/4 | 4/4 | 1/4 | 2 |
| agency_in_account_with | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agent_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| aging_table | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| appended_statements | 1 | 1/1 | 1/1 | 1/1 | 0 |
| approval_letter | 2 | 0/2 | 1/2 | 1/2 | 1 |
| as_of_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attachment_note | 2 | 2/2 | 2/2 | 2/2 | 0 |
| automated_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bankruptcy | 3 | 3/3 | 3/3 | 3/3 | 0 |
| blank_date | 1 | – | 0/1 | 0/1 | 0 |
| born_digital | 16 | 8/16 | 14/16 | 12/16 | 3 |
| business_review_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| c_o_customer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 1/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_list | 2 | 0/2 | 1/2 | 1/2 | 1 |
| company_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 43 | 23/42 | 37/42 | 32/43 | 7 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filing | 11 | 10/11 | 11/11 | 9/11 | 0 |
| da_number | 3 | 1/2 | 2/2 | 3/3 | 1 |
| damaged | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dataset_footer_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 3 | 1/3 | 3/3 | 3/3 | 0 |
| date_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dd_mm_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dual_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| due_date_trap | 2 | 1/2 | 2/2 | 1/2 | 1 |
| ecf_header | 5 | 4/5 | 5/5 | 4/5 | 0 |
| ecf_header_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email | 1 | 1/1 | 1/1 | 1/1 | 0 |
| email_addresses | 1 | 0/1 | 0/1 | 1/1 | 1 |
| email_printout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eml | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enron | 7 | 4/7 | 7/7 | 5/7 | 2 |
| exhibit_attached | 2 | 2/2 | 2/2 | 2/2 | 0 |
| faint | 1 | 0/1 | 0/1 | 0/1 | 0 |
| first_name_salutation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| footer_codes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 3 | 0/3 | 3/3 | 3/3 | 0 |
| footnotes_inline | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 2/2 | 0/2 | 0 |
| former_company_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government | 7 | 5/7 | 6/7 | 7/7 | 1 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 1/1 | 0/1 | 0 |
| government_regulator | 3 | 2/3 | 3/3 | 3/3 | 0 |
| government_sender | 8 | 2/7 | 6/7 | 6/8 | 2 |
| handwritten | 7 | 1/6 | 2/7 | 2/7 | 1 |
| handwritten_signature | 5 | 5/5 | 5/5 | 4/5 | 0 |
| hearing_date_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historical | 5 | 0/4 | 0/5 | 1/5 | 1 |
| illegible_labels | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 9 | 2/8 | 3/9 | 3/9 | 1 |
| in_re_caption | 3 | 3/3 | 3/3 | 3/3 | 0 |
| inconsistent_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 17 | 7/16 | 13/17 | 9/17 | 4 |
| invoice_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice_reference | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 0/1 | 0 |
| issuer_in_remit_to | 1 | 0/1 | 1/1 | 0/1 | 1 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| law_firm_letterhead | 2 | 1/2 | 2/2 | 2/2 | 0 |
| letter | 21 | 9/20 | 19/20 | 19/21 | 2 |
| letter_in_spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_to_judge | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 4 | 2/4 | 3/4 | 4/4 | 1 |
| long_lines | 1 | – | – | 1/1 | 1 |
| many_attorneys | 3 | 3/3 | 3/3 | 3/3 | 0 |
| many_dates | 6 | 2/6 | 6/6 | 2/6 | 2 |
| many_deadlines | 1 | 0/1 | 1/1 | 1/1 | 0 |
| memo_endorsed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| metadata_date_trap | 5 | 3/5 | 5/5 | 3/5 | 2 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| missing_date | 1 | – | – | 1/1 | 1 |
| missing_letterhead | 1 | – | – | 1/1 | 1 |
| missing_logo | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 1/4 | 4/4 | 1/4 | 2 |
| multi_frame | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_page | 6 | 2/6 | 6/6 | 4/6 | 1 |
| multi_sheet | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_addressees | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_defendants | 4 | 4/4 | 4/4 | 3/4 | 0 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| needs_review | 1 | – | 0/1 | 0/1 | 0 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_title | 4 | 0/4 | 0/4 | 1/4 | 1 |
| no_invoice_date_field | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| no_typed_signature | 1 | 0/1 | 1/1 | 1/1 | 0 |
| non_ascii_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| non_utf8_encoding | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice | 18 | 16/18 | 17/18 | 16/18 | 1 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odd_text_spacing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 1 | 1/1 | 1/1 | 1/1 | 0 |
| office_format | 8 | 6/8 | 8/8 | 8/8 | 0 |
| office_holder_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| one_page | 2 | 1/2 | 2/2 | 1/2 | 0 |
| ordinal_dates | 1 | 1/1 | 1/1 | 0/1 | 0 |
| organisation_sender | 1 | 1/1 | 1/1 | 1/1 | 0 |
| parent_company_mention | 1 | 0/1 | 1/1 | 0/1 | 1 |
| parent_company_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| payment_stub | 1 | 1/1 | 1/1 | 1/1 | 0 |
| penalty | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_only_date | 1 | – | 0/1 | 0/1 | 0 |
| person_in_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| placeholder_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholder_defendants | 1 | 1/1 | 1/1 | 1/1 | 0 |
| plain_text | 5 | 3/4 | 4/4 | 5/5 | 1 |
| published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line_names_client | 1 | 0/1 | 1/1 | 1/1 | 0 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| references_other_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 2/3 | 2/3 | 3/3 | 1 |
| release_date_equals_letter_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| request_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| response_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| rtf | 1 | 0/1 | 1/1 | 1/1 | 0 |
| scanned | 15 | 9/15 | 15/15 | 11/15 | 1 |
| secretariat_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| short | 2 | 1/2 | 2/2 | 2/2 | 0 |
| signatory_trap | 9 | 6/9 | 9/9 | 9/9 | 0 |
| signed_for | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 6 | 3/6 | 6/6 | 4/6 | 2 |
| staff_names | 2 | 1/2 | 2/2 | 1/2 | 1 |
| stamp | 7 | 5/7 | 5/7 | 5/7 | 1 |
| statement | 1 | 0/1 | 1/1 | 0/1 | 1 |
| stylised_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tabular | 2 | 0/2 | 2/2 | 0/2 | 1 |
| tax | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text_date_cell | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_stamp | 2 | 2/2 | 2/2 | 2/2 | 0 |
| timezone_headers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| titled_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_addressees | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_senders | 1 | 1/1 | 1/1 | 1/1 | 0 |
| type_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 1/1 | 1/1 | 1/1 | 0 |
| uk | 3 | 1/3 | 2/3 | 2/3 | 1 |
| uk_date_format | 2 | 1/2 | 2/2 | 1/2 | 0 |
| unsigned_department_signoff | 1 | 1/1 | 1/1 | 1/1 | 0 |
| untitled_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| utf8_bom | 1 | – | – | 1/1 | 1 |
| warning_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 2 | 1/2 | 2/2 | 1/2 | 1 |
| word_doc_legacy | 2 | 2/2 | 2/2 | 2/2 | 0 |
| word_docx | 4 | 3/4 | 4/4 | 4/4 | 0 |
| xls | 6 | 3/6 | 6/6 | 4/6 | 2 |

## Routes

- **Pages:** fast 66 · layout 18 · ocr 75 · ocr_regions 10
- **Documents by route class:** fast 27 · layout 9 · ocr 21 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 58 | 52.91 s | 107.72 s | 144.13 s |
| `extraction_wall_ms` | 58 | 7.7 ms | 20.42 s | 75.10 s |
| `worker_total_ms` | 58 | 7.2 ms | 20.42 s | 75.10 s |
| `worker_snapshot_ms` | 58 | 0.26 ms | 1.8 ms | 4.3 ms |
| `worker_parse_ms` | 58 | 3.3 ms | 44.7 ms | 405.3 ms |
| `worker_analysis_ms` | 58 | 0.22 ms | 4.5 ms | 19.8 ms |
| `worker_render_ms` | 58 | 0.00 ms | 1.62 s | 3.34 s |
| `worker_image_decode_ms` | 58 | 0.00 ms | 82.1 ms | 535.6 ms |
| `worker_ocr_ms` | 58 | 0.00 ms | 39.84 s | 147.71 s |
| `worker_ocr_encode_ms` | 58 | 0.00 ms | 2.90 s | 8.94 s |
| `worker_ocr_engine_ms` | 58 | 0.00 ms | 38.15 s | 138.39 s |
| `worker_vision_ms` | 58 | 0.00 ms | 5.9 ms | 35.8 ms |
| `analyze_wall_ms` | 58 | 52.49 s | 92.43 s | 109.29 s |
| `distill_ms` | 58 | 2.4 ms | 40.4 ms | 62.3 ms |
| `prompt_ms` | 58 | 0.04 ms | 0.06 ms | 0.09 ms |
| `inference_ms` | 58 | 52.48 s | 92.35 s | 109.19 s |
| `prefill_ms` | 58 | 41.10 s | 81.98 s | 97.61 s |
| `generation_ms` | 58 | 11.37 s | 16.00 s | 17.80 s |
| `validation_ms` | 58 | 6.9 ms | 34.7 ms | 61.5 ms |
| `naming_ms` | 58 | 0.28 ms | 3.9 ms | 7.3 ms |
| `index_ms` | 58 | 2.2 ms | 35.4 ms | 53.1 ms |
| `retrieval_ms` | 58 | 0.04 ms | 3.0 ms | 8.0 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 58 | 28.5 tok/s | 28.8 tok/s | 29.0 tok/s |
| `generation_tok_per_s` | 58 | 8.0 tok/s | 8.4 tok/s | 8.5 tok/s |
| `prompt_tokens` | 58 | 1174 | 2322 | 2754 |
| `cached_tokens` | 58 | 477 | 497 | 497 |
| `generated_tokens` | 58 | 90 | 130 | 141 |
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
| fast | 27 | 3.1 ms | 67.2 ms | 2.4 ms | 66.9 ms | 65.2 ms | 0.71 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 7.7 ms | 26.0 ms | 7.2 ms | 25.3 ms | 21.3 ms | 4.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 20.19 s | 20.19 s | 20.18 s | 20.18 s | 22.6 ms | 19.8 ms | 1.29 s | 39.45 s |
| ocr | 21 | 3.15 s | 31.33 s | 3.15 s | 31.33 s | 24.3 ms | 3.1 ms | 2.45 s | 61.60 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 3.4 ms | 2.11 s | 2.7 ms | 2.11 s | 22.4 ms | 0.32 ms | 0.00 ms | 1.67 s |
| 2-4 | 27 | 13.0 ms | 7.26 s | 10.8 ms | 7.25 s | 65.2 ms | 4.2 ms | 684.6 ms | 13.22 s |
| 5-9 | 5 | 5.9 ms | 31.33 s | 5.3 ms | 31.33 s | 13.6 ms | 2.9 ms | 1.51 s | 61.60 s |
| 10-24 | 4 | 20.19 s | 75.10 s | 20.18 s | 75.10 s | 44.7 ms | 19.8 ms | 3.34 s | 147.71 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 17 | 1.12 s | 20.19 s | 1.12 s | 20.18 s | 27.4 ms | 19.8 ms | 1.62 s | 39.45 s |
| letter | 20 | 5.5 ms | 31.33 s | 5.0 ms | 31.33 s | 65.2 ms | 2.7 ms | 1.51 s | 61.60 s |
| notice | 18 | 5.5 ms | 20.42 s | 5.2 ms | 20.42 s | 44.7 ms | 4.9 ms | 2.45 s | 39.84 s |
| receipt | 2 | 605.5 ms | 677.6 ms | 605.1 ms | 677.2 ms | 12.1 ms | 0.07 ms | 0.00 ms | 580.6 ms |
| statement | 1 | 2.6 ms | 2.6 ms | 1.9 ms | 1.9 ms | 1.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 585 MB | 1457 MB | real-invoice-13 |
| Model server | 3266 MB | 3317 MB | real-invoice-08 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (25)

- **real-letter-02**: expected `2012-06-19 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-06-19 Letter to Antitrust Division.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-03**: expected `2012-11-30 Business Review Request Letter to U.S. Department of Justice, Antitrust Division.pdf`, got `2012-11-30 Letter to Department of Justice.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `2011-04-27 EPA Approval of Closure Plan. Former Kincaid Park Biathlon Range Site, Anchorage.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-08 Letter from Toledo Refining Company LLC.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-03-14 Approval of SEP Completion Report for TPWR Developer, LLC.pdf` · trap: date 2025-03-14 (second amendment of the report) · review: LOW_CONFIDENCE
- **real-letter-09**: expected `2020-04-16 Business Review Letter to Association of Independent Commercial Producers.pdf`, got `2020-04-16 Letter from U.S. Department of Justice.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Letter to CMA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real-letter-14**: expected `2013-08-09 Decision Letter to Convergence Entertainment and Communications, LLC.txt`, got `2013-08-09 Notice of Apparent Liability for Forfeiture for Convergence Entertainment and Communications, LLC.txt` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `2024-08-09 Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `2025-10-20 Final enforcement notice for Euro Car Parks Limited.pdf` · trap: date 2025-10-20 (date of the CMA's provisional notice) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document between Abwoodworth. Jm.Wa and E.B.WOODWORTH.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document from Jacoby Brothers Wholesale Department.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles from CAPYTOL.jpg` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-04**: expected `1859-01-13 Receipt from Leesburg Washingtonian.jpg`, got `Document - Dr.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document from Snyder DRSGH.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `1996-08-28 Document from Vernon Computer Rentals & Leaging.tif` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-invoice-11**: expected `2010-11-29 Invoice from C-SPAN Archives.pdf`, got `2010-11-29 Invoice from orders@c-spanarchives.org.pdf` · review: LOW_CONFIDENCE
- **real-invoice-12**: expected `2012-10-28 Official Billing Invoice from WPXI.pdf`, got `2012-10-28 Wpxi Official Billing Invoice from Main Street Media Group.pdf` · trap: party Main Street Media Group (agency / billing address) · review: LOW_CONFIDENCE
- **real-invoice-14**: expected `2025-03-30 Invoice from WRAL.pdf`, got `2025-03-30 Invoice.pdf` · review: PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real-invoice-15**: expected `2025-02-23 Invoice from WBTV.pdf`, got `2025-02-23 Invoice from Club for Growth Action.pdf` · trap: party Club for Growth Action (advertiser) · review: LOW_CONFIDENCE
- **real-office-01**: expected `2024-01-03 License Renewal Letter to Common Frequency, Inc.docx`, got `2024-01-03 Certified Mail Return Receipt Requested from FCC.docx` · review: LOW_CONFIDENCE
- **real-office-04**: expected `2011-07-11 Letter to Ofcom.rtf`, got `2011-07-11 Letter from OFCOM.rtf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-office-09**: expected `2001-02-08 Invoice from California Power Exchange.xls`, got `2001-02-08 INVOICE from Bank of America.xls` · trap: party Bank of America (remit-to bank) · review: LOW_CONFIDENCE
- **real-office-11**: expected `2001-04-23 Invoice Dispute Letter to Enron North America.xls`, got `2001-04-23 Invoice from Enron North America.xls` · **filed without review**
- **real-office-14**: expected `2001-11-26 Outstanding Invoices Statement from Locke Liddell & Sapp LLP.xls`, got `2001-11-26 Document - Enron Corp.xls` · trap: party Enron Corp. (client alone) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
