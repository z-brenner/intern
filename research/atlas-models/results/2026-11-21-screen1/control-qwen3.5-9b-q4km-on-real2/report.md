# InternBench: live run

- **Run:** 2026-11-21T04:51:49Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 58 documents · 58 completed · gold `cffaf72d3b`
- **Wall time:** 2985.05 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 34/58 (58.6%) |
| Document type | 51/58 (87.9%) |
| Date | 53/58 (91.4%) |
| Date role (when the reviewed date was chosen) | 24/53 (45.3%) |
| Parties | 44/58 (75.9%) |
| Relation word (when parties were named) | 50/56 (89.3%) |
| Party in the right role (when parties were named) | 24/39 (61.5%) |
| Ready / review routing | 5/36 (13.9%) |
| Description states nothing false | 58/58 (100.0%) |
| Description is specific | 45/58 (77.6%) |
| Description specificity | 70.7% (mean of 58) |
| Evidence recall (model's quotes) | 69.2% (mean of 58) |
| Digest recall (facts reaching the digest) | 93.0% (mean of 58) |
| Prompt recall (facts reaching the prompt sent) | 93.0% (mean of 58) |
| Sent to review | 89.7% of 58 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 72.4% | 58 |
| Review rate | 89.7% | 58 |
| Evidence recall | 69.2% | 58 |
| Total latency p50 | 47.03 s | 58 |
| Total latency p95 | 90.75 s | 58 |
| Generation latency p50 | 22.70 s | 58 |
| Generation latency p95 | 28.66 s | 58 |
| Generated tokens p50 | 92 | 58 |
| Generated tokens p95 | 117 | 58 |
| Prompt tokens p50 | 835 | 58 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 58 |
| Trap date chosen | 0 of 58 |
| Forbidden party named | 10 of 58 |
| Spurious parties named | 11 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 150 (0.0%) |
| Right name sent to review anyway | 19 of 36 |
| Sent to review for an unsupported fact | 42 of 58 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 6/11 | 8/11 | 7/11 | 1/8 | 81.8% | 39.87 s | 62.65 s |
| letter | 24 | 14/24 | 23/24 | 17/24 | 0/13 | 100.0% | 47.03 s | 90.75 s |
| notice | 20 | 14/20 | 20/20 | 18/20 | 4/14 | 80.0% | 53.40 s | 85.67 s |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0/1 | 100.0% | 23.65 s | 32.44 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 32.33 s | 32.33 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 11 | 5/11 | 8/11 | 8/11 | 0/7 | 100.0% | 38.60 s | 62.65 s |
| native | 15 | 10/15 | 14/15 | 13/15 | 3/11 | 80.0% | 49.25 s | 85.67 s |
| office | 17 | 9/17 | 17/17 | 11/17 | 1/8 | 94.1% | 47.56 s | 94.10 s |
| scan | 13 | 9/13 | 12/13 | 11/13 | 0/8 | 92.3% | 62.13 s | 99.95 s |
| text | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 50.0% | 48.88 s | 62.34 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 27 | 15/27 | 23/27 | 19/27 | 1/15 | 92.6% | 38.60 s | 49.00 s |
| 2-4 | 24 | 13/24 | 23/24 | 18/24 | 4/16 | 83.3% | 50.92 s | 85.67 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 0/3 | 100.0% | 70.65 s | 82.99 s |
| 10-24 | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 100.0% | 73.64 s | 99.95 s |
| 25-49 | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 94.10 s | 94.10 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 2/3 | 3/3 | 3/3 | 0/2 | 0 of 3 | 94.10 s | 99.95 s | 26.37 s | 29.70 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_person_at_company | 7 | 4/7 | 6/7 | 6/7 | 1 |
| addressed_to_person_at_firm | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_at_bottom | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_at_foot | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_in_frame | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 0/1 | 1 |
| advisory_opinion | 2 | 1/2 | 1/2 | 2/2 | 0 |
| agency_billing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_enforcement | 7 | 7/7 | 7/7 | 7/7 | 0 |
| agency_notice | 2 | 1/2 | 2/2 | 2/2 | 0 |
| aka_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ambiguous_digit | 1 | 0/1 | 0/1 | 1/1 | 0 |
| annex | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachments | 2 | 1/2 | 2/2 | 1/2 | 1 |
| australian_government | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to_prominent | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 4 | 0/4 | 2/4 | 1/4 | 1 |
| body_recounts_earlier_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| born_digital | 2 | 2/2 | 2/2 | 2/2 | 0 |
| brand_vs_legal_name | 3 | 0/3 | 2/3 | 2/3 | 1 |
| business_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_table | 6 | 4/6 | 6/6 | 5/6 | 0 |
| caption_table_flattened | 1 | 1/1 | 1/1 | 1/1 | 0 |
| case_citations | 1 | 1/1 | 1/1 | 1/1 | 0 |
| cc_in_body | 2 | 0/2 | 2/2 | 0/2 | 2 |
| cc_list | 5 | 5/5 | 5/5 | 5/5 | 0 |
| cell_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 3 | 2/3 | 3/3 | 2/3 | 0 |
| class_action | 1 | 1/1 | 1/1 | 1/1 | 0 |
| client_named_in_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| coloured_paper | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 22 | 12/22 | 21/22 | 18/22 | 2 |
| consignee_prominent | 1 | 0/1 | 0/1 | 1/1 | 0 |
| copy_list | 1 | 1/1 | 1/1 | 1/1 | 0 |
| corrected_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 6 | 4/6 | 6/6 | 5/6 | 0 |
| court_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| customer_first | 3 | 0/3 | 2/3 | 0/3 | 2 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_field_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_typo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dated_line_at_end | 7 | 5/7 | 7/7 | 7/7 | 0 |
| dd_mm_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dd_month_yyyy | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_dates | 3 | 1/3 | 3/3 | 3/3 | 0 |
| delivery_contact | 1 | 1/1 | 1/1 | 1/1 | 0 |
| descriptive_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| doc | 4 | 2/4 | 4/4 | 2/4 | 2 |
| docx | 4 | 2/4 | 4/4 | 3/4 | 1 |
| due_date | 2 | 1/2 | 2/2 | 1/2 | 1 |
| ecf_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ecf_header_text_only | 3 | 2/3 | 3/3 | 2/3 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| enclosure_after_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| endorsement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enforcement | 2 | 2/2 | 2/2 | 2/2 | 0 |
| established_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| fax_artifacts | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fax_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| federal_register_stamp | 2 | 1/2 | 2/2 | 2/2 | 0 |
| filename_date_trap | 2 | 1/2 | 2/2 | 1/2 | 1 |
| first_name_salutation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| flight_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnote_citation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| footnotes | 2 | 1/2 | 2/2 | 2/2 | 0 |
| form_layout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 5 | 0/5 | 2/5 | 2/5 | 0 |
| handwritten_annotation | 6 | 3/6 | 5/6 | 4/6 | 2 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 6 | 0/6 | 2/6 | 2/6 | 2 |
| historical | 1 | 1/1 | 1/1 | 1/1 | 0 |
| image | 2 | 2/2 | 2/2 | 2/2 | 0 |
| in_re_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 11 | 6/11 | 8/11 | 7/11 | 3 |
| issuer_not_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| jpeg | 4 | 0/4 | 1/4 | 2/4 | 1 |
| letter | 24 | 14/24 | 23/24 | 17/24 | 7 |
| letter_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letter_style | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 1/2 | 2/2 | 2/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead_officers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 4 | 1/4 | 3/4 | 3/4 | 0 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_named_organisations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| metadata_date_mismatch | 1 | 0/1 | 1/1 | 0/1 | 1 |
| metadata_date_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 1 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 3/4 | 4/4 | 3/4 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_sheets | 1 | 1/1 | 1/1 | 1/1 | 0 |
| native | 7 | 5/7 | 7/7 | 7/7 | 0 |
| native_pdf | 5 | 3/5 | 5/5 | 4/5 | 1 |
| no_dated_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_letterhead_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_subject_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notary_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 20 | 14/20 | 20/20 | 18/20 | 0 |
| notice_of_meeting | 1 | 0/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odt | 4 | 2/4 | 4/4 | 3/4 | 1 |
| office | 13 | 6/13 | 13/13 | 8/13 | 5 |
| old_referenced_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ordinal_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| paid_stamp | 2 | 0/2 | 1/2 | 1/2 | 1 |
| past_due_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| person_addressee | 2 | 2/2 | 2/2 | 2/2 | 0 |
| person_party | 2 | 0/2 | 2/2 | 0/2 | 2 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 1/1 | 0 |
| printer_credit | 1 | 0/1 | 0/1 | 1/1 | 0 |
| product_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_records_release | 3 | 2/3 | 3/3 | 2/3 | 1 |
| publication_note_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| re_line_other_matter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| receipt | 2 | 0/2 | 1/2 | 1/2 | 0 |
| receipted | 1 | 0/1 | 0/1 | 0/1 | 1 |
| received_stamp | 2 | 1/2 | 2/2 | 1/2 | 1 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 5/6 | 6/6 | 6/6 | 0 |
| referenced_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 2/4 | 3/4 | 3/4 | 1 |
| released_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| remit_to | 2 | 1/2 | 2/2 | 2/2 | 0 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rtf | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 20 | 13/20 | 18/20 | 16/20 | 2 |
| schedules_attached | 1 | 1/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 1/1 | 1/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_vs_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sister_company_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sole_proprietor | 1 | 0/1 | 0/1 | 0/1 | 0 |
| source_title_mismatch | 1 | 1/1 | 1/1 | 1/1 | 0 |
| staff_names | 8 | 3/8 | 6/8 | 6/8 | 1 |
| stamp_date | 2 | 1/2 | 1/2 | 2/2 | 0 |
| statement | 1 | 0/1 | 1/1 | 1/1 | 0 |
| status_log | 1 | 1/1 | 1/1 | 1/1 | 0 |
| subject_line_names_statute | 1 | 0/1 | 1/1 | 0/1 | 1 |
| subject_line_other_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| table_layout | 2 | 1/2 | 2/2 | 2/2 | 0 |
| termination | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 1 | 0/1 | 1/1 | 0/1 | 0 |
| tiff | 5 | 3/5 | 5/5 | 4/5 | 1 |
| two_addressees | 3 | 2/3 | 3/3 | 2/3 | 1 |
| two_digit_year | 2 | 2/2 | 2/2 | 2/2 | 0 |
| txt | 2 | 1/2 | 2/2 | 1/2 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| uk | 9 | 5/9 | 9/9 | 7/9 | 2 |
| uk_date_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk_government | 7 | 3/7 | 7/7 | 3/7 | 4 |
| uk_regulator | 3 | 1/3 | 3/3 | 3/3 | 0 |
| us_federal | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_state_government | 3 | 2/3 | 3/3 | 2/3 | 1 |
| utf8_bom | 1 | 1/1 | 1/1 | 1/1 | 0 |
| vat | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wmo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| xlsx | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_title | 1 | 0/1 | 1/1 | 0/1 | 1 |
| zero_amount | 1 | 1/1 | 1/1 | 1/1 | 0 |

## Routes

- **Pages:** fast 60 · layout 11 · ocr 69 · ocr_regions 1
- **Documents by route class:** fast 25 · layout 7 · ocr 25 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 58 | 47.03 s | 90.75 s | 99.95 s |
| `extraction_wall_ms` | 58 | 10.7 ms | 14.18 s | 39.26 s |
| `worker_total_ms` | 58 | 10.4 ms | 14.18 s | 39.26 s |
| `worker_snapshot_ms` | 58 | 0.22 ms | 0.89 ms | 2.8 ms |
| `worker_parse_ms` | 58 | 2.1 ms | 16.0 ms | 1.65 s |
| `worker_analysis_ms` | 58 | 0.21 ms | 1.6 ms | 13.0 ms |
| `worker_render_ms` | 58 | 0.00 ms | 948.6 ms | 6.01 s |
| `worker_image_decode_ms` | 58 | 0.00 ms | 77.4 ms | 86.8 ms |
| `worker_ocr_ms` | 58 | 0.00 ms | 27.91 s | 77.20 s |
| `worker_ocr_encode_ms` | 58 | 0.00 ms | 2.14 s | 5.95 s |
| `worker_ocr_engine_ms` | 58 | 0.00 ms | 25.44 s | 70.98 s |
| `worker_vision_ms` | 58 | 0.00 ms | 10.0 ms | 13.9 ms |
| `analyze_wall_ms` | 58 | 45.78 s | 82.46 s | 94.09 s |
| `distill_ms` | 58 | 1.5 ms | 20.9 ms | 39.1 ms |
| `prompt_ms` | 58 | 0.03 ms | 0.04 ms | 0.06 ms |
| `inference_ms` | 58 | 45.77 s | 82.44 s | 94.03 s |
| `prefill_ms` | 58 | 22.76 s | 54.96 s | 70.52 s |
| `generation_ms` | 58 | 22.70 s | 28.66 s | 33.69 s |
| `validation_ms` | 58 | 4.4 ms | 22.0 ms | 28.6 ms |
| `naming_ms` | 58 | 0.20 ms | 2.1 ms | 4.4 ms |
| `index_ms` | 58 | 1.4 ms | 19.0 ms | 35.0 ms |
| `retrieval_ms` | 58 | 0.03 ms | 1.5 ms | 3.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 58 | 36.7 tok/s | 37.4 tok/s | 37.5 tok/s |
| `generation_tok_per_s` | 58 | 4.1 tok/s | 4.2 tok/s | 4.3 tok/s |
| `prompt_tokens` | 58 | 835 | 2053 | 2593 |
| `cached_tokens` | 58 | 477 | 477 | 477 |
| `generated_tokens` | 58 | 92 | 117 | 136 |
| `estimated_prompt_tokens` | 58 | 1460 | 2699 | 3333 |
| `model_requests` | 58 | 1 | 1 | 1 |
| `redistillations` | 58 | 0 | 0 | 0 |
| `source_characters` | 58 | 2504 | 43392 | 76305 |
| `digest_characters` | 58 | 2837 | 6796 | 8597 |
| `prompt_characters` | 58 | 4886 | 8845 | 10646 |
| `worker_ocr_pages` | 58 | 0 | 7 | 18 |
| `worker_ocr_passes` | 58 | 0 | 7 | 18 |
| `worker_orientation_passes` | 58 | 0 | 7 | 18 |
| `worker_rendered_pixels` | 58 | 0 | 58905000 | 156552480 |
| `index_units` | 58 | 24 | 255 | 397 |
| `context_units` | 58 | 23 | 51 | 100 |
| `context_tokens` | 58 | 872 | 2111 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 25 | 2.1 ms | 14.3 ms | 1.8 ms | 12.9 ms | 12.6 ms | 0.58 ms | 0.00 ms | 0.00 ms |
| layout | 7 | 4.6 ms | 128.9 ms | 4.1 ms | 127.2 ms | 112.3 ms | 13.0 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 330.7 ms | 330.7 ms | 330.2 ms | 330.2 ms | 7.1 ms | 1.3 ms | 46.3 ms | 266.3 ms |
| ocr | 25 | 2.18 s | 15.56 s | 2.18 s | 15.56 s | 15.4 ms | 0.85 ms | 973.7 ms | 28.63 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 27 | 1.41 s | 3.02 s | 1.41 s | 3.01 s | 16.0 ms | 0.42 ms | 125.6 ms | 2.82 s |
| 2-4 | 24 | 4.6 ms | 6.04 s | 4.1 ms | 6.04 s | 7.1 ms | 1.3 ms | 442.7 ms | 11.13 s |
| 5-9 | 4 | 10.18 s | 15.56 s | 10.18 s | 15.56 s | 8.2 ms | 1.1 ms | 973.7 ms | 28.63 s |
| 10-24 | 2 | 128.9 ms | 39.26 s | 127.2 ms | 39.26 s | 112.3 ms | 13.0 ms | 6.01 s | 77.20 s |
| 25-49 | 1 | 14.3 ms | 14.3 ms | 12.9 ms | 12.9 ms | 12.6 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 11 | 1.51 s | 3.16 s | 1.51 s | 3.16 s | 16.0 ms | 3.4 ms | 125.6 ms | 3.09 s |
| letter | 24 | 7.0 ms | 8.29 s | 6.4 ms | 8.29 s | 12.5 ms | 1.3 ms | 548.1 ms | 15.11 s |
| notice | 20 | 4.6 ms | 14.18 s | 4.1 ms | 14.18 s | 12.6 ms | 0.97 ms | 948.6 ms | 27.91 s |
| receipt | 2 | 1.84 s | 2.18 s | 1.84 s | 2.18 s | 15.4 ms | 0.12 ms | 0.00 ms | 2.14 s |
| statement | 1 | 1.30 s | 1.30 s | 1.30 s | 1.30 s | 10.8 ms | 0.23 ms | 0.00 ms | 1.24 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 562 MB | 1389 MB | real2-letter-06 |
| Model server | 6085 MB | 6085 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 21 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (24)

- **real2-letter-04**: expected `1974-12-20 Advisory Opinion to ARP Industries, Inc.pdf`, got `1974-12-28 Letter to ARP Industries, Inc.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-letter-06**: expected `2013-09-11 Advice Letter to HM Treasury.pdf`, got `2013-09-11 Letter to The Rt Hon George Osborne MP Chancellor of the Exchequer HM Treasury 1 Horse.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-11**: expected `2022-12-14 Letter of Comfort to Low Carbon Contracts Company.pdf`, got `2022-12-14 Letter to Patrick Bibby Low Carbon Contracts Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-12**: expected `2025-08-20 Guidance Letter to Pool Reinsurance Company Limited.pdf`, got `2025-08-20 Letter from Gwyneth Nurse.pdf` · trap: party Gwyneth Nurse (HM Treasury Director General who signs) · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-13**: expected `2021-12-14 Remit Letter to Armed Forces’ Pay Review Body.odt`, got `2021-12-14 Letter from BEN WALLACE.odt` · trap: party Ben Wallace (Secretary of State who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-05**: expected `2008-01-22 Notice of Entry of Order between American Service Insurance Company and NAFTA General Agency.pdf`, got `2008-01-22 Notice of Entry of Order for Nafta General Agency.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-07**: expected `2003-10-20 Notice of Related Action between Synopsys, Inc and Ricoh Company, Ltd.pdf`, got `2003-10-20 Notice of Related Action Pending in the Northern District of California with Synopsys, Inc.pdf` · **filed without review**
- **real2-notice-10**: expected `2024-06-12 Notice of Technical Conference for Stingray Pipeline Company, L.L.C.pdf`, got `2024-06-12 Document from Federal Energy Regulatory Commission.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real2-invoice-01**: expected `1995-12-04 Ticket Subscription Invoice from USTA.tif`, got `1995-12-04 1996 Ticket Subscription Invoice from Philip Morris.tif` · trap: party Philip Morris (customer (account name), not the issuer) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-03**: expected `1996-02-15 Statement from The Hearst Corporation.tif`, got `1996-02-15 Service Center Statement from Hearst Corporation.tif` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-06**: expected `1882-10-01 Bill from Charles Williams, Jr.jpg`, got `Bill from CHARLES WILLIAMS, Jr.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-07**: expected `1897-01-02 Bill from Pollok & Mauro.jpg`, got `Document from Alexander Graham Bell.jpg` · trap: party Alexander Graham Bell (client billed, printed first) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-08**: expected `1864-12-31 Receipt from The United States Telegraph Company.jpg`, got `1864-12-31 Document.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-09**: expected `1860-10-02 Receipt from The Adams Express Company.jpg`, got `Document from The Adams Express Company.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-10**: expected `1923-11-22 Invoice from Times-Mirror Printing & Binding House.pdf`, got `1923-11-22 Corrected Invoice from Chas F Lummis.pdf` · trap: party Chas F Lummis (customer, typed above the letterhead) · review: DATE_UNSUPPORTED
- **real2-invoice-12**: expected `1895-12-09 Bill from Geo. P. McNear.pdf`, got `Document from Ru fush.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-office-01**: expected `2023-06-22 Request Letter to Ofsted.docx`, got `2023-06-22 Letter to Amanda Spielman.docx` · trap: party Amanda Spielman (individual Chief Inspector addressed; the organisation is Ofsted) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-04**: expected `2014-11-17 Announcement of Consultation from Environment Agency.docx`, got `2014-11-17 Environmental Assessment of Plans and Programmes Regulations from Environment Agency.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-07**: expected `2024-03-28 Response Letter to Welsh Government.odt`, got `2024-03-28 Letter from Liz Kendall.odt` · trap: party Liz Kendall (Secretary of State who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED
- **real2-office-08**: expected `2026-01-29 Notice of Proposal to Modify Definitive Map Order for North Yorkshire Council.odt`, got `2026-01-29 Notice of Proposal to Modify Definitive Map Order from North Yorkshire Council.odt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-10**: expected `2011-07-19 Response Letter to John Mann MP.doc`, got `2011-07-19 Letter to SIR GUS O'DONNELL KCB.doc` · trap: party Gus O'Donnell (Cabinet Secretary who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-11**: expected `2013-04-08 Advice Letter to Lord Strathclyde CH PC.doc`, got `2013-04-08 Document - Lord Lang of Monkton.doc` · trap: party Lord Lang of Monkton (committee chairman who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-14**: expected `2016-06-21 Email to ICC.msg`, got `2016-06-21 Email to Mark Johnson.msg` · trap: party Mark Johnson (ICC staff member addressed) · review: TYPE_INFERRED, LOW_CONFIDENCE
- **real2-office-15**: expected `2024-06-21 Public Information Statement from National Weather Service.txt`, got `2024-06-21 Public Information Statement.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
