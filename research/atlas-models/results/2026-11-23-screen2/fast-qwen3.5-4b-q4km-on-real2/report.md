# InternBench: live run

- **Run:** 2026-11-23T17:04:29Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 54 documents · 54 completed · gold `7d547be379`
- **Wall time:** 1734.67 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 25/54 (46.3%) |
| Document type | 47/54 (87.0%) |
| Date | 47/54 (87.0%) |
| Date role (when the reviewed date was chosen) | 26/47 (55.3%) |
| Parties | 39/54 (72.2%) |
| Relation word (when parties were named) | 40/53 (75.5%) |
| Party in the right role (when parties were named) | 21/34 (61.8%) |
| Ready / review routing | 3/33 (9.1%) |
| Description states nothing false | 54/54 (100.0%) |
| Description is specific | 45/54 (83.3%) |
| Description specificity | 70.4% (mean of 54) |
| Evidence recall (model's quotes) | 76.2% (mean of 54) |
| Digest recall (facts reaching the digest) | 94.3% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 94.3% (mean of 54) |
| Sent to review | 92.6% of 54 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 74.1% | 54 |
| Review rate | 92.6% | 54 |
| Evidence recall | 76.2% | 54 |
| Total latency p50 | 29.03 s | 54 |
| Total latency p95 | 58.65 s | 54 |
| Generation latency p50 | 13.19 s | 54 |
| Generation latency p95 | 17.12 s | 54 |
| Generated tokens p50 | 90 | 54 |
| Generated tokens p95 | 114 | 54 |
| Prompt tokens p50 | 848 | 54 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 54 |
| Trap date chosen | 2 of 54 |
| Forbidden party named | 9 of 54 |
| Spurious parties named | 14 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 138 (0.0%) |
| Right name sent to review anyway | 13 of 33 |
| Sent to review for an unsupported fact | 40 of 54 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 8 | 3/8 | 5/8 | 5/8 | 0/5 | 100.0% | 23.03 s | 36.83 s |
| letter | 24 | 9/24 | 23/24 | 17/24 | 1/13 | 91.7% | 26.96 s | 58.65 s |
| notice | 20 | 13/20 | 18/20 | 17/20 | 2/14 | 90.0% | 33.30 s | 54.98 s |
| receipt | 2 | 0/2 | 1/2 | 0/2 | 0/1 | 100.0% | 16.10 s | 21.70 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 1/8 | 5/8 | 3/8 | 0/5 | 100.0% | 21.56 s | 29.38 s |
| native | 15 | 8/15 | 13/15 | 14/15 | 2/11 | 86.7% | 29.50 s | 50.47 s |
| office | 17 | 8/17 | 16/17 | 12/17 | 1/8 | 88.2% | 27.95 s | 59.78 s |
| scan | 12 | 7/12 | 11/12 | 9/12 | 0/7 | 100.0% | 39.67 s | 72.71 s |
| text | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 30.09 s | 36.83 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 8/23 | 18/23 | 15/23 | 1/12 | 91.3% | 22.14 s | 29.71 s |
| 2-4 | 24 | 11/24 | 22/24 | 18/24 | 2/16 | 91.7% | 30.01 s | 50.47 s |
| 5-9 | 4 | 4/4 | 4/4 | 4/4 | 0/3 | 100.0% | 46.33 s | 54.98 s |
| 10-24 | 2 | 1/2 | 2/2 | 1/2 | 0/1 | 100.0% | 46.12 s | 72.71 s |
| 25-49 | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 59.78 s | 59.78 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 2/3 | 3/3 | 2/3 | 0/2 | 0 of 3 | 59.78 s | 72.71 s | 16.21 s | 17.37 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_person_at_company | 7 | 3/7 | 6/7 | 6/7 | 1 |
| addressed_to_person_at_firm | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressee_at_bottom | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_at_foot | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_in_frame | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| advisory_opinion | 2 | 1/2 | 1/2 | 2/2 | 0 |
| agency_billing | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_enforcement | 7 | 7/7 | 7/7 | 7/7 | 0 |
| agency_notice | 2 | 1/2 | 1/2 | 2/2 | 1 |
| aka_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ambiguous_digit | 1 | 0/1 | 0/1 | 0/1 | 0 |
| annex | 1 | 0/1 | 1/1 | 1/1 | 0 |
| annex_table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachments | 2 | 2/2 | 2/2 | 2/2 | 0 |
| australian_government | 1 | 0/1 | 0/1 | 1/1 | 1 |
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to_prominent | 1 | 0/1 | 1/1 | 0/1 | 0 |
| billhead | 3 | 0/3 | 1/3 | 1/3 | 0 |
| body_recounts_earlier_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| born_digital | 2 | 2/2 | 2/2 | 2/2 | 0 |
| brand_vs_legal_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| business_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_table | 6 | 3/6 | 6/6 | 4/6 | 0 |
| caption_table_flattened | 1 | 1/1 | 1/1 | 1/1 | 0 |
| case_citations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_in_body | 2 | 0/2 | 2/2 | 1/2 | 1 |
| cc_list | 5 | 3/5 | 5/5 | 5/5 | 0 |
| cell_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 3 | 2/3 | 3/3 | 2/3 | 0 |
| class_action | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_named_in_body | 1 | 0/1 | 1/1 | 1/1 | 0 |
| competing_dates | 20 | 11/20 | 17/20 | 16/20 | 4 |
| consignee_prominent | 1 | 0/1 | 0/1 | 0/1 | 1 |
| copy_list | 1 | 1/1 | 1/1 | 1/1 | 0 |
| court_filing | 6 | 3/6 | 6/6 | 4/6 | 0 |
| court_letter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| customer_first | 2 | 0/2 | 1/2 | 0/2 | 1 |
| date_at_end | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_field_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_typo | 1 | 0/1 | 1/1 | 0/1 | 1 |
| dated_line_at_end | 7 | 4/7 | 6/7 | 6/7 | 1 |
| dd_mm_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dd_month_yyyy | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_dates | 3 | 2/3 | 3/3 | 3/3 | 0 |
| delivery_contact | 1 | 1/1 | 1/1 | 1/1 | 0 |
| descriptive_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| doc | 4 | 2/4 | 4/4 | 2/4 | 2 |
| docx | 4 | 1/4 | 4/4 | 3/4 | 1 |
| due_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ecf_header | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ecf_header_text_only | 3 | 2/3 | 3/3 | 2/3 | 0 |
| email | 2 | 2/2 | 2/2 | 2/2 | 0 |
| enclosure_after_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enclosures | 1 | 1/1 | 1/1 | 1/1 | 0 |
| endorsement | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enforcement | 2 | 2/2 | 2/2 | 2/2 | 0 |
| established_year | 1 | 0/1 | 0/1 | 1/1 | 0 |
| fax_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| federal_register_stamp | 2 | 1/2 | 1/2 | 2/2 | 1 |
| filename_date_trap | 2 | 0/2 | 2/2 | 1/2 | 1 |
| first_name_salutation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| flight_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| footnote_citation | 1 | 0/1 | 0/1 | 1/1 | 1 |
| footnotes | 2 | 0/2 | 2/2 | 1/2 | 1 |
| form_layout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 5 | 0/5 | 2/5 | 1/5 | 1 |
| handwritten_annotation | 3 | 0/3 | 2/3 | 1/3 | 1 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten_salutation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 5 | 0/5 | 1/5 | 1/5 | 2 |
| historical | 1 | 1/1 | 1/1 | 1/1 | 0 |
| image | 2 | 1/2 | 2/2 | 2/2 | 0 |
| in_re_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 8 | 3/8 | 5/8 | 5/8 | 1 |
| issuer_not_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| jpeg | 4 | 0/4 | 1/4 | 0/4 | 2 |
| letter | 24 | 9/24 | 23/24 | 17/24 | 7 |
| letter_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letter_style | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 1/2 | 2/2 | 2/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_officers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 3 | 1/3 | 2/3 | 1/3 | 0 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| long_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| many_named_organisations | 1 | 0/1 | 1/1 | 0/1 | 1 |
| memo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| metadata_date_trap | 1 | 0/1 | 1/1 | 1/1 | 0 |
| msg | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_defendants | 4 | 2/4 | 4/4 | 2/4 | 0 |
| multiple_parties | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_sheets | 1 | 1/1 | 1/1 | 1/1 | 0 |
| native | 7 | 4/7 | 6/7 | 6/7 | 1 |
| native_pdf | 5 | 2/5 | 5/5 | 5/5 | 0 |
| no_dated_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_explicit_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_letterhead | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_letterhead_text | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_subject_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notary_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 20 | 13/20 | 18/20 | 17/20 | 2 |
| notice_of_meeting | 1 | 0/1 | 0/1 | 1/1 | 1 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| odt | 4 | 2/4 | 4/4 | 3/4 | 1 |
| office | 13 | 4/13 | 12/13 | 8/13 | 6 |
| old_referenced_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ordinal_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| paid_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| person_addressee | 2 | 1/2 | 2/2 | 2/2 | 0 |
| person_party | 2 | 0/2 | 2/2 | 0/2 | 2 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| printed_form | 1 | 0/1 | 0/1 | 0/1 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_records_release | 3 | 3/3 | 3/3 | 3/3 | 0 |
| publication_note_date | 1 | 0/1 | 0/1 | 1/1 | 1 |
| re_line_other_matter | 2 | 1/2 | 2/2 | 2/2 | 0 |
| receipt | 2 | 0/2 | 1/2 | 0/2 | 1 |
| receipted | 1 | 0/1 | 0/1 | 0/1 | 1 |
| received_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 1/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 6 | 3/6 | 6/6 | 4/6 | 2 |
| referenced_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 1/4 | 3/4 | 4/4 | 0 |
| released_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| remit_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rtf | 1 | 0/1 | 0/1 | 1/1 | 1 |
| scanned | 16 | 7/16 | 14/16 | 12/16 | 2 |
| schedules_attached | 1 | 1/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 1/1 | 1/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 1/2 | 2/2 | 1/2 | 0 |
| signatory_vs_org | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sister_company_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sole_proprietor | 1 | 0/1 | 0/1 | 1/1 | 0 |
| source_title_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 5 | 1/5 | 3/5 | 1/5 | 1 |
| stamp_date | 2 | 1/2 | 1/2 | 2/2 | 0 |
| status_log | 1 | 1/1 | 1/1 | 1/1 | 0 |
| subject_line_names_statute | 1 | 0/1 | 1/1 | 1/1 | 0 |
| subject_line_other_party | 1 | 0/1 | 1/1 | 0/1 | 1 |
| table | 1 | 1/1 | 1/1 | 1/1 | 0 |
| table_layout | 2 | 1/2 | 2/2 | 2/2 | 0 |
| termination | 1 | 1/1 | 1/1 | 1/1 | 0 |
| text | 1 | 0/1 | 1/1 | 0/1 | 0 |
| tiff | 2 | 0/2 | 2/2 | 1/2 | 0 |
| two_addressees | 3 | 3/3 | 3/3 | 3/3 | 0 |
| two_digit_year | 2 | 1/2 | 2/2 | 1/2 | 0 |
| txt | 2 | 1/2 | 2/2 | 1/2 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| uk | 9 | 3/9 | 9/9 | 6/9 | 3 |
| uk_date_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk_government | 7 | 1/7 | 7/7 | 3/7 | 4 |
| uk_regulator | 3 | 2/3 | 3/3 | 3/3 | 0 |
| us_federal | 1 | 0/1 | 1/1 | 0/1 | 0 |
| us_state_government | 3 | 3/3 | 3/3 | 3/3 | 0 |
| utf8_bom | 1 | 1/1 | 1/1 | 1/1 | 0 |
| vat | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wmo_header | 1 | 0/1 | 1/1 | 0/1 | 0 |
| xlsx | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| zero_amount | 1 | 0/1 | 1/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 60 · layout 11 · ocr 65 · ocr_regions 1
- **Documents by route class:** fast 25 · layout 7 · ocr 21 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 54 | 29.03 s | 58.65 s | 72.71 s |
| `extraction_wall_ms` | 54 | 10.1 ms | 14.21 s | 39.54 s |
| `worker_total_ms` | 54 | 9.3 ms | 14.20 s | 39.54 s |
| `worker_snapshot_ms` | 54 | 0.22 ms | 0.60 ms | 1.8 ms |
| `worker_parse_ms` | 54 | 1.7 ms | 14.6 ms | 2.12 s |
| `worker_analysis_ms` | 54 | 0.19 ms | 1.7 ms | 12.9 ms |
| `worker_render_ms` | 54 | 0.00 ms | 939.8 ms | 6.22 s |
| `worker_image_decode_ms` | 54 | 0.00 ms | 77.7 ms | 87.8 ms |
| `worker_ocr_ms` | 54 | 0.00 ms | 27.96 s | 77.54 s |
| `worker_ocr_encode_ms` | 54 | 0.00 ms | 2.10 s | 6.15 s |
| `worker_ocr_engine_ms` | 54 | 0.00 ms | 25.62 s | 71.11 s |
| `worker_vision_ms` | 54 | 0.00 ms | 10.1 ms | 13.7 ms |
| `analyze_wall_ms` | 54 | 27.27 s | 50.30 s | 59.76 s |
| `distill_ms` | 54 | 1.6 ms | 21.0 ms | 39.0 ms |
| `prompt_ms` | 54 | 0.03 ms | 0.05 ms | 0.06 ms |
| `inference_ms` | 54 | 27.26 s | 50.28 s | 59.71 s |
| `prefill_ms` | 54 | 14.04 s | 34.31 s | 43.49 s |
| `generation_ms` | 54 | 13.19 s | 17.12 s | 17.37 s |
| `validation_ms` | 54 | 4.6 ms | 20.9 ms | 27.4 ms |
| `naming_ms` | 54 | 0.21 ms | 2.4 ms | 4.4 ms |
| `index_ms` | 54 | 1.4 ms | 19.1 ms | 34.9 ms |
| `retrieval_ms` | 54 | 0.03 ms | 1.6 ms | 3.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 54 | 60.1 tok/s | 61.6 tok/s | 61.8 tok/s |
| `generation_tok_per_s` | 54 | 6.8 tok/s | 7.2 tok/s | 7.3 tok/s |
| `prompt_tokens` | 54 | 848 | 2053 | 2593 |
| `cached_tokens` | 54 | 477 | 477 | 477 |
| `generated_tokens` | 54 | 90 | 114 | 116 |
| `estimated_prompt_tokens` | 54 | 1502 | 2699 | 3333 |
| `model_requests` | 54 | 1 | 1 | 1 |
| `redistillations` | 54 | 0 | 0 | 0 |
| `source_characters` | 54 | 2779 | 43392 | 76305 |
| `digest_characters` | 54 | 2849 | 6796 | 8597 |
| `prompt_characters` | 54 | 4898 | 8845 | 10646 |
| `worker_ocr_pages` | 54 | 0 | 7 | 18 |
| `worker_ocr_passes` | 54 | 0 | 7 | 18 |
| `worker_orientation_passes` | 54 | 0 | 7 | 18 |
| `worker_rendered_pixels` | 54 | 0 | 58905000 | 156552480 |
| `index_units` | 54 | 24 | 255 | 397 |
| `context_units` | 54 | 23 | 51 | 100 |
| `context_tokens` | 54 | 915 | 2111 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 25 | 2.0 ms | 14.6 ms | 1.7 ms | 13.1 ms | 12.8 ms | 0.56 ms | 0.00 ms | 0.00 ms |
| layout | 7 | 5.0 ms | 128.1 ms | 4.4 ms | 126.4 ms | 111.7 ms | 12.9 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 338.7 ms | 338.7 ms | 338.3 ms | 338.3 ms | 7.2 ms | 1.2 ms | 47.3 ms | 273.4 ms |
| ocr | 21 | 2.19 s | 15.61 s | 2.19 s | 15.61 s | 14.5 ms | 0.86 ms | 1.00 s | 28.58 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 1.51 s | 2.19 s | 1.51 s | 2.19 s | 14.6 ms | 0.37 ms | 113.8 ms | 2.15 s |
| 2-4 | 24 | 4.6 ms | 6.55 s | 4.3 ms | 6.55 s | 7.2 ms | 1.2 ms | 505.1 ms | 11.99 s |
| 5-9 | 4 | 9.73 s | 15.61 s | 9.73 s | 15.61 s | 8.1 ms | 1.1 ms | 1.00 s | 28.58 s |
| 10-24 | 2 | 128.1 ms | 39.54 s | 126.4 ms | 39.54 s | 111.7 ms | 12.9 ms | 6.22 s | 77.54 s |
| 25-49 | 1 | 14.6 ms | 14.6 ms | 13.1 ms | 13.1 ms | 12.8 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 8 | 1.51 s | 2.17 s | 1.51 s | 2.17 s | 12.0 ms | 3.4 ms | 113.8 ms | 2.06 s |
| letter | 24 | 6.9 ms | 8.35 s | 6.5 ms | 8.35 s | 14.5 ms | 1.2 ms | 577.0 ms | 15.30 s |
| notice | 20 | 4.6 ms | 14.21 s | 4.3 ms | 14.20 s | 12.8 ms | 0.98 ms | 939.8 ms | 27.96 s |
| receipt | 2 | 1.89 s | 2.19 s | 1.89 s | 2.19 s | 14.6 ms | 0.12 ms | 0.00 ms | 2.15 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 556 MB | 1392 MB | real2-letter-06 |
| Model server | 3266 MB | 3266 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 18 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (29)

- **real2-letter-01**: expected `2008-12-05 Approval Letter to Cleary Gottlieb Steen & Hamilton LLP.pdf`, got `2008-12-05 Letter from Federal Reserve System.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-04**: expected `1974-12-20 Advisory Opinion to ARP Industries, Inc.pdf`, got `1974-12-28 Letter to ARP Industries, Inc.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-05**: expected `2014-06-30 Extension Letter to PricewaterhouseCoopers LLP.pdf`, got `2014-06-30 Letter to Paul Rowsell.pdf` · trap: party Paul Rowsell (DCLG Deputy Director who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-06**: expected `2013-09-11 Advice Letter to HM Treasury.pdf`, got `2013-09-11 Letter from Clive Maxwell CBE.pdf` · trap: party Clive Maxwell (OFT Chief Executive who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-10**: expected `2024-10-07 Letter between Nike, Inc and StockX LLC.pdf`, got `2024-10-07 Letter to Nike, Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-11**: expected `2022-12-14 Letter of Comfort to Low Carbon Contracts Company.pdf`, got `2022-12-14 Letter to Patrick Bibby Low Carbon Contracts Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-12**: expected `2025-08-20 Guidance Letter to Pool Reinsurance Company Limited.pdf`, got `2025-08-20 Letter to Tom Clementi Pool Reinsurance Company Limited.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-13**: expected `2021-12-14 Remit Letter to Armed Forces’ Pay Review Body.odt`, got `2021-12-14 Letter from Ben Wallace.odt` · trap: party Ben Wallace (Secretary of State who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-15**: expected `2013-11-29 Response Letter to Baroness Lane-Fox of Soho.jpg`, got `2013-11-29 Letter to The Right Honourable The Baroness Lane-Fox of Soho CBE.jpg` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-05**: expected `2008-01-22 Notice of Entry of Order between American Service Insurance Company and NAFTA General Agency.pdf`, got `2008-01-22 Notice of Entry of Order between American Service Insurance Company and Underwriters MGA, Inc.pdf` · review: LOW_CONFIDENCE
- **real2-notice-07**: expected `2003-10-20 Notice of Related Action between Synopsys, Inc and Ricoh Company, Ltd.pdf`, got `2003-10-20 Notice of Related Action Pending in the Northern District of California with SYNOPSYS, INC.pdf` · review: LOW_CONFIDENCE
- **real2-notice-08**: expected `2012-12-20 Notice of Motion between Marylebone PCC Limited—Rose 2 Fund and Millennium Global Investments Ltd.pdf`, got `2012-12-20 Notice of Motion for Marylebone PCC Limited -- Rose 2 Fund.pdf` · review: PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-10**: expected `2024-06-12 Notice of Technical Conference for Stingray Pipeline Company, L.L.C.pdf`, got `2024-06-18 Document from Federal Energy Regulatory Commission.pdf` · trap: date 2024-06-18 (scheduled Federal Register publication date) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-02**: expected `1995-07-20 Invoice from The New York Academy of Sciences.tif`, got `1995-07-20 INVOICE.tif` · review: PARTY_MISSING, LOW_CONFIDENCE
- **real2-invoice-04**: expected `1989-03-06 Invoice from International Union of Biochemistry.tif`, got `1989-03-06 Invoice from International Union.tif` · review: LOW_CONFIDENCE
- **real2-invoice-06**: expected `1882-10-01 Bill from Charles Williams, Jr.jpg`, got `Bill from M Chrof. A. b. Bill.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-07**: expected `1897-01-02 Bill from Pollok & Mauro.jpg`, got `Document from Alexander Graham Bell, Esq.jpg` · trap: party Alexander Graham Bell (client billed, printed first) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-08**: expected `1864-12-31 Receipt from The United States Telegraph Company.jpg`, got `1864-12-31 Document - M Cx Mansion.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real2-invoice-09**: expected `1860-10-02 Receipt from The Adams Express Company.jpg`, got `Contract between The Adams Express Company and Gustavus V. Town.jpg` · trap: party Gustavus V. Town (printer of the blank form) · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-12**: expected `1895-12-09 Bill from Geo. P. McNear.pdf`, got `Document - GEO. P. MCNEAR.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-office-01**: expected `2023-06-22 Request Letter to Ofsted.docx`, got `2023-06-22 Letter to Minister for Women and Equalities.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-02**: expected `2019-11-04 Remit and Recommendations Letter to Bank of England.docx`, got `2019-11-04 Letter from Mark Carney.docx` · trap: party Mark Carney (Governor addressed; the organisation is the Bank of England) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-04**: expected `2014-11-17 Announcement of Consultation from Environment Agency.docx`, got `2014-11-17 Environmental Assessment of Plans and Programmes Regulations - Environment Agency.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED
- **real2-office-06**: expected `2019-10-24 Response Letter to ONS.odt`, got `2019-10-24 Letter to Migration Advisory Committee.odt` · review: TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-07**: expected `2024-03-28 Response Letter to Welsh Government.odt`, got `2024-03-28 Letter from Liz Kendall.odt` · trap: party Liz Kendall (Secretary of State who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED
- **real2-office-10**: expected `2011-07-19 Response Letter to John Mann MP.doc`, got `2011-07-19 Document - SIR GUS O'DONNELL KCB.doc` · trap: party Gus O'Donnell (Cabinet Secretary who signs) · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-11**: expected `2013-04-08 Advice Letter to Lord Strathclyde CH PC.doc`, got `2013-04-08 Letter to Lord Lang of Monkton.doc` · trap: party Lord Lang of Monkton (committee chairman who signs) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-12**: expected `2001-07-25 Notice of the Primary Universal Service Provider for Telstra Corporation Limited.rtf`, got `2001-06-01 Notice for Telstra Corporation Limited.rtf` · trap: date 2001-06-01 (date of the Commonwealth-Telstra agreement) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-15**: expected `2024-06-21 Public Information Statement from National Weather Service.txt`, got `2024-06-21 Public Information Statement.txt` · review: PARTY_UNSUPPORTED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
