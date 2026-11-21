# InternBench: live run

- **Run:** 2026-11-21T04:49:45Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 4.23 GB · sha256 `5a38b08c441a`
- **Corpus:** 60 documents · 60 completed · gold `308fa4b10b`
- **Wall time:** 882.48 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 17/60 (28.3%) |
| Document type | 49/60 (81.7%) |
| Date | 45/60 (75.0%) |
| Date role (when the reviewed date was chosen) | 24/45 (53.3%) |
| Parties | 29/60 (48.3%) |
| Relation word (when parties were named) | 42/58 (72.4%) |
| Party in the right role (when parties were named) | 12/39 (30.8%) |
| Ready / review routing | 5/36 (13.9%) |
| Description states nothing false | 60/60 (100.0%) |
| Description is specific | 41/60 (68.3%) |
| Description specificity | 62.2% (mean of 60) |
| Evidence recall (model's quotes) | 38.6% (mean of 60) |
| Digest recall (facts reaching the digest) | 91.5% (mean of 60) |
| Prompt recall (facts reaching the prompt sent) | 91.5% (mean of 60) |
| Sent to review | 91.7% of 60 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 73.3% | 60 |
| Review rate | 91.7% | 60 |
| Evidence recall | 38.6% | 60 |
| Total latency p50 | 12.16 s | 60 |
| Total latency p95 | 30.55 s | 60 |
| Generation latency p50 | 4.63 s | 60 |
| Generation latency p95 | 5.97 s | 60 |
| Generated tokens p50 | 87 | 60 |
| Generated tokens p95 | 109 | 60 |
| Prompt tokens p50 | 771 | 60 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 60 |
| Trap date chosen | 4 of 60 |
| Forbidden party named | 24 of 60 |
| Spurious parties named | 31 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 154 (0.0%) |
| Right name sent to review anyway | 8 of 36 |
| Sent to review for an unsupported fact | 44 of 60 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 2/11 | 7/11 | 4/11 | 1/8 | 90.9% | 10.97 s | 18.02 s |
| letter | 25 | 6/25 | 22/25 | 11/25 | 1/13 | 96.0% | 12.13 s | 31.21 s |
| notice | 20 | 9/20 | 15/20 | 13/20 | 3/14 | 85.0% | 14.52 s | 30.55 s |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0/1 | 100.0% | 7.94 s | 10.21 s |
| statement | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 6.66 s | 8.14 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 11 | 0/11 | 6/11 | 4/11 | 0/7 | 100.0% | 9.99 s | 18.02 s |
| native | 16 | 8/16 | 12/16 | 11/16 | 4/11 | 75.0% | 13.23 s | 21.78 s |
| office | 17 | 2/17 | 15/17 | 6/17 | 0/8 | 100.0% | 11.98 s | 24.22 s |
| scan | 14 | 6/14 | 11/14 | 7/14 | 1/8 | 92.9% | 15.46 s | 54.28 s |
| text | 2 | 1/2 | 1/2 | 1/2 | 0/2 | 100.0% | 13.41 s | 16.98 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 28 | 2/28 | 17/28 | 10/28 | 0/15 | 100.0% | 9.70 s | 14.23 s |
| 2-4 | 24 | 12/24 | 22/24 | 15/24 | 3/16 | 87.5% | 13.41 s | 23.86 s |
| 5-9 | 5 | 2/5 | 3/5 | 3/5 | 2/3 | 60.0% | 25.72 s | 31.93 s |
| 10-24 | 2 | 1/2 | 2/2 | 1/2 | 0/1 | 100.0% | 21.31 s | 54.28 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 24.22 s | 24.22 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 1/3 | 3/3 | 1/3 | 0/2 | 0 of 3 | 24.22 s | 54.28 s | 5.78 s | 7.07 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_person_at_company | 8 | 4/8 | 6/8 | 5/8 | 3 |
| addressed_to_person_at_firm | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_at_bottom | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_at_foot | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_in_frame | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| advisory_opinion | 2 | 1/2 | 1/2 | 1/2 | 1 |
| agency_billing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_enforcement | 7 | 4/7 | 6/7 | 5/7 | 2 |
| agency_notice | 2 | 1/2 | 1/2 | 2/2 | 1 |
| aka_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ambiguous_digit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| annex | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex_table | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attachments | 2 | 1/2 | 2/2 | 1/2 | 1 |
| australian_government | 1 | 0/1 | 0/1 | 0/1 | 2 |
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 0/1 | 1/1 | 0/1 | 1 |
| bill_to_prominent | 1 | 0/1 | 1/1 | 0/1 | 1 |
| billhead | 5 | 0/5 | 2/5 | 0/5 | 2 |
| body_recounts_earlier_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| born_digital | 2 | 1/2 | 2/2 | 1/2 | 1 |
| brand_vs_legal_name | 4 | 0/4 | 0/4 | 1/4 | 2 |
| business_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| caption_table | 6 | 4/6 | 6/6 | 5/6 | 0 |
| caption_table_flattened | 1 | 1/1 | 1/1 | 1/1 | 0 |
| case_citations | 1 | 0/1 | 0/1 | 0/1 | 2 |
| cc_in_body | 2 | 0/2 | 2/2 | 0/2 | 2 |
| cc_list | 5 | 3/5 | 4/5 | 4/5 | 2 |
| cell_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 3 | 3/3 | 3/3 | 3/3 | 0 |
| class_action | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_named_in_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| coloured_paper | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 22 | 7/22 | 15/22 | 12/22 | 11 |
| consignee_prominent | 1 | 0/1 | 0/1 | 1/1 | 0 |
| copy_list | 1 | 0/1 | 0/1 | 0/1 | 1 |
| corrected_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 6 | 4/6 | 6/6 | 5/6 | 0 |
| court_letter | 2 | 0/2 | 1/2 | 1/2 | 2 |
| customer_first | 3 | 0/3 | 2/3 | 0/3 | 2 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_field_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_typo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dated_line_at_end | 7 | 4/7 | 6/7 | 6/7 | 1 |
| dd_mm_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dd_month_yyyy | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_dates | 3 | 1/3 | 2/3 | 2/3 | 1 |
| delivery_contact | 1 | 0/1 | 1/1 | 0/1 | 1 |
| descriptive_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| doc | 4 | 1/4 | 4/4 | 1/4 | 3 |
| docx | 4 | 0/4 | 3/4 | 2/4 | 2 |
| due_date | 2 | 1/2 | 1/2 | 2/2 | 0 |
| ecf_header | 1 | 0/1 | 0/1 | 0/1 | 2 |
| ecf_header_text_only | 3 | 3/3 | 3/3 | 3/3 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| enclosure_after_notice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| endorsement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enforcement | 3 | 2/3 | 2/3 | 3/3 | 0 |
| established_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| fax_artifacts | 1 | 0/1 | 1/1 | 0/1 | 1 |
| fax_header | 1 | 0/1 | 1/1 | 1/1 | 0 |
| federal_register_stamp | 2 | 1/2 | 1/2 | 2/2 | 1 |
| filename_date_trap | 2 | 0/2 | 2/2 | 1/2 | 1 |
| first_name_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| flight_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnote_citation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 2 | 1/2 | 2/2 | 1/2 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| handwritten | 6 | 1/6 | 2/6 | 2/6 | 1 |
| handwritten_annotation | 6 | 0/6 | 4/6 | 1/6 | 5 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 0/1 | 1 |
| handwritten_salutation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 7 | 0/7 | 2/7 | 1/7 | 3 |
| historical | 1 | 0/1 | 1/1 | 0/1 | 0 |
| image | 2 | 0/2 | 2/2 | 1/2 | 0 |
| in_re_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 11 | 2/11 | 7/11 | 4/11 | 6 |
| issuer_not_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 4 | 0/4 | 1/4 | 1/4 | 2 |
| letter | 25 | 6/25 | 22/25 | 11/25 | 13 |
| letter_format | 2 | 0/2 | 1/2 | 0/2 | 2 |
| letter_style | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_officers | 1 | 0/1 | 1/1 | 0/1 | 0 |
| line_item_dates | 5 | 1/5 | 2/5 | 1/5 | 2 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 1 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_named_organisations | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo_header | 1 | 0/1 | 0/1 | 0/1 | 1 |
| metadata_date_mismatch | 1 | 0/1 | 0/1 | 1/1 | 0 |
| metadata_date_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 1 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 3/4 | 4/4 | 3/4 | 0 |
| multiple_parties | 1 | 0/1 | 0/1 | 0/1 | 1 |
| multiple_sheets | 1 | 0/1 | 1/1 | 1/1 | 0 |
| name_variant | 1 | 0/1 | 0/1 | 1/1 | 0 |
| native | 7 | 4/7 | 6/7 | 6/7 | 1 |
| native_pdf | 6 | 3/6 | 4/6 | 4/6 | 3 |
| no_dated_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_dated_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_explicit_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_letterhead_text | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_ocr_layer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_subject_line | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notary_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice | 20 | 9/20 | 15/20 | 13/20 | 8 |
| notice_of_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odt | 4 | 0/4 | 4/4 | 1/4 | 2 |
| office | 13 | 0/13 | 11/13 | 3/13 | 10 |
| old_referenced_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ordinal_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| paid_stamp | 2 | 0/2 | 1/2 | 0/2 | 2 |
| past_due_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| person_addressee | 2 | 0/2 | 2/2 | 1/2 | 0 |
| person_party | 2 | 0/2 | 2/2 | 0/2 | 2 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 1/1 | 0 |
| printer_credit | 1 | 0/1 | 0/1 | 1/1 | 0 |
| product_names | 1 | 0/1 | 1/1 | 0/1 | 1 |
| public_records_release | 3 | 1/3 | 3/3 | 1/3 | 2 |
| publication_note_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| re_line_other_matter | 2 | 0/2 | 1/2 | 1/2 | 2 |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0 |
| receipted | 1 | 0/1 | 0/1 | 0/1 | 1 |
| received_stamp | 2 | 1/2 | 1/2 | 2/2 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 1/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 7 | 4/7 | 6/7 | 5/7 | 2 |
| referenced_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 2/4 | 3/4 | 2/4 | 2 |
| released_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| remit_to | 2 | 1/2 | 1/2 | 1/2 | 1 |
| remit_to_bank | 1 | 0/1 | 1/1 | 0/1 | 1 |
| rtf | 1 | 0/1 | 0/1 | 0/1 | 2 |
| scanned | 21 | 6/21 | 15/21 | 10/21 | 9 |
| schedules_attached | 1 | 1/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 1/1 | 1/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 1/2 | 2/2 | 1/2 | 0 |
| signatory_vs_org | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_page_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| sister_company_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sole_proprietor | 1 | 0/1 | 0/1 | 0/1 | 0 |
| source_title_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 8 | 1/8 | 4/8 | 3/8 | 4 |
| stamp_date | 2 | 0/2 | 0/2 | 0/2 | 2 |
| statement | 2 | 0/2 | 0/2 | 0/2 | 1 |
| status_log | 1 | 0/1 | 1/1 | 0/1 | 1 |
| subject_line_names_statute | 1 | 0/1 | 1/1 | 0/1 | 1 |
| subject_line_other_party | 1 | 0/1 | 1/1 | 0/1 | 1 |
| table | 1 | 0/1 | 0/1 | 0/1 | 1 |
| table_layout | 2 | 0/2 | 0/2 | 2/2 | 1 |
| termination | 1 | 0/1 | 0/1 | 1/1 | 1 |
| text | 1 | 0/1 | 0/1 | 0/1 | 1 |
| tiff | 5 | 0/5 | 3/5 | 2/5 | 3 |
| two_addressees | 3 | 0/3 | 3/3 | 0/3 | 3 |
| two_digit_year | 3 | 1/3 | 2/3 | 1/3 | 1 |
| txt | 2 | 1/2 | 1/2 | 1/2 | 1 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| uk | 9 | 3/9 | 9/9 | 5/9 | 4 |
| uk_date_format | 2 | 1/2 | 2/2 | 1/2 | 1 |
| uk_government | 7 | 0/7 | 7/7 | 2/7 | 4 |
| uk_regulator | 3 | 0/3 | 2/3 | 1/3 | 2 |
| us_federal | 1 | 0/1 | 0/1 | 0/1 | 1 |
| us_state_government | 3 | 1/3 | 3/3 | 1/3 | 2 |
| utf8_bom | 1 | 1/1 | 1/1 | 1/1 | 0 |
| vat | 1 | 0/1 | 1/1 | 0/1 | 1 |
| wmo_header | 1 | 0/1 | 0/1 | 0/1 | 1 |
| xlsx | 1 | 0/1 | 1/1 | 1/1 | 0 |
| year_in_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| year_in_title | 1 | 0/1 | 0/1 | 1/1 | 0 |
| zero_amount | 1 | 0/1 | 1/1 | 0/1 | 1 |

## Routes

- **Pages:** fast 65 · layout 12 · ocr 70 · ocr_regions 1
- **Documents by route class:** fast 25 · layout 8 · ocr 26 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 60 | 12.16 s | 30.55 s | 54.28 s |
| `extraction_wall_ms` | 60 | 10.6 ms | 9.97 s | 39.14 s |
| `worker_total_ms` | 60 | 9.9 ms | 9.97 s | 39.13 s |
| `worker_snapshot_ms` | 60 | 0.20 ms | 0.83 ms | 2.9 ms |
| `worker_parse_ms` | 60 | 2.1 ms | 14.0 ms | 2.21 s |
| `worker_analysis_ms` | 60 | 0.20 ms | 1.3 ms | 12.8 ms |
| `worker_render_ms` | 60 | 0.00 ms | 879.7 ms | 6.30 s |
| `worker_image_decode_ms` | 60 | 0.00 ms | 64.0 ms | 86.8 ms |
| `worker_ocr_ms` | 60 | 0.00 ms | 18.78 s | 76.86 s |
| `worker_ocr_encode_ms` | 60 | 0.00 ms | 1.35 s | 6.07 s |
| `worker_ocr_engine_ms` | 60 | 0.00 ms | 17.20 s | 70.56 s |
| `worker_vision_ms` | 60 | 0.00 ms | 9.7 ms | 13.6 ms |
| `analyze_wall_ms` | 60 | 11.24 s | 21.18 s | 24.21 s |
| `distill_ms` | 60 | 1.5 ms | 8.9 ms | 38.7 ms |
| `prompt_ms` | 60 | 0.03 ms | 0.04 ms | 0.06 ms |
| `inference_ms` | 60 | 11.23 s | 21.11 s | 24.15 s |
| `prefill_ms` | 60 | 6.54 s | 14.04 s | 19.29 s |
| `generation_ms` | 60 | 4.63 s | 5.97 s | 7.07 s |
| `validation_ms` | 60 | 4.6 ms | 16.1 ms | 27.6 ms |
| `naming_ms` | 60 | 0.20 ms | 1.1 ms | 4.3 ms |
| `index_ms` | 60 | 1.4 ms | 7.8 ms | 36.0 ms |
| `retrieval_ms` | 60 | 0.03 ms | 0.96 ms | 3.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 60 | 112.4 tok/s | 117.0 tok/s | 117.8 tok/s |
| `generation_tok_per_s` | 60 | 18.8 tok/s | 19.5 tok/s | 19.6 tok/s |
| `prompt_tokens` | 60 | 771 | 1609 | 2242 |
| `cached_tokens` | 60 | 469 | 469 | 469 |
| `generated_tokens` | 60 | 87 | 109 | 127 |
| `estimated_prompt_tokens` | 60 | 1460 | 2530 | 3333 |
| `model_requests` | 60 | 1 | 1 | 1 |
| `redistillations` | 60 | 0 | 0 | 0 |
| `source_characters` | 60 | 2504 | 16805 | 76305 |
| `digest_characters` | 60 | 2837 | 6299 | 8597 |
| `prompt_characters` | 60 | 4886 | 8348 | 10646 |
| `worker_ocr_pages` | 60 | 0 | 6 | 18 |
| `worker_ocr_passes` | 60 | 0 | 6 | 18 |
| `worker_orientation_passes` | 60 | 0 | 6 | 18 |
| `worker_rendered_pixels` | 60 | 0 | 50490000 | 156552480 |
| `index_units` | 60 | 24 | 117 | 397 |
| `context_units` | 60 | 22 | 42 | 100 |
| `context_tokens` | 60 | 872 | 1942 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 25 | 2.0 ms | 10.6 ms | 1.7 ms | 9.9 ms | 8.5 ms | 0.56 ms | 0.00 ms | 0.00 ms |
| layout | 8 | 4.8 ms | 2.21 s | 4.3 ms | 2.21 s | 2.21 s | 12.8 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 334.3 ms | 334.3 ms | 333.2 ms | 333.2 ms | 7.1 ms | 1.2 ms | 46.2 ms | 269.9 ms |
| ocr | 26 | 2.11 s | 15.32 s | 2.11 s | 15.32 s | 14.0 ms | 0.83 ms | 987.7 ms | 28.12 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 28 | 1.31 s | 2.99 s | 1.31 s | 2.99 s | 14.0 ms | 0.36 ms | 157.8 ms | 2.80 s |
| 2-4 | 24 | 4.5 ms | 6.07 s | 4.1 ms | 6.07 s | 7.1 ms | 1.2 ms | 473.8 ms | 11.13 s |
| 5-9 | 5 | 9.97 s | 15.32 s | 9.97 s | 15.32 s | 2.21 s | 1.3 ms | 987.7 ms | 28.12 s |
| 10-24 | 2 | 128.4 ms | 39.14 s | 126.4 ms | 39.13 s | 111.7 ms | 12.8 ms | 6.30 s | 76.86 s |
| 25-49 | 1 | 14.1 ms | 14.1 ms | 13.0 ms | 13.0 ms | 12.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 1.52 s | 3.12 s | 1.52 s | 3.12 s | 15.5 ms | 3.4 ms | 129.0 ms | 3.06 s |
| letter | 25 | 6.8 ms | 8.21 s | 6.4 ms | 8.20 s | 11.7 ms | 1.3 ms | 565.0 ms | 15.06 s |
| notice | 20 | 4.5 ms | 14.04 s | 4.1 ms | 14.04 s | 12.6 ms | 0.96 ms | 968.3 ms | 27.58 s |
| receipt | 2 | 1.81 s | 2.11 s | 1.81 s | 2.11 s | 14.0 ms | 0.13 ms | 0.00 ms | 2.07 s |
| statement | 2 | 816.5 ms | 1.33 s | 816.2 ms | 1.33 s | 10.4 ms | 0.23 ms | 157.8 ms | 1.28 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 580 MB | 1388 MB | real2-letter-06 |
| Model server | 4428 MB | 4429 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 22 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (43)

- **real2-letter-02**: expected `2007-06-25 Letter to Proskauer Rose LLP.pdf`, got `2007-06-25 Letter to Gallion & Spielvogel.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-04**: expected `1974-12-20 Advisory Opinion to ARP Industries, Inc.pdf`, got `Letter to Michael A. Brown.pdf` · trap: party Michael A. Brown (General Counsel, signer) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-05**: expected `2014-06-30 Extension Letter to PricewaterhouseCoopers LLP.pdf`, got `2014-06-30 report for London Borough of Tower Hamlets.pdf` · trap: party London Borough of Tower Hamlets (subject of the inspection, named in the subject line) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real2-letter-06**: expected `2013-09-11 Advice Letter to HM Treasury.pdf`, got `2013-09-11 Letter from Clive Maxwell CBE.pdf` · trap: party Clive Maxwell (OFT Chief Executive who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-07**: expected `2018-02-09 Untitled Letter to Collegium Pharmaceutical, Inc.pdf`, got `Letter to Collegium Pharmaceutical, Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-10**: expected `2024-10-07 Letter between Nike, Inc and StockX LLC.pdf`, got `2024-10-04 Letter to Tamar Y. Duvdevani.pdf` · trap: date 2024-10-04 (date StockX filed the sur-reply); party Tamar Y. Duvdevani (signing lawyer) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-12**: expected `2025-08-20 Guidance Letter to Pool Reinsurance Company Limited.pdf`, got `2025-08-20 Letter from Gwyneth Nurse.pdf` · trap: party Gwyneth Nurse (HM Treasury Director General who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-13**: expected `2021-12-14 Remit Letter to Armed Forces’ Pay Review Body.odt`, got `2021-12-14 Letter from Ben Wallace.odt` · trap: party Ben Wallace (Secretary of State who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-14**: expected `1891-04-25 Response Letter to A. G. Bell.jpg`, got `1891-04-25 Letter to F Doria.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-15**: expected `2013-11-29 Response Letter to Baroness Lane-Fox of Soho.jpg`, got `2013-11-29 Letter from Baroness Lane-Fox of Soho CBE.jpg` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-01**: expected `2011-03-03 Notice of Probable Violation for Dakota Gasification Company.pdf`, got `2011-03-03 Notice of Probable Violation.pdf` · trap: party Gary Loop (VP & COO of Dakota Gasification Company, the individual addressee) · review: TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-02**: expected `2015-09-18 Notice of Violation for Volkswagen AG.pdf`, got `Notice of Violation for David Geanacopoulos.pdf` · trap: party David Geanacopoulos (VWGoA EVP and General Counsel, 'Thru' addressee) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-07**: expected `2003-10-20 Notice of Related Action between Synopsys, Inc and Ricoh Company, Ltd.pdf`, got `2003-10-20 Notice of Related Action Pending in the Northern District of California with Synopsys, Inc.pdf` · review: DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-08**: expected `2012-12-20 Notice of Motion between Marylebone PCC Limited—Rose 2 Fund and Millennium Global Investments Ltd.pdf`, got `2012-12-20 Notice of Motion between Marylebone PCC Limited -- Rose 2 Fund and BCP Securities LLC.pdf` · review: LOW_CONFIDENCE
- **real2-notice-09**: expected `2026-04-01 Notice of Intent to Terminate Receivership for Citizens National Bank.pdf`, got `2026-04-03 Notice to All Interested Parties of Intent to Terminate Receivership.pdf` · trap: date 2026-04-03 (scheduled Federal Register publication date in the OFR banner and footer) · **filed without review**
- **real2-notice-13**: expected `2017-02-03 Improvement Notice for Statoil (U.K.) Limited.xlsx`, got `2017-02-03 Improvement Notice from Statoil (U.K.) Limited.xlsx` · review: PARTY_UNSUPPORTED
- **real2-invoice-01**: expected `1995-12-04 Ticket Subscription Invoice from USTA.tif`, got `Ticket Subscription Invoice from U.S. Open Ticket Department.tif` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-02**: expected `1995-07-20 Invoice from The New York Academy of Sciences.tif`, got `1995-07-20 Bill to Shp from Council Tobacco Res.-USA.tif` · trap: party Council Tobacco Res-USA (customer (bill-to/ship-to), not the issuer) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-03**: expected `1996-02-15 Statement from The Hearst Corporation.tif`, got `Service Center Statement for Philip Morris USA.tif` · trap: party Philip Morris USA (bill-to customer, not the issuer) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real2-invoice-04**: expected `1989-03-06 Invoice from International Union of Biochemistry.tif`, got `1989-03-06 Invoice from International Union.tif` · review: LOW_CONFIDENCE
- **real2-invoice-05**: expected `1996-03-04 Invoice from Marketing Visions, Inc.tif`, got `1996-03-04 Invoice from Rob Gotti.tif` · trap: party Rob Gotti (Philip Morris employee who ordered the job) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-06**: expected `1882-10-01 Bill from Charles Williams, Jr.jpg`, got `Bill from National Bell Telephone Co.jpg` · trap: party National Bell Telephone Co. (company the issuer is authorized manufacturer for, printed large on the billhead) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-07**: expected `1897-01-02 Bill from Pollok & Mauro.jpg`, got `Document from Alexander Graham Bell, Esq.jpg` · trap: party Alexander Graham Bell (client billed, printed first) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-08**: expected `1864-12-31 Receipt from The United States Telegraph Company.jpg`, got `1864-12-31 Document.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-09**: expected `1860-10-02 Receipt from The Adams Express Company.jpg`, got `Document from Adams Express Company.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-10**: expected `1923-11-22 Invoice from Times-Mirror Printing & Binding House.pdf`, got `1923-11-22 Document from Chas F Lummis.pdf` · trap: party Chas F Lummis (customer, typed above the letterhead) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-11**: expected `1920-01-13 Statement from Grace Bros. Brewing Co.pdf`, got `Statement from Goeme Ellen.pdf` · review: TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-12**: expected `1895-12-09 Bill from Geo. P. McNear.pdf`, got `Document.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-14**: expected `2010-08-25 Invoice from Royal Images.pdf`, got `2010-08-25 Invoice from DCLG FSSD CP2P Team.pdf` · trap: party DCLG FSSD CP2P Team (customer (bill-to block)) · review: LOW_CONFIDENCE
- **real2-office-01**: expected `2023-06-22 Request Letter to Ofsted.docx`, got `2023-06-22 Letter to Minister for Women and Equalities.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-02**: expected `2019-11-04 Remit and Recommendations Letter to Bank of England.docx`, got `2019-11-04 Letter from Mark Carney.docx` · trap: party Mark Carney (Governor addressed; the organisation is the Bank of England) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real2-office-03**: expected `2024-07-16 Notice of Variation and Consolidation for Britvic Soft Drinks Limited.docx`, got `2024-07-16 Notice of variation and consolidation for Sandra Cavill.docx` · trap: party Sandra Cavill (officer who authorises the notice) · review: PARTY_UNSUPPORTED
- **real2-office-04**: expected `2014-11-17 Announcement of Consultation from Environment Agency.docx`, got `Announcement of Consultation on Draft Plan and Its Accompanying Environmental from Environment Agency.docx` · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-05**: expected `2019-10-18 Advice Letter to Home Office.odt`, got `2019-10-18 Letter to Owen Bowden-Jones.odt` · trap: party Owen Bowden-Jones (ACMD chair who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-06**: expected `2019-10-24 Response Letter to ONS.odt`, got `2019-10-24 Letter to Migration Advisory Committee.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING
- **real2-office-07**: expected `2024-03-28 Response Letter to Welsh Government.odt`, got `2024-03-28 Letter to Secretary of State for Work & Pensions.odt` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-08**: expected `2026-01-29 Notice of Proposal to Modify Definitive Map Order for North Yorkshire Council.odt`, got `2026-01-29 Notice of Proposal to Modify Definitive Map Order for Zoe McPhail.odt` · trap: party Zoe McPhail (Planning Inspectorate case officer) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-09**: expected `2012-11-29 Cover Letter to National Fire Protection Association, Inc.doc`, got `2012-11-29 2013 California Electrical Code- Executed Agreement between Debra Rose and Cynthia Biedermann.doc` · trap: party Debra Rose (NFPA staff member addressed); party Cynthia Biedermann (commission staff member who signs) · review: LOW_CONFIDENCE
- **real2-office-10**: expected `2011-07-19 Response Letter to John Mann MP.doc`, got `2011-07-19 Letter to SIR GUS O’DONNELL KCB.doc` · trap: party Gus O'Donnell (Cabinet Secretary who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED
- **real2-office-11**: expected `2013-04-08 Advice Letter to Lord Strathclyde CH PC.doc`, got `2013-04-08 Document - Lord Lang of Monkton.doc` · trap: party Lord Lang of Monkton (committee chairman who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- … and 3 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
