# InternBench: live run

- **Run:** 2026-11-23T16:58:44Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 53 documents · 53 completed · gold `4e0bf77af4`
- **Wall time:** 3058.15 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 33/51 (64.7%) |
| Document type | 43/53 (81.1%) |
| Date | 43/52 (82.7%) |
| Date role (when the reviewed date was chosen) | 15/42 (35.7%) |
| Parties | 40/53 (75.5%) |
| Relation word (when parties were named) | 42/49 (85.7%) |
| Party in the right role (when parties were named) | 15/26 (57.7%) |
| Ready / review routing | 7/39 (17.9%) |
| Description states nothing false | 53/53 (100.0%) |
| Description is specific | 43/53 (81.1%) |
| Description specificity | 67.3% (mean of 53) |
| Evidence recall (model's quotes) | 64.5% (mean of 53) |
| Digest recall (facts reaching the digest) | 88.3% (mean of 53) |
| Prompt recall (facts reaching the prompt sent) | 88.3% (mean of 53) |
| Sent to review | 90.6% of 53 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 45.3% | 53 |
| Review rate | 90.6% | 53 |
| Evidence recall | 64.5% | 53 |
| Total latency p50 | 56.34 s | 53 |
| Total latency p95 | 96.90 s | 53 |
| Generation latency p50 | 23.18 s | 53 |
| Generation latency p95 | 30.21 s | 53 |
| Generated tokens p50 | 93 | 53 |
| Generated tokens p95 | 122 | 53 |
| Prompt tokens p50 | 1171 | 53 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 51 |
| Trap date chosen | 5 of 53 |
| Forbidden party named | 3 of 53 |
| Spurious parties named | 8 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 125 (0.0%) |
| Right name sent to review anyway | 25 of 37 |
| Sent to review for an unsupported fact | 24 of 53 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 15 | 10/14 | 11/15 | 10/15 | 3/11 | 86.7% | 47.21 s | 79.28 s |
| letter | 17 | 9/16 | 15/16 | 13/17 | 1/11 | 100.0% | 56.34 s | 76.77 s |
| notice | 18 | 14/18 | 16/18 | 16/18 | 3/17 | 83.3% | 59.17 s | 100.78 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 12.17 s | 18.67 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 88.77 s | 88.77 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 1/7 | 2/8 | 1/8 | 1/3 | 100.0% | 18.67 s | 55.90 s |
| native | 16 | 11/16 | 13/16 | 13/16 | 2/12 | 87.5% | 56.93 s | 88.59 s |
| office | 14 | 11/14 | 14/14 | 13/14 | 1/11 | 92.9% | 59.17 s | 88.77 s |
| scan | 9 | 7/9 | 9/9 | 9/9 | 2/8 | 77.8% | 63.83 s | 99.47 s |
| text | 6 | 3/5 | 5/5 | 4/6 | 1/5 | 100.0% | 55.47 s | 100.78 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 10/20 | 14/21 | 12/22 | 3/14 | 95.5% | 39.49 s | 96.90 s |
| 2-4 | 24 | 17/24 | 23/24 | 21/24 | 2/18 | 91.7% | 57.85 s | 77.83 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 1/4 | 75.0% | 74.05 s | 81.24 s |
| 10-24 | 3 | 2/3 | 2/3 | 3/3 | 1/3 | 66.7% | 88.59 s | 99.47 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 2/3 | 2/3 | 3/3 | 1/3 | 0 of 3 | 88.59 s | 99.47 s | 24.80 s | 35.59 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| addressed_to_company | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_counsel | 2 | 1/1 | 1/1 | 1/2 | 1 |
| addressed_to_government | 1 | 0/1 | 1/1 | 1/1 | 0 |
| adopted_vs_released | 1 | 0/1 | 1/1 | 1/1 | 0 |
| agency_billing | 3 | 3/3 | 3/3 | 3/3 | 0 |
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
| born_digital | 15 | 10/15 | 12/15 | 12/15 | 4 |
| c_o_customer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_block | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| case_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| cc_list | 2 | 0/2 | 1/2 | 2/2 | 1 |
| company_letterhead | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 39 | 24/38 | 32/38 | 29/39 | 8 |
| consolidated_captions | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filed_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 11 | 10/11 | 10/11 | 10/11 | 1 |
| da_number | 3 | 2/2 | 2/2 | 2/3 | 1 |
| damaged | 1 | 0/1 | 0/1 | 0/1 | 0 |
| dataset_footer_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_at_end | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_only_in_digital_signature | 1 | 0/1 | 0/1 | 1/1 | 1 |
| date_repeated_in_page_headers | 1 | 0/1 | 1/1 | 0/1 | 1 |
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
| footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes_inline | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 1/2 | 0/2 | 1 |
| former_company_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government | 7 | 5/7 | 6/7 | 7/7 | 1 |
| government_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| government_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| government_regulator | 3 | 3/3 | 3/3 | 3/3 | 0 |
| government_sender | 7 | 3/6 | 5/6 | 5/7 | 2 |
| handwritten | 7 | 1/6 | 2/7 | 1/7 | 1 |
| handwritten_signature | 5 | 5/5 | 5/5 | 5/5 | 0 |
| hearing_date_banner | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historical | 5 | 0/4 | 0/5 | 0/5 | 1 |
| illegible_labels | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 8 | 1/7 | 2/8 | 1/8 | 1 |
| in_re_caption | 3 | 3/3 | 3/3 | 3/3 | 0 |
| inconsistent_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 0/1 | 1/1 | 1/1 | 0 |
| invoice | 15 | 10/14 | 11/15 | 10/15 | 1 |
| invoice_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice_reference | 1 | 0/1 | 1/1 | 1/1 | 0 |
| issuer_ambiguity | 1 | 0/1 | 0/1 | 0/1 | 0 |
| issuer_in_remit_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| law_firm_letterhead | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letter | 18 | 10/17 | 16/17 | 14/18 | 4 |
| letter_in_spreadsheet | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letter_to_judge | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| logo_only_issuer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 3 | 1/3 | 2/3 | 3/3 | 1 |
| long_lines | 1 | – | – | 0/1 | 1 |
| many_attorneys | 3 | 3/3 | 3/3 | 3/3 | 0 |
| many_dates | 5 | 3/5 | 5/5 | 4/5 | 0 |
| many_deadlines | 1 | 1/1 | 1/1 | 1/1 | 0 |
| memo_endorsed | 1 | 0/1 | 1/1 | 0/1 | 1 |
| metadata_date_trap | 5 | 4/5 | 5/5 | 5/5 | 0 |
| minimal_address_block | 1 | 0/1 | 1/1 | 0/1 | 0 |
| missing_date | 1 | – | – | 0/1 | 1 |
| missing_letterhead | 1 | – | – | 0/1 | 1 |
| missing_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 3 | 3/3 | 3/3 | 3/3 | 0 |
| multi_page | 4 | 4/4 | 4/4 | 4/4 | 0 |
| multi_sheet | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_addressees | 1 | 0/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 4/4 | 4/4 | 4/4 | 0 |
| multiple_plaintiffs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| needs_review | 1 | – | 0/1 | 0/1 | 0 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_title | 4 | 0/4 | 0/4 | 0/4 | 1 |
| no_invoice_date_field | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_typed_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
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
| placeholder_data | 1 | 1/1 | 1/1 | 1/1 | 0 |
| placeholder_defendants | 1 | 1/1 | 1/1 | 1/1 | 0 |
| plain_text | 5 | 3/4 | 4/4 | 4/5 | 1 |
| published_sample | 1 | 1/1 | 1/1 | 1/1 | 0 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| references_other_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| register_tape | 1 | 0/1 | 0/1 | 0/1 | 0 |
| regulatory_penalty | 3 | 1/3 | 2/3 | 3/3 | 1 |
| release_date_equals_letter_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| response_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| rtf | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 11 | 8/11 | 11/11 | 10/11 | 0 |
| secretariat_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sender_in_from_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period_trap | 2 | 2/2 | 2/2 | 2/2 | 0 |
| short | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_trap | 9 | 7/9 | 9/9 | 8/9 | 1 |
| signed_for | 1 | 0/1 | 1/1 | 1/1 | 0 |
| spreadsheet | 6 | 4/6 | 6/6 | 6/6 | 0 |
| staff_names | 2 | 1/2 | 2/2 | 2/2 | 0 |
| stamp | 6 | 4/6 | 4/6 | 4/6 | 1 |
| statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tabular | 1 | 1/1 | 1/1 | 1/1 | 0 |
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

- **Pages:** fast 62 · layout 18 · ocr 39 · ocr_regions 10
- **Documents by route class:** fast 26 · layout 9 · ocr 17 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 53 | 56.34 s | 96.90 s | 100.78 s |
| `extraction_wall_ms` | 53 | 5.4 ms | 14.76 s | 18.13 s |
| `worker_total_ms` | 53 | 5.0 ms | 14.76 s | 18.13 s |
| `worker_snapshot_ms` | 53 | 0.20 ms | 1.2 ms | 3.5 ms |
| `worker_parse_ms` | 53 | 2.4 ms | 32.1 ms | 1.59 s |
| `worker_analysis_ms` | 53 | 0.19 ms | 3.6 ms | 16.4 ms |
| `worker_render_ms` | 53 | 0.00 ms | 952.6 ms | 1.94 s |
| `worker_image_decode_ms` | 53 | 0.00 ms | 55.8 ms | 446.5 ms |
| `worker_ocr_ms` | 53 | 0.00 ms | 28.32 s | 35.33 s |
| `worker_ocr_encode_ms` | 53 | 0.00 ms | 783.3 ms | 2.42 s |
| `worker_ocr_engine_ms` | 53 | 0.00 ms | 27.13 s | 34.32 s |
| `worker_vision_ms` | 53 | 0.00 ms | 5.4 ms | 28.1 ms |
| `analyze_wall_ms` | 53 | 55.47 s | 88.77 s | 100.78 s |
| `distill_ms` | 53 | 1.9 ms | 32.9 ms | 50.7 ms |
| `prompt_ms` | 53 | 0.03 ms | 0.05 ms | 0.07 ms |
| `inference_ms` | 53 | 55.45 s | 88.76 s | 100.70 s |
| `prefill_ms` | 53 | 32.11 s | 63.69 s | 76.90 s |
| `generation_ms` | 53 | 23.18 s | 30.21 s | 35.59 s |
| `validation_ms` | 53 | 5.3 ms | 20.0 ms | 48.0 ms |
| `naming_ms` | 53 | 0.23 ms | 3.2 ms | 6.1 ms |
| `index_ms` | 53 | 1.8 ms | 29.1 ms | 43.3 ms |
| `retrieval_ms` | 53 | 0.04 ms | 2.5 ms | 6.7 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 53 | 36.5 tok/s | 37.0 tok/s | 37.2 tok/s |
| `generation_tok_per_s` | 53 | 4.1 tok/s | 4.3 tok/s | 4.3 tok/s |
| `prompt_tokens` | 53 | 1171 | 2322 | 2754 |
| `cached_tokens` | 53 | 477 | 497 | 497 |
| `generated_tokens` | 53 | 93 | 122 | 137 |
| `estimated_prompt_tokens` | 53 | 1649 | 3249 | 3525 |
| `model_requests` | 53 | 1 | 1 | 1 |
| `redistillations` | 53 | 0 | 0 | 0 |
| `source_characters` | 53 | 3187 | 46165 | 91065 |
| `digest_characters` | 53 | 2879 | 8058 | 8523 |
| `prompt_characters` | 53 | 4928 | 10112 | 10577 |
| `worker_ocr_pages` | 53 | 0 | 4 | 12 |
| `worker_ocr_passes` | 53 | 0 | 5 | 15 |
| `worker_orientation_passes` | 53 | 0 | 4 | 12 |
| `worker_rendered_pixels` | 53 | 0 | 33660000 | 100225682 |
| `index_units` | 53 | 37 | 196 | 781 |
| `context_units` | 53 | 32 | 62 | 71 |
| `context_tokens` | 53 | 1062 | 2659 | 2936 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 26 | 2.1 ms | 38.9 ms | 1.0 ms | 37.8 ms | 32.1 ms | 0.32 ms | 0.00 ms | 0.00 ms |
| layout | 9 | 8.9 ms | 1.60 s | 8.0 ms | 1.59 s | 1.59 s | 3.6 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 18.13 s | 18.13 s | 18.13 s | 18.13 s | 15.9 ms | 16.4 ms | 952.6 ms | 35.33 s |
| ocr | 17 | 2.15 s | 16.18 s | 2.15 s | 16.18 s | 12.5 ms | 2.6 ms | 1.94 s | 31.46 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 22 | 2.9 ms | 1.81 s | 1.9 ms | 1.81 s | 11.6 ms | 0.29 ms | 0.00 ms | 1.42 s |
| 2-4 | 24 | 5.7 ms | 5.14 s | 5.3 ms | 5.14 s | 49.1 ms | 2.6 ms | 512.0 ms | 9.22 s |
| 5-9 | 4 | 4.5 ms | 13.6 ms | 3.9 ms | 12.9 ms | 9.7 ms | 2.3 ms | 0.00 ms | 0.00 ms |
| 10-24 | 3 | 16.18 s | 18.13 s | 16.18 s | 18.13 s | 32.1 ms | 16.4 ms | 1.94 s | 35.33 s |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 15 | 914.3 ms | 18.13 s | 913.9 ms | 18.13 s | 15.9 ms | 16.4 ms | 1.19 s | 35.33 s |
| letter | 17 | 3.7 ms | 5.14 s | 3.3 ms | 5.14 s | 49.1 ms | 1.8 ms | 512.0 ms | 9.22 s |
| notice | 18 | 4.7 ms | 16.18 s | 4.1 ms | 16.18 s | 1.59 s | 4.1 ms | 1.94 s | 31.46 s |
| receipt | 2 | 577.3 ms | 621.2 ms | 576.9 ms | 620.9 ms | 8.0 ms | 0.07 ms | 0.00 ms | 553.0 ms |
| statement | 1 | 1.5 ms | 1.5 ms | 0.86 ms | 0.86 ms | 0.65 ms | 0.00 ms | 0.00 ms | 0.00 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 469 MB | 1383 MB | real-invoice-13 |
| Model server | 6085 MB | 6135 MB | real-invoice-10 |
| Benchmark (engine) | 16 MB | 20 MB | real-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (18)

- **real-letter-04**: expected `2011-04-27 Approval of Closure Plan to Kincaid Project Group, Inc.pdf`, got `2011-04-27 EPA Approval of Closure Plan for Kincaid Project Group. Inc.pdf` · review: LOW_CONFIDENCE
- **real-letter-06**: expected `2018-03-08 Response Letter to Pipeline and Hazardous Materials Safety Administration.pdf`, got `2018-03-08 Letter from Toledo Refining Company LLC.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-07**: expected `2025-04-28 Approval of SEP Completion Report to TPWR Developer, LLC.pdf`, got `2025-03-14 Letter to TPWR Developer, LLC.pdf` · trap: date 2025-03-14 (second amendment of the report) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real-letter-10**: expected `2022-06-22 Public Letter to Tarmac.pdf`, got `2022-06-22 Letter from CMA.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-letter-12**: expected `2024-05-21 Letter to Explain the Need for Redactions.pdf`, got `2024-05-21 Letter to Explain the Need for Redactions to Hon. Jessica G. L. Clarke.pdf` · trap: party Jessica G. L. Clarke (judge to whom it is addressed and who endorses it) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-07**: expected `2024-08-09 Notice of Appeal between Phoenix Metal Co., Ltd and United States.pdf`, got `2024-06-10 Notice of Appeal for Phoenix Metal Co., Ltd.pdf` · trap: date 2024-06-10 (date of the final judgment or order being appealed) · review: DATE_AMBIGUOUS
- **real-notice-13**: expected `2025-12-10 Final Enforcement Notice for Euro Car Parks Limited.pdf`, got `2025-10-20 Document from Competition and Markets Authority.pdf` · trap: date 2025-10-20 (date of the CMA's provisional notice) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real-notice-14**: expected `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN, Inc.txt`, got `2024-10-17 Notice of Apparent Liability for Forfeiture for ESPN Inc.txt` · review: LOW_CONFIDENCE
- **real-invoice-01**: expected `1898-12-30 Invoice from Woodworth & Co.jpg`, got `1898-12-31 Document - WOODWRTH&CO.jpg` · trap: date 1898-12-31 (PAID stamp (payment date)) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-02**: expected `1897-07-31 Invoice from Jacoby Brothers.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING
- **real-invoice-03**: expected `1882-07-21 Invoice from Deming, Palmer & Co.jpg`, got `List of Articles.jpg` · review: TYPE_INFERRED, DATE_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real-invoice-04**: expected `1859-01-13 Receipt from Leesburg Washingtonian.jpg`, got `Document - Dr.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real-invoice-05**: expected `1983-02-06 Receipt from Snyder Drug.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, DESCRIPTION_INVALID, PARSER_WARNING
- **real-invoice-07**: expected `1996-08-28 Invoice from Vernon Computer Rentals & Leasing.tif`, got `1996-08-28 Document from Vernon Computer Rentals & Leaging.tif` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
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
