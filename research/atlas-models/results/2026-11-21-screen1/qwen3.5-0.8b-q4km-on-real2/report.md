# InternBench: live run

- **Run:** 2026-11-21T04:49:46Z · commit `a51447d321`
- **Machine:** AMD EPYC 9V74 80-Core Processor, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 0.53 GB · sha256 `bd258782e35f`
- **Corpus:** 56 documents · 56 completed · gold `df7a4c920b`
- **Wall time:** 478.15 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 11/56 (19.6%) |
| Document type | 48/56 (85.7%) |
| Date | 40/56 (71.4%) |
| Date role (when the reviewed date was chosen) | 14/40 (35.0%) |
| Parties | 27/56 (48.2%) |
| Relation word (when parties were named) | 36/53 (67.9%) |
| Party in the right role (when parties were named) | 11/33 (33.3%) |
| Ready / review routing | 0/34 (0.0%) |
| Description states nothing false | 56/56 (100.0%) |
| Description is specific | 37/56 (66.1%) |
| Description specificity | 58.9% (mean of 56) |
| Evidence recall (model's quotes) | 21.7% (mean of 56) |
| Digest recall (facts reaching the digest) | 94.5% (mean of 56) |
| Prompt recall (facts reaching the prompt sent) | 94.5% (mean of 56) |
| Sent to review | 100.0% of 56 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 33.3% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 82.1% | 56 |
| Review rate | 100.0% | 56 |
| Evidence recall | 21.7% | 56 |
| Total latency p50 | 6.09 s | 56 |
| Total latency p95 | 24.42 s | 56 |
| Generation latency p50 | 2.81 s | 56 |
| Generation latency p95 | 3.46 s | 56 |
| Generated tokens p50 | 103 | 56 |
| Generated tokens p95 | 118 | 56 |
| Prompt tokens p50 | 855 | 56 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 56 |
| Trap date chosen | 4 of 56 |
| Forbidden party named | 12 of 56 |
| Spurious parties named | 26 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 130 (0.0%) |
| Right name sent to review anyway | 8 of 34 |
| Sent to review for an unsupported fact | 46 of 56 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 9 | 3/9 | 6/9 | 5/9 | 0/7 | 100.0% | 6.67 s | 10.11 s |
| letter | 25 | 3/25 | 19/25 | 10/25 | 0/13 | 100.0% | 6.01 s | 19.01 s |
| notice | 20 | 5/20 | 15/20 | 12/20 | 0/14 | 100.0% | 7.19 s | 24.42 s |
| statement | 2 | 0/2 | 0/2 | 0/2 | – | 100.0% | 3.99 s | 5.43 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 7 | 2/7 | 5/7 | 4/7 | 0/5 | 100.0% | 5.45 s | 10.11 s |
| native | 16 | 4/16 | 12/16 | 13/16 | 0/11 | 100.0% | 6.09 s | 11.21 s |
| office | 17 | 2/17 | 13/17 | 4/17 | 0/8 | 100.0% | 5.37 s | 12.40 s |
| scan | 14 | 2/14 | 9/14 | 5/14 | 0/8 | 100.0% | 8.78 s | 50.97 s |
| text | 2 | 1/2 | 1/2 | 1/2 | 0/2 | 100.0% | 5.05 s | 8.17 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 24 | 5/24 | 14/24 | 10/24 | 0/13 | 100.0% | 5.10 s | 8.47 s |
| 2-4 | 24 | 5/24 | 21/24 | 12/24 | 0/16 | 100.0% | 6.27 s | 14.81 s |
| 5-9 | 5 | 0/5 | 3/5 | 4/5 | 0/3 | 100.0% | 18.04 s | 24.66 s |
| 10-24 | 2 | 1/2 | 1/2 | 1/2 | 0/1 | 100.0% | 8.90 s | 50.97 s |
| 25-49 | 1 | 0/1 | 1/1 | 0/1 | 0/1 | 100.0% | 12.40 s | 12.40 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 1/3 | 2/3 | 1/3 | 0/2 | 0 of 3 | 12.40 s | 50.97 s | 2.83 s | 3.27 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 0/1 | 1/1 | 0/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 0 |
| addressed_to_person_at_company | 8 | 1/8 | 5/8 | 3/8 | 2 |
| addressed_to_person_at_firm | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_at_bottom | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressee_at_foot | 1 | 0/1 | 1/1 | 1/1 | 0 |
| addressee_in_frame | 1 | 0/1 | 1/1 | 0/1 | 0 |
| addressee_is_office_holder | 1 | 0/1 | 1/1 | 1/1 | 0 |
| advisory_opinion | 2 | 0/2 | 1/2 | 1/2 | 0 |
| agency_billing | 1 | 0/1 | 0/1 | 1/1 | 0 |
| agency_enforcement | 7 | 3/7 | 6/7 | 5/7 | 1 |
| agency_notice | 2 | 0/2 | 1/2 | 2/2 | 1 |
| aka_name | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex | 1 | 0/1 | 1/1 | 0/1 | 1 |
| annex_table | 1 | 0/1 | 1/1 | 0/1 | 1 |
| attachments | 2 | 0/2 | 2/2 | 0/2 | 1 |
| australian_government | 1 | 0/1 | 0/1 | 0/1 | 2 |
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to_prominent | 1 | 0/1 | 0/1 | 0/1 | 0 |
| billhead | 3 | 0/3 | 1/3 | 0/3 | 1 |
| body_recounts_earlier_dates | 2 | 1/2 | 2/2 | 2/2 | 0 |
| born_digital | 2 | 1/2 | 1/2 | 2/2 | 0 |
| brand_vs_legal_name | 3 | 0/3 | 1/3 | 0/3 | 1 |
| business_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| caption_table | 6 | 1/6 | 6/6 | 4/6 | 1 |
| caption_table_flattened | 1 | 1/1 | 1/1 | 1/1 | 0 |
| case_citations | 1 | 0/1 | 1/1 | 1/1 | 0 |
| cc_in_body | 2 | 0/2 | 2/2 | 0/2 | 1 |
| cc_list | 5 | 1/5 | 5/5 | 3/5 | 0 |
| cell_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| certificate_of_service | 3 | 2/3 | 3/3 | 2/3 | 0 |
| class_action | 1 | 0/1 | 1/1 | 1/1 | 0 |
| client_named_in_body | 1 | 0/1 | 1/1 | 0/1 | 0 |
| coloured_paper | 1 | 0/1 | 1/1 | 0/1 | 1 |
| competing_dates | 21 | 5/21 | 14/21 | 10/21 | 8 |
| copy_list | 1 | 0/1 | 0/1 | 1/1 | 0 |
| corrected_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 6 | 2/6 | 6/6 | 5/6 | 0 |
| court_letter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 1 |
| date_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| date_at_top | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_field_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| date_typo | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dated_line_at_end | 7 | 2/7 | 6/7 | 7/7 | 1 |
| dd_mm_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| dd_month_yyyy | 1 | 0/1 | 1/1 | 1/1 | 0 |
| deadline_dates | 3 | 0/3 | 2/3 | 0/3 | 1 |
| delivery_contact | 1 | 1/1 | 1/1 | 1/1 | 0 |
| descriptive_title | 1 | 1/1 | 1/1 | 1/1 | 0 |
| doc | 4 | 0/4 | 2/4 | 1/4 | 3 |
| docx | 4 | 1/4 | 3/4 | 2/4 | 1 |
| due_date | 2 | 1/2 | 2/2 | 1/2 | 1 |
| ecf_header | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ecf_header_text_only | 3 | 1/3 | 3/3 | 2/3 | 0 |
| email | 2 | 0/2 | 2/2 | 0/2 | 1 |
| enclosure_after_notice | 1 | 0/1 | 1/1 | 0/1 | 0 |
| enclosures | 1 | 0/1 | 1/1 | 1/1 | 0 |
| enforcement | 3 | 1/3 | 2/3 | 2/3 | 0 |
| established_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| fax_artifacts | 1 | 1/1 | 1/1 | 1/1 | 0 |
| fax_header | 1 | 0/1 | 1/1 | 1/1 | 0 |
| federal_register_stamp | 2 | 0/2 | 1/2 | 2/2 | 1 |
| filename_date_trap | 2 | 0/2 | 2/2 | 1/2 | 0 |
| first_name_salutation | 1 | 0/1 | 1/1 | 0/1 | 0 |
| flight_dates | 1 | 0/1 | 0/1 | 1/1 | 0 |
| footnote_citation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| footnotes | 2 | 0/2 | 1/2 | 0/2 | 1 |
| form_layout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten | 3 | 0/3 | 1/3 | 0/3 | 0 |
| handwritten_annotation | 5 | 1/5 | 3/5 | 2/5 | 1 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 0/1 | 0 |
| handwritten_salutation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| historic | 3 | 0/3 | 1/3 | 0/3 | 1 |
| historical | 1 | 1/1 | 1/1 | 1/1 | 0 |
| image | 2 | 1/2 | 2/2 | 2/2 | 0 |
| in_re_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 9 | 3/9 | 6/9 | 5/9 | 2 |
| issuer_not_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letter | 25 | 3/25 | 19/25 | 10/25 | 8 |
| letter_format | 2 | 0/2 | 1/2 | 1/2 | 0 |
| letter_style | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 1/2 | 2/2 | 1/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_officers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 3 | 0/3 | 0/3 | 1/3 | 0 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 0 |
| long_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| many_companies_named | 1 | 0/1 | 1/1 | 0/1 | 1 |
| many_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| many_named_organisations | 1 | 0/1 | 0/1 | 0/1 | 1 |
| memo_header | 1 | 0/1 | 0/1 | 0/1 | 1 |
| metadata_date_mismatch | 1 | 0/1 | 1/1 | 0/1 | 1 |
| metadata_date_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| msg | 2 | 0/2 | 2/2 | 0/2 | 1 |
| multi_page | 1 | 0/1 | 0/1 | 1/1 | 0 |
| multiple_defendants | 4 | 1/4 | 4/4 | 3/4 | 0 |
| multiple_parties | 1 | 0/1 | 0/1 | 1/1 | 0 |
| multiple_sheets | 1 | 1/1 | 1/1 | 1/1 | 0 |
| name_variant | 1 | 0/1 | 0/1 | 1/1 | 0 |
| native | 7 | 2/7 | 6/7 | 7/7 | 1 |
| native_pdf | 6 | 1/6 | 5/6 | 4/6 | 1 |
| no_dated_letterhead | 1 | 0/1 | 0/1 | 1/1 | 0 |
| no_dated_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_explicit_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| no_letterhead_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_subject_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_text_layer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notary_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 20 | 5/20 | 15/20 | 12/20 | 6 |
| notice_of_meeting | 1 | 0/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odt | 4 | 0/4 | 4/4 | 0/4 | 2 |
| office | 13 | 1/13 | 9/13 | 3/13 | 7 |
| ordinal_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| paid_stamp | 1 | 0/1 | 1/1 | 0/1 | 1 |
| past_due_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| person_addressee | 2 | 1/2 | 2/2 | 2/2 | 0 |
| person_party | 2 | 0/2 | 1/2 | 0/2 | 2 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| product_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_records_release | 3 | 0/3 | 2/3 | 1/3 | 1 |
| publication_note_date | 1 | 0/1 | 0/1 | 0/1 | 2 |
| re_line_other_matter | 2 | 0/2 | 2/2 | 2/2 | 0 |
| received_stamp | 2 | 1/2 | 2/2 | 1/2 | 1 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 0/1 | 0/1 | 1 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 7 | 1/7 | 4/7 | 3/7 | 2 |
| referenced_document | 1 | 0/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 0/4 | 3/4 | 1/4 | 1 |
| released_date | 2 | 1/2 | 2/2 | 1/2 | 1 |
| remit_to | 2 | 0/2 | 0/2 | 1/2 | 0 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| rtf | 1 | 0/1 | 0/1 | 0/1 | 2 |
| scanned | 21 | 3/21 | 13/21 | 8/21 | 4 |
| schedules_attached | 1 | 0/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 0/1 | 0/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 1/2 | 2/2 | 2/2 | 0 |
| signatory_vs_org | 1 | 0/1 | 1/1 | 1/1 | 0 |
| signature_page_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| sister_company_named | 1 | 0/1 | 1/1 | 0/1 | 0 |
| sole_proprietor | 1 | 0/1 | 0/1 | 0/1 | 0 |
| source_title_mismatch | 1 | 0/1 | 1/1 | 1/1 | 0 |
| staff_names | 5 | 1/5 | 2/5 | 2/5 | 1 |
| stamp_date | 2 | 0/2 | 0/2 | 1/2 | 0 |
| statement | 2 | 0/2 | 0/2 | 0/2 | 0 |
| status_log | 1 | 0/1 | 1/1 | 0/1 | 0 |
| subject_line_names_statute | 1 | 0/1 | 1/1 | 0/1 | 1 |
| subject_line_other_party | 1 | 0/1 | 0/1 | 0/1 | 1 |
| table | 1 | 0/1 | 0/1 | 1/1 | 0 |
| table_layout | 2 | 0/2 | 0/2 | 1/2 | 2 |
| termination | 1 | 0/1 | 0/1 | 1/1 | 1 |
| text | 1 | 0/1 | 0/1 | 0/1 | 1 |
| tiff | 5 | 1/5 | 3/5 | 2/5 | 1 |
| two_addressees | 3 | 0/3 | 2/3 | 1/3 | 2 |
| two_digit_year | 3 | 0/3 | 0/3 | 1/3 | 0 |
| txt | 2 | 1/2 | 1/2 | 1/2 | 1 |
| uk | 9 | 2/9 | 7/9 | 4/9 | 3 |
| uk_date_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| uk_government | 7 | 1/7 | 6/7 | 2/7 | 4 |
| uk_regulator | 3 | 0/3 | 2/3 | 0/3 | 1 |
| us_federal | 1 | 0/1 | 0/1 | 0/1 | 1 |
| us_state_government | 3 | 0/3 | 2/3 | 1/3 | 1 |
| utf8_bom | 1 | 1/1 | 1/1 | 1/1 | 0 |
| vat | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wmo_header | 1 | 0/1 | 0/1 | 0/1 | 1 |
| xlsx | 1 | 1/1 | 1/1 | 1/1 | 0 |
| year_in_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| year_in_title | 1 | 0/1 | 1/1 | 0/1 | 1 |
| zero_amount | 1 | 0/1 | 0/1 | 0/1 | 0 |

## Routes

- **Pages:** fast 65 · layout 12 · ocr 66 · ocr_regions 1
- **Documents by route class:** fast 25 · layout 8 · ocr 22 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 56 | 6.09 s | 24.42 s | 50.97 s |
| `extraction_wall_ms` | 56 | 10.2 ms | 16.05 s | 44.72 s |
| `worker_total_ms` | 56 | 9.8 ms | 16.05 s | 44.72 s |
| `worker_snapshot_ms` | 56 | 0.22 ms | 1.6 ms | 2.8 ms |
| `worker_parse_ms` | 56 | 1.9 ms | 26.2 ms | 201.8 ms |
| `worker_analysis_ms` | 56 | 0.22 ms | 1.6 ms | 12.5 ms |
| `worker_render_ms` | 56 | 0.00 ms | 1.04 s | 6.06 s |
| `worker_image_decode_ms` | 56 | 0.00 ms | 63.1 ms | 80.0 ms |
| `worker_ocr_ms` | 56 | 0.00 ms | 31.46 s | 88.01 s |
| `worker_ocr_encode_ms` | 56 | 0.00 ms | 2.48 s | 6.21 s |
| `worker_ocr_engine_ms` | 56 | 0.00 ms | 28.67 s | 81.55 s |
| `worker_vision_ms` | 56 | 0.00 ms | 9.4 ms | 12.0 ms |
| `analyze_wall_ms` | 56 | 5.37 s | 9.94 s | 12.39 s |
| `distill_ms` | 56 | 1.6 ms | 20.6 ms | 38.0 ms |
| `prompt_ms` | 56 | 0.03 ms | 0.05 ms | 0.06 ms |
| `inference_ms` | 56 | 5.36 s | 9.91 s | 12.32 s |
| `prefill_ms` | 56 | 2.80 s | 6.88 s | 9.04 s |
| `generation_ms` | 56 | 2.81 s | 3.46 s | 3.94 s |
| `validation_ms` | 56 | 6.0 ms | 23.0 ms | 36.2 ms |
| `naming_ms` | 56 | 0.20 ms | 1.9 ms | 4.1 ms |
| `index_ms` | 56 | 1.5 ms | 18.9 ms | 34.2 ms |
| `retrieval_ms` | 56 | 0.03 ms | 1.5 ms | 3.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 56 | 302.3 tok/s | 310.5 tok/s | 315.1 tok/s |
| `generation_tok_per_s` | 56 | 36.1 tok/s | 38.1 tok/s | 38.4 tok/s |
| `prompt_tokens` | 56 | 855 | 2053 | 2593 |
| `cached_tokens` | 56 | 477 | 477 | 477 |
| `generated_tokens` | 56 | 103 | 118 | 142 |
| `estimated_prompt_tokens` | 56 | 1513 | 2699 | 3333 |
| `model_requests` | 56 | 1 | 1 | 1 |
| `redistillations` | 56 | 0 | 0 | 0 |
| `source_characters` | 56 | 2795 | 43392 | 76305 |
| `digest_characters` | 56 | 2934 | 6796 | 8597 |
| `prompt_characters` | 56 | 4983 | 8845 | 10646 |
| `worker_ocr_pages` | 56 | 0 | 7 | 18 |
| `worker_ocr_passes` | 56 | 0 | 7 | 18 |
| `worker_orientation_passes` | 56 | 0 | 7 | 18 |
| `worker_rendered_pixels` | 56 | 0 | 58905000 | 156552480 |
| `index_units` | 56 | 24 | 255 | 397 |
| `context_units` | 56 | 23 | 51 | 100 |
| `context_tokens` | 56 | 925 | 2111 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 25 | 2.2 ms | 11.0 ms | 1.9 ms | 10.5 ms | 8.9 ms | 0.58 ms | 0.00 ms | 0.00 ms |
| layout | 8 | 5.1 ms | 204.3 ms | 4.4 ms | 203.7 ms | 201.8 ms | 12.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 305.8 ms | 305.8 ms | 305.2 ms | 305.2 ms | 7.4 ms | 1.2 ms | 47.0 ms | 238.8 ms |
| ocr | 22 | 2.16 s | 17.27 s | 2.16 s | 17.27 s | 22.3 ms | 0.85 ms | 1.07 s | 31.55 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 24 | 682.6 ms | 3.18 s | 682.3 ms | 3.18 s | 22.3 ms | 0.38 ms | 146.5 ms | 3.00 s |
| 2-4 | 24 | 4.7 ms | 6.84 s | 4.3 ms | 6.84 s | 7.4 ms | 1.2 ms | 550.2 ms | 12.79 s |
| 5-9 | 5 | 10.66 s | 17.27 s | 10.66 s | 17.27 s | 201.8 ms | 1.3 ms | 1.07 s | 31.55 s |
| 10-24 | 2 | 131.7 ms | 44.72 s | 130.0 ms | 44.72 s | 115.6 ms | 12.5 ms | 6.06 s | 88.01 s |
| 25-49 | 1 | 14.1 ms | 14.1 ms | 12.8 ms | 12.8 ms | 12.5 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 9 | 1.39 s | 3.21 s | 1.39 s | 3.21 s | 26.2 ms | 3.4 ms | 121.2 ms | 3.14 s |
| letter | 25 | 7.6 ms | 9.07 s | 7.3 ms | 9.07 s | 22.3 ms | 1.3 ms | 634.7 ms | 16.50 s |
| notice | 20 | 4.8 ms | 16.05 s | 4.4 ms | 16.05 s | 12.5 ms | 0.97 ms | 1.04 s | 31.46 s |
| statement | 2 | 682.6 ms | 1.12 s | 682.3 ms | 1.12 s | 15.5 ms | 0.23 ms | 146.5 ms | 1.05 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 571 MB | 1399 MB | real2-letter-06 |
| Model server | 856 MB | 856 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 22 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (45)

- **real2-letter-01**: expected `2008-12-05 Approval Letter to Cleary Gottlieb Steen & Hamilton LLP.pdf`, got `2008-12-05 Letter to BNPP.pdf` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-02**: expected `2007-06-25 Letter to Proskauer Rose LLP.pdf`, got `2007-06-25 Letter to Gallion & Spielvogel.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-03**: expected `2015-04-06 Enforcement Letter to Packaging Specialties, Inc.pdf`, got `2015-04-06 Letter - C.SIMOWN.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-04**: expected `1974-12-20 Advisory Opinion to ARP Industries, Inc.pdf`, got `1974-12-28 Letter.pdf` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-letter-05**: expected `2014-06-30 Extension Letter to PricewaterhouseCoopers LLP.pdf`, got `2014-04-04 Letter to Paul Rone.pdf` · trap: date 2014-04-04 (earlier letter appointing PwC) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-06**: expected `2013-09-11 Advice Letter to HM Treasury.pdf`, got `Letter from Clive Maxwell.pdf` · trap: party Clive Maxwell (OFT Chief Executive who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-07**: expected `2018-02-09 Untitled Letter to Collegium Pharmaceutical, Inc.pdf`, got `Letter to Collegium Pharmaceutical, Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-09**: expected `1997-05-16 Advisory Opinion to Chicago Mercantile Exchange.pdf`, got `1997-05-16 Advisory Opinion - Chicago Mercantile Exchange.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-10**: expected `2024-10-07 Letter between Nike, Inc and StockX LLC.pdf`, got `2024-10-07 Letter to Nike, Inc.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-11**: expected `2022-12-14 Letter of Comfort to Low Carbon Contracts Company.pdf`, got `2022-12-14 Letter to Electricity Capacity Regulations 2014.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-12**: expected `2025-08-20 Guidance Letter to Pool Reinsurance Company Limited.pdf`, got `2025-08-20 Letter from Gwyneth Nurse.pdf` · trap: party Gwyneth Nurse (HM Treasury Director General who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-letter-13**: expected `2021-12-14 Remit Letter to Armed Forces’ Pay Review Body.odt`, got `2021-12-14 Letter to AFPRB.odt` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-letter-15**: expected `2013-11-29 Response Letter to Baroness Lane-Fox of Soho.jpg`, got `2013-11-29 Document - The Right Honourable The Baroness Lane-Fox of Soho CBE.jpg` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-01**: expected `2011-03-03 Notice of Probable Violation for Dakota Gasification Company.pdf`, got `2011-03-03 Notice of Probable Violation from Department of Transportation.pdf` · review: LOW_CONFIDENCE
- **real2-notice-02**: expected `2015-09-18 Notice of Violation for Volkswagen AG.pdf`, got `Notice of Violation for Volkswagen AG.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-04**: expected `2008-09-03 Notice of Deposition between Girafa.com, Inc and Amazon Web Services LLC.pdf`, got `2008-09-03 Notice of Deposition between GIRAFA.COM, INC and V. C.A. No. 07-787-SLR AMAZON WEB SERVICES LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real2-notice-05**: expected `2008-01-22 Notice of Entry of Order between American Service Insurance Company and NAFTA General Agency.pdf`, got `2008-01-22 Notice of Entry of Order between American Service Insurance Company and V. Case No. 08 CV 443.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE
- **real2-notice-07**: expected `2003-10-20 Notice of Related Action between Synopsys, Inc and Ricoh Company, Ltd.pdf`, got `2003-10-20 Notice of Related Action Pending in the Northern District of California with Synopsys, Inc.pdf` · review: LOW_CONFIDENCE
- **real2-notice-08**: expected `2012-12-20 Notice of Motion between Marylebone PCC Limited—Rose 2 Fund and Millennium Global Investments Ltd.pdf`, got `2012-12-20 Notice of Motion between Marylebone PCC Limited and BCP Securities LLC.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-09**: expected `2026-04-01 Notice of Intent to Terminate Receivership for Citizens National Bank.pdf`, got `2026-04-03 Notice of Intent to Terminate Receivership from Federal Deposit Insurance Corporation.pdf` · trap: date 2026-04-03 (scheduled Federal Register publication date in the OFR banner and footer) · review: LOW_CONFIDENCE
- **real2-notice-10**: expected `2024-06-12 Notice of Technical Conference for Stingray Pipeline Company, L.L.C.pdf`, got `2024-06-12 Document - Stingray Pipeline Company, L.L.C.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-11**: expected `2020-01-29 Notice of Monetary Penalty for AQA Education.pdf`, got `2020-01-29 Notice of Monetary Penalty to AQA.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-notice-14**: expected `2011-07-20 Notice of Violation for WaveDivision VII, LLC.doc`, got `2011-07-20 Notice of Violation for Binh Nguyen.doc` · trap: party Binh Nguyen (FCC Resident Agent who signs) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-01**: expected `1995-12-04 Ticket Subscription Invoice from USTA.tif`, got `1995-12-04 Ticket Subscription Invoice from Philip Morris.tif` · trap: party Philip Morris (customer (account name), not the issuer) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-02**: expected `1995-07-20 Invoice from The New York Academy of Sciences.tif`, got `Bill to Shp from James F. Chairman of the Board.tif` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-03**: expected `1996-02-15 Statement from The Hearst Corporation.tif`, got `Service Center Statement for Customer Number 1004573.tif` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-invoice-04**: expected `1989-03-06 Invoice from International Union of Biochemistry.tif`, got `1989-03-06 Invoice from International Union.tif` · review: LOW_CONFIDENCE
- **real2-invoice-10**: expected `1923-11-22 Invoice from Times-Mirror Printing & Binding House.pdf`, got `1923-11-22 Invoice from Chas F Lummis.pdf` · trap: party Chas F Lummis (customer, typed above the letterhead) · review: DATE_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-11**: expected `1920-01-13 Statement from Grace Bros. Brewing Co.pdf`, got `Statement.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARTY_MISSING, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-12**: expected `1895-12-09 Bill from Geo. P. McNear.pdf`, got `Document.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real2-invoice-15**: expected `2024-10-20 Invoice from KUSA-TV.pdf`, got `Invoice from KUSA.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-01**: expected `2023-06-22 Request Letter to Ofsted.docx`, got `2023-06-22 Letter to Minister for Women and Equalities.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-03**: expected `2024-07-16 Notice of Variation and Consolidation for Britvic Soft Drinks Limited.docx`, got `2024-07-16 Notice of variation and consolidation for Beckton Soft Drinks Factory.docx` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-04**: expected `2014-11-17 Announcement of Consultation from Environment Agency.docx`, got `Announcement of Consultation on Draft Plan and Its Accompanying Environmental from Edward Nichols.docx` · trap: party Edward Nichols (project manager who signs) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-05**: expected `2019-10-18 Advice Letter to Home Office.odt`, got `2019-10-18 Letter from Dr Owen Bowden-Jones Prof Roger Knaggs.odt` · trap: party Owen Bowden-Jones (ACMD chair who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-06**: expected `2019-10-24 Response Letter to ONS.odt`, got `2019-10-24 Letter to Alan Manning.odt` · trap: party Alan Manning (committee chair who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-07**: expected `2024-03-28 Response Letter to Welsh Government.odt`, got `2024-03-28 Letter to Secretary of State for Work & Pensions.odt` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real2-office-08**: expected `2026-01-29 Notice of Proposal to Modify Definitive Map Order for North Yorkshire Council.odt`, got `2026-01-29 Notice of Proposal to Modify Definitive Map Order for The Council.odt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-09**: expected `2012-11-29 Cover Letter to National Fire Protection Association, Inc.doc`, got `Letter from Building Standards Commission.doc` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real2-office-10**: expected `2011-07-19 Response Letter to John Mann MP.doc`, got `Document - SIR GUS O'DONNELL KCB.doc` · trap: party Gus O'Donnell (Cabinet Secretary who signs) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- … and 5 more in the JSON report

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
