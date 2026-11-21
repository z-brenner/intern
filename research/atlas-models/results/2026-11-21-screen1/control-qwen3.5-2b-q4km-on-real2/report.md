# InternBench: live run

- **Run:** 2026-11-21T04:59:28Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 55 documents · 55 completed · gold `c3f4c25953`
- **Wall time:** 796.37 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 23/55 (41.8%) |
| Document type | 47/55 (85.5%) |
| Date | 47/55 (85.5%) |
| Date role (when the reviewed date was chosen) | 19/47 (40.4%) |
| Parties | 38/55 (69.1%) |
| Relation word (when parties were named) | 40/53 (75.5%) |
| Party in the right role (when parties were named) | 19/36 (52.8%) |
| Ready / review routing | 4/33 (12.1%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 41/55 (74.6%) |
| Description specificity | 63.0% (mean of 55) |
| Evidence recall (model's quotes) | 53.6% (mean of 55) |
| Digest recall (facts reaching the digest) | 93.5% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 93.5% (mean of 55) |
| Sent to review | 92.7% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 61.8% | 55 |
| Review rate | 92.7% | 55 |
| Evidence recall | 53.6% | 55 |
| Total latency p50 | 11.96 s | 55 |
| Total latency p95 | 28.41 s | 55 |
| Generation latency p50 | 5.99 s | 55 |
| Generation latency p95 | 7.90 s | 55 |
| Generated tokens p50 | 91 | 55 |
| Generated tokens p95 | 119 | 55 |
| Prompt tokens p50 | 855 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 55 |
| Trap date chosen | 1 of 55 |
| Forbidden party named | 10 of 55 |
| Spurious parties named | 15 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 117 (0.0%) |
| Right name sent to review anyway | 14 of 33 |
| Sent to review for an unsupported fact | 34 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 3/7 | 5/7 | 5/7 | 1/5 | 85.7% | 9.72 s | 14.48 s |
| letter | 25 | 9/25 | 23/25 | 14/25 | 0/13 | 100.0% | 12.25 s | 28.18 s |
| notice | 20 | 11/20 | 18/20 | 18/20 | 3/14 | 85.0% | 13.52 s | 28.41 s |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0/1 | 100.0% | 7.59 s | 11.45 s |
| statement | 1 | 0/1 | 0/1 | 0/1 | – | 100.0% | 6.18 s | 6.18 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 3/8 | 5/8 | 6/8 | 0/5 | 100.0% | 9.72 s | 11.65 s |
| native | 15 | 8/15 | 15/15 | 13/15 | 2/11 | 86.7% | 12.79 s | 21.81 s |
| office | 17 | 6/17 | 15/17 | 11/17 | 0/8 | 100.0% | 11.25 s | 24.78 s |
| scan | 13 | 5/13 | 10/13 | 7/13 | 2/7 | 84.6% | 18.71 s | 52.10 s |
| text | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 10.24 s | 16.13 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 6/23 | 17/23 | 13/23 | 0/12 | 100.0% | 10.11 s | 13.04 s |
| 2-4 | 24 | 11/24 | 23/24 | 18/24 | 2/16 | 91.7% | 13.29 s | 22.26 s |
| 5-9 | 5 | 4/5 | 4/5 | 5/5 | 2/3 | 60.0% | 24.42 s | 31.31 s |
| 10-24 | 2 | 1/2 | 2/2 | 1/2 | 0/1 | 100.0% | 18.17 s | 52.10 s |
| 25-49 | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 24.78 s | 24.78 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 2/3 | 3/3 | 2/3 | 0/2 | 0 of 3 | 24.78 s | 52.10 s | 7.04 s | 8.16 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_person_at_company | 8 | 4/8 | 7/8 | 5/8 | 3 |
| addressed_to_person_at_firm | 1 | 0/1 | 1/1 | 0/1 | 0 |
| addressee_at_bottom | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_at_foot | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_in_frame | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| advisory_opinion | 2 | 1/2 | 1/2 | 1/2 | 1 |
| agency_billing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_enforcement | 7 | 4/7 | 6/7 | 7/7 | 0 |
| agency_notice | 2 | 2/2 | 2/2 | 2/2 | 0 |
| aka_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ambiguous_digit | 1 | 0/1 | 0/1 | 1/1 | 0 |
| annex | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex_table | 1 | 0/1 | 1/1 | 1/1 | 0 |
| attachments | 2 | 0/2 | 2/2 | 0/2 | 1 |
| australian_government | 1 | 0/1 | 0/1 | 1/1 | 1 |
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 0/1 | 1/1 | 0/1 | 0 |
| billhead | 3 | 0/3 | 1/3 | 1/3 | 0 |
| body_recounts_earlier_dates | 2 | 1/2 | 2/2 | 2/2 | 0 |
| born_digital | 2 | 1/2 | 2/2 | 1/2 | 0 |
| brand_vs_legal_name | 2 | 0/2 | 0/2 | 1/2 | 0 |
| business_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_table | 6 | 3/6 | 6/6 | 6/6 | 0 |
| caption_table_flattened | 1 | 1/1 | 1/1 | 1/1 | 0 |
| case_citations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_in_body | 2 | 0/2 | 2/2 | 0/2 | 2 |
| cc_list | 5 | 3/5 | 5/5 | 4/5 | 0 |
| cell_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 3 | 1/3 | 3/3 | 2/3 | 1 |
| class_action | 1 | 0/1 | 1/1 | 1/1 | 0 |
| client_named_in_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| competing_dates | 20 | 13/20 | 18/20 | 15/20 | 3 |
| consignee_prominent | 1 | 0/1 | 0/1 | 1/1 | 0 |
| copy_list | 1 | 0/1 | 0/1 | 1/1 | 0 |
| court_filing | 6 | 2/6 | 6/6 | 5/6 | 1 |
| court_letter | 2 | 0/2 | 2/2 | 1/2 | 0 |
| customer_first | 2 | 0/2 | 1/2 | 0/2 | 1 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_field_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_typo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dated_line_at_end | 7 | 4/7 | 7/7 | 6/7 | 1 |
| dd_mm_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deadline_dates | 3 | 2/3 | 3/3 | 2/3 | 0 |
| delivery_contact | 1 | 0/1 | 1/1 | 0/1 | 0 |
| descriptive_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| doc | 4 | 2/4 | 3/4 | 2/4 | 2 |
| docx | 4 | 3/4 | 4/4 | 4/4 | 0 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ecf_header | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ecf_header_text_only | 3 | 1/3 | 3/3 | 2/3 | 1 |
| email | 2 | 0/2 | 2/2 | 0/2 | 1 |
| enclosure_after_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| endorsement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enforcement | 3 | 3/3 | 3/3 | 3/3 | 0 |
| fax_artifacts | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fax_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| federal_register_stamp | 2 | 2/2 | 2/2 | 2/2 | 0 |
| filename_date_trap | 2 | 1/2 | 2/2 | 1/2 | 1 |
| first_name_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| flight_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnote_citation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnotes | 2 | 1/2 | 2/2 | 1/2 | 1 |
| form_layout | 1 | 0/1 | 1/1 | 1/1 | 0 |
| handwritten | 5 | 0/5 | 2/5 | 3/5 | 0 |
| handwritten_annotation | 3 | 1/3 | 2/3 | 2/3 | 1 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 0/1 | 1 |
| handwritten_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 5 | 0/5 | 1/5 | 2/5 | 1 |
| historical | 1 | 1/1 | 1/1 | 1/1 | 0 |
| image | 2 | 2/2 | 2/2 | 2/2 | 0 |
| in_re_caption | 1 | 0/1 | 1/1 | 0/1 | 1 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 7 | 3/7 | 5/7 | 5/7 | 1 |
| issuer_not_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 4 | 0/4 | 1/4 | 2/4 | 1 |
| letter | 25 | 9/25 | 23/25 | 14/25 | 8 |
| letter_format | 2 | 1/2 | 1/2 | 2/2 | 0 |
| letter_style | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 1/2 | 2/2 | 2/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_officers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 4 | 1/4 | 2/4 | 2/4 | 0 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_named_organisations | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| metadata_date_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| msg | 2 | 0/2 | 2/2 | 0/2 | 1 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 2/4 | 4/4 | 4/4 | 0 |
| multiple_parties | 1 | 0/1 | 0/1 | 1/1 | 0 |
| multiple_sheets | 1 | 0/1 | 1/1 | 1/1 | 0 |
| name_variant | 1 | 1/1 | 1/1 | 1/1 | 0 |
| native | 7 | 4/7 | 7/7 | 7/7 | 0 |
| native_pdf | 6 | 3/6 | 6/6 | 5/6 | 1 |
| no_dated_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_dated_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_explicit_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_letterhead_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_subject_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notary_stamp | 1 | 0/1 | 1/1 | 1/1 | 0 |
| notice | 20 | 11/20 | 18/20 | 18/20 | 2 |
| notice_of_meeting | 1 | 1/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| odt | 4 | 1/4 | 4/4 | 3/4 | 0 |
| office | 13 | 5/13 | 11/13 | 9/13 | 4 |
| old_referenced_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ordinal_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| past_due_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| person_addressee | 2 | 2/2 | 2/2 | 2/2 | 0 |
| person_party | 2 | 1/2 | 2/2 | 1/2 | 1 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 1/1 | 0 |
| printer_credit | 1 | 0/1 | 0/1 | 1/1 | 0 |
| product_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_records_release | 3 | 0/3 | 2/3 | 0/3 | 2 |
| publication_note_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| re_line_other_matter | 2 | 0/2 | 2/2 | 1/2 | 0 |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0 |
| receipted | 1 | 0/1 | 0/1 | 0/1 | 1 |
| received_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 1/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 7 | 5/7 | 7/7 | 5/7 | 2 |
| referenced_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 2/4 | 3/4 | 2/4 | 2 |
| released_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| remit_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_to_bank | 1 | 0/1 | 1/1 | 0/1 | 0 |
| rtf | 1 | 0/1 | 0/1 | 1/1 | 1 |
| scanned | 16 | 7/16 | 13/16 | 10/16 | 4 |
| schedules_attached | 1 | 1/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 1/1 | 1/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 1/2 | 2/2 | 2/2 | 0 |
| signatory_vs_org | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_page_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sister_company_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| source_title_mismatch | 1 | 1/1 | 1/1 | 1/1 | 0 |
| staff_names | 5 | 2/5 | 3/5 | 4/5 | 0 |
| stamp_date | 2 | 0/2 | 0/2 | 1/2 | 1 |
| statement | 1 | 0/1 | 0/1 | 0/1 | 0 |
| status_log | 1 | 1/1 | 1/1 | 1/1 | 0 |
| subject_line_names_statute | 1 | 0/1 | 1/1 | 0/1 | 1 |
| subject_line_other_party | 1 | 0/1 | 1/1 | 0/1 | 1 |
| table | 1 | 0/1 | 0/1 | 1/1 | 0 |
| table_layout | 2 | 2/2 | 2/2 | 2/2 | 0 |
| termination | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 1 | 0/1 | 1/1 | 0/1 | 0 |
| tiff | 2 | 1/2 | 2/2 | 2/2 | 0 |
| two_addressees | 3 | 0/3 | 2/3 | 1/3 | 2 |
| two_digit_year | 2 | 1/2 | 1/2 | 1/2 | 0 |
| txt | 2 | 1/2 | 2/2 | 1/2 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| uk | 9 | 2/9 | 9/9 | 5/9 | 4 |
| uk_date_format | 2 | 1/2 | 2/2 | 1/2 | 0 |
| uk_government | 7 | 2/7 | 7/7 | 5/7 | 1 |
| uk_regulator | 3 | 3/3 | 3/3 | 3/3 | 0 |
| us_federal | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_state_government | 3 | 0/3 | 2/3 | 0/3 | 2 |
| utf8_bom | 1 | 1/1 | 1/1 | 1/1 | 0 |
| vat | 1 | 0/1 | 1/1 | 0/1 | 0 |
| wmo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| xlsx | 1 | 0/1 | 1/1 | 1/1 | 0 |
| year_in_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 65 · layout 12 · ocr 65 · ocr_regions 1
- **Documents by route class:** fast 25 · layout 8 · ocr 21 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 11.96 s | 28.41 s | 52.10 s |
| `extraction_wall_ms` | 55 | 10.1 ms | 14.18 s | 38.60 s |
| `worker_total_ms` | 55 | 9.5 ms | 14.18 s | 38.60 s |
| `worker_snapshot_ms` | 55 | 0.21 ms | 0.59 ms | 2.2 ms |
| `worker_parse_ms` | 55 | 1.8 ms | 14.3 ms | 2.06 s |
| `worker_analysis_ms` | 55 | 0.19 ms | 1.6 ms | 12.8 ms |
| `worker_render_ms` | 55 | 0.00 ms | 981.3 ms | 6.19 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 64.3 ms | 86.8 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 27.69 s | 76.06 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 1.93 s | 5.92 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 25.04 s | 69.89 s |
| `worker_vision_ms` | 55 | 0.00 ms | 9.8 ms | 13.3 ms |
| `analyze_wall_ms` | 55 | 11.21 s | 20.11 s | 24.77 s |
| `distill_ms` | 55 | 1.6 ms | 20.6 ms | 38.7 ms |
| `prompt_ms` | 55 | 0.03 ms | 0.06 ms | 0.06 ms |
| `inference_ms` | 55 | 11.21 s | 20.09 s | 24.71 s |
| `prefill_ms` | 55 | 5.40 s | 13.17 s | 16.54 s |
| `generation_ms` | 55 | 5.99 s | 7.90 s | 8.16 s |
| `validation_ms` | 55 | 5.1 ms | 25.2 ms | 30.0 ms |
| `naming_ms` | 55 | 0.21 ms | 2.1 ms | 4.4 ms |
| `index_ms` | 55 | 1.5 ms | 19.0 ms | 34.8 ms |
| `retrieval_ms` | 55 | 0.03 ms | 1.5 ms | 3.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 159.3 tok/s | 161.5 tok/s | 161.7 tok/s |
| `generation_tok_per_s` | 55 | 15.2 tok/s | 15.7 tok/s | 15.9 tok/s |
| `prompt_tokens` | 55 | 855 | 2053 | 2593 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 91 | 119 | 121 |
| `estimated_prompt_tokens` | 55 | 1513 | 2699 | 3333 |
| `model_requests` | 55 | 1 | 1 | 1 |
| `redistillations` | 55 | 0 | 0 | 0 |
| `source_characters` | 55 | 2795 | 43392 | 76305 |
| `digest_characters` | 55 | 2934 | 6796 | 8597 |
| `prompt_characters` | 55 | 4983 | 8845 | 10646 |
| `worker_ocr_pages` | 55 | 0 | 7 | 18 |
| `worker_ocr_passes` | 55 | 0 | 7 | 18 |
| `worker_orientation_passes` | 55 | 0 | 7 | 18 |
| `worker_rendered_pixels` | 55 | 0 | 58905000 | 156552480 |
| `index_units` | 55 | 24 | 255 | 397 |
| `context_units` | 55 | 22 | 51 | 100 |
| `context_tokens` | 55 | 925 | 2111 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 25 | 2.1 ms | 10.5 ms | 1.8 ms | 10.1 ms | 8.7 ms | 0.56 ms | 0.00 ms | 0.00 ms |
| layout | 8 | 4.7 ms | 2.06 s | 4.1 ms | 2.06 s | 2.06 s | 12.8 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 342.9 ms | 342.9 ms | 342.5 ms | 342.5 ms | 7.3 ms | 1.2 ms | 47.0 ms | 278.3 ms |
| ocr | 21 | 2.12 s | 15.05 s | 2.12 s | 15.05 s | 12.1 ms | 0.86 ms | 1.02 s | 27.82 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 769.9 ms | 2.12 s | 769.5 ms | 2.12 s | 12.1 ms | 0.27 ms | 134.2 ms | 2.06 s |
| 2-4 | 24 | 4.4 ms | 6.18 s | 4.1 ms | 6.18 s | 7.3 ms | 1.2 ms | 432.9 ms | 11.37 s |
| 5-9 | 5 | 9.64 s | 15.05 s | 9.64 s | 15.05 s | 2.06 s | 1.3 ms | 1.02 s | 27.82 s |
| 10-24 | 2 | 127.8 ms | 38.60 s | 126.1 ms | 38.60 s | 111.6 ms | 12.8 ms | 6.19 s | 76.06 s |
| 25-49 | 1 | 13.9 ms | 13.9 ms | 12.8 ms | 12.8 ms | 12.5 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 1.40 s | 1.82 s | 1.40 s | 1.82 s | 10.5 ms | 3.4 ms | 96.4 ms | 1.72 s |
| letter | 25 | 7.1 ms | 8.07 s | 6.5 ms | 8.07 s | 10.8 ms | 1.3 ms | 561.4 ms | 14.83 s |
| notice | 20 | 4.4 ms | 14.18 s | 4.1 ms | 14.18 s | 12.5 ms | 1.0 ms | 981.3 ms | 27.69 s |
| receipt | 2 | 1.78 s | 2.09 s | 1.78 s | 2.09 s | 14.3 ms | 0.11 ms | 0.00 ms | 2.06 s |
| statement | 1 | 815.0 ms | 815.0 ms | 814.6 ms | 814.6 ms | 1.2 ms | 0.12 ms | 134.2 ms | 666.1 ms |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 541 MB | 1388 MB | real2-letter-06 |
| Model server | 1573 MB | 1578 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 18 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (32)

- **real2-letter-02**: expected `2007-06-25 Letter to Proskauer Rose LLP.pdf`, got `2007-06-25 Letter to Olde Monmouth Stock Transfer Co., Inc.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-04**: expected `1974-12-20 Advisory Opinion to ARP Industries, Inc.pdf`, got `1974-12-28 Letter from Michael A. Brown.pdf` · trap: party Michael A. Brown (General Counsel, signer) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-05**: expected `2014-06-30 Extension Letter to PricewaterhouseCoopers LLP.pdf`, got `2014-06-30 Letter to London Borough of Tower Hamlets.pdf` · trap: party London Borough of Tower Hamlets (subject of the inspection, named in the subject line) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-06**: expected `2013-09-11 Advice Letter to HM Treasury.pdf`, got `2013-09-11 Letter from Clive Maxwell CBE.pdf` · trap: party Clive Maxwell (OFT Chief Executive who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-10**: expected `2024-10-07 Letter between Nike, Inc and StockX LLC.pdf`, got `2024-10-07 Letter to Nike, Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-11**: expected `2022-12-14 Letter of Comfort to Low Carbon Contracts Company.pdf`, got `2022-12-14 Notice from Low Carbon Contracts Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-12**: expected `2025-08-20 Guidance Letter to Pool Reinsurance Company Limited.pdf`, got `2025-08-20 Letter from Gwyneth Nurse.pdf` · trap: party Gwyneth Nurse (HM Treasury Director General who signs) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-13**: expected `2021-12-14 Remit Letter to Armed Forces’ Pay Review Body.odt`, got `2021-12-14 Letter to Ben Wallace.odt` · trap: party Ben Wallace (Secretary of State who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-02**: expected `2015-09-18 Notice of Violation for Volkswagen AG.pdf`, got `Notice of Violation to Volkswagen Group of America. Inc.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-03**: expected `2008-09-16 Notice of Bankruptcy for Lehman Brothers Holdings Inc.pdf`, got `2008-09-16 Notice of Bankruptcy for United States Bankruptcy Court for the Southern District of New York.pdf` · trap: party United States Bankruptcy Court for the Southern District of New York (court where the bankruptcy case is pending) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-05**: expected `2008-01-22 Notice of Entry of Order between American Service Insurance Company and NAFTA General Agency.pdf`, got `2008-01-22 Notice of Entry of Order between American Service Insurance and NAFTA General Agency.pdf` · review: LOW_CONFIDENCE
- **real2-notice-07**: expected `2003-10-20 Notice of Related Action between Synopsys, Inc and Ricoh Company, Ltd.pdf`, got `2003-10-20 Notice of Related Action Pending between SYNOPSYS, INC and Ricoh Company, Ltd.pdf` · review: LOW_CONFIDENCE
- **real2-notice-08**: expected `2012-12-20 Notice of Motion between Marylebone PCC Limited—Rose 2 Fund and Millennium Global Investments Ltd.pdf`, got `2012-12-20 Notice of Motion between Marylebone PCC Limited and BCP Securities LLC.pdf` · **filed without review**
- **real2-notice-11**: expected `2020-01-29 Notice of Monetary Penalty for AQA Education.pdf`, got `2020-01-29 Notice of Monetary Penalty to AQA.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-13**: expected `2017-02-03 Improvement Notice for Statoil (U.K.) Limited.xlsx`, got `2017-02-03 Improvement Notice from Statoil (U.K.) Limited.xlsx` · review: PARTY_UNSUPPORTED
- **real2-invoice-04**: expected `1989-03-06 Invoice from International Union of Biochemistry.tif`, got `1989-03-06 Invoice from International Union.tif` · review: LOW_CONFIDENCE
- **real2-invoice-06**: expected `1882-10-01 Bill from Charles Williams, Jr.jpg`, got `Bill from Charles Williams Jr.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-07**: expected `1897-01-02 Bill from Pollok & Mauro.jpg`, got `Document from Alexander Graham Bell.jpg` · trap: party Alexander Graham Bell (client billed, printed first) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-08**: expected `1864-12-31 Receipt from The United States Telegraph Company.jpg`, got `1864-12-31 Document from cbachinglon.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-09**: expected `1860-10-02 Receipt from The Adams Express Company.jpg`, got `Document - Adams Express Company.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-11**: expected `1920-01-13 Statement from Grace Bros. Brewing Co.pdf`, got `Statement.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARTY_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-14**: expected `2010-08-25 Invoice from Royal Images.pdf`, got `2010-08-25 Invoice.pdf` · review: PARTY_MISSING
- **real2-office-01**: expected `2023-06-22 Request Letter to Ofsted.docx`, got `2023-06-22 Letter to Minister for Women and Equalities.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-05**: expected `2019-10-18 Advice Letter to Home Office.odt`, got `2019-10-18 Letter to Advisory Council on the Misuse of Drugs.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-06**: expected `2019-10-24 Response Letter to ONS.odt`, got `2019-10-24 Response from Migration Advisory Committee.odt` · review: TYPE_MISSING, PARTY_UNSUPPORTED
- **real2-office-07**: expected `2024-03-28 Response Letter to Welsh Government.odt`, got `2024-03-28 Letter to Secretary of State for Work & Pensions.odt` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-09**: expected `2012-11-29 Cover Letter to National Fire Protection Association, Inc.doc`, got `California Electrical Code- Executed Agreement with Debra Rose.doc` · trap: party Debra Rose (NFPA staff member addressed) · review: DATE_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE
- **real2-office-11**: expected `2013-04-08 Advice Letter to Lord Strathclyde CH PC.doc`, got `2013-04-08 Letter to Lord Lang of Monkton.doc` · trap: party Lord Lang of Monkton (committee chairman who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-12**: expected `2001-07-25 Notice of the Primary Universal Service Provider for Telstra Corporation Limited.rtf`, got `2001-06-01 Notice for Telstra Corporation Limited.rtf` · trap: date 2001-06-01 (date of the Commonwealth-Telstra agreement) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-13**: expected `2016-04-05 Email to NFPA.msg`, got `2016-04-05 Agreement between NFPA and Department of General Services.msg` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-14**: expected `2016-06-21 Email to ICC.msg`, got `2016-06-21 MOU - Mark Johnson.msg` · trap: party Mark Johnson (ICC staff member addressed) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-15**: expected `2024-06-21 Public Information Statement from National Weather Service.txt`, got `2024-06-21 Public Information Statement from NOAA.txt` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
