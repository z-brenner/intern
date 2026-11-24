# InternBench: live run

- **Run:** 2026-11-23T16:49:54Z · commit `70ce2164a1`
- **Machine:** INTEL(R) XEON(R) PLATINUM 8573C, 4 logical cores, 15.6 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 55 documents · 55 completed · gold `7e77820964`
- **Wall time:** 3213.72 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 34/55 (61.8%) |
| Document type | 51/55 (92.7%) |
| Date | 52/55 (94.5%) |
| Date role (when the reviewed date was chosen) | 23/52 (44.2%) |
| Parties | 43/55 (78.2%) |
| Relation word (when parties were named) | 48/54 (88.9%) |
| Party in the right role (when parties were named) | 23/37 (62.2%) |
| Ready / review routing | 5/34 (14.7%) |
| Description states nothing false | 55/55 (100.0%) |
| Description is specific | 43/55 (78.2%) |
| Description specificity | 71.5% (mean of 55) |
| Evidence recall (model's quotes) | 70.3% (mean of 55) |
| Digest recall (facts reaching the digest) | 95.3% (mean of 55) |
| Prompt recall (facts reaching the prompt sent) | 95.3% (mean of 55) |
| Sent to review | 89.1% of 55 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 66.7% | 3 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 70.9% | 55 |
| Review rate | 89.1% | 55 |
| Evidence recall | 70.3% | 55 |
| Total latency p50 | 52.12 s | 55 |
| Total latency p95 | 102.87 s | 55 |
| Generation latency p50 | 24.34 s | 55 |
| Generation latency p95 | 31.55 s | 55 |
| Generated tokens p50 | 94 | 55 |
| Generated tokens p95 | 117 | 55 |
| Prompt tokens p50 | 868 | 55 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 55 |
| Trap date chosen | 0 of 55 |
| Forbidden party named | 9 of 55 |
| Spurious parties named | 10 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 148 (0.0%) |
| Right name sent to review anyway | 19 of 34 |
| Sent to review for an unsupported fact | 39 of 55 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 10 | 6/10 | 8/10 | 7/10 | 1/7 | 80.0% | 43.90 s | 69.17 s |
| letter | 25 | 15/25 | 24/25 | 18/25 | 0/13 | 100.0% | 52.12 s | 102.87 s |
| notice | 19 | 13/19 | 19/19 | 17/19 | 4/14 | 79.0% | 67.49 s | 103.13 s |
| statement | 1 | 0/1 | 1/1 | 1/1 | – | 100.0% | 35.49 s | 35.49 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 8 | 5/8 | 7/8 | 7/8 | 0/5 | 100.0% | 43.90 s | 69.17 s |
| native | 16 | 11/16 | 15/16 | 14/16 | 3/11 | 81.2% | 54.79 s | 94.50 s |
| office | 16 | 8/16 | 16/16 | 10/16 | 1/8 | 93.8% | 50.08 s | 103.13 s |
| scan | 13 | 9/13 | 12/13 | 11/13 | 0/8 | 92.3% | 69.57 s | 121.29 s |
| text | 2 | 1/2 | 2/2 | 1/2 | 1/2 | 50.0% | 52.93 s | 67.49 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 14/23 | 21/23 | 17/23 | 1/13 | 91.3% | 43.68 s | 52.93 s |
| 2-4 | 24 | 13/24 | 23/24 | 18/24 | 4/16 | 83.3% | 55.64 s | 94.50 s |
| 5-9 | 5 | 5/5 | 5/5 | 5/5 | 0/3 | 100.0% | 80.14 s | 96.03 s |
| 10-24 | 2 | 1/2 | 2/2 | 2/2 | 0/1 | 100.0% | 80.43 s | 121.29 s |
| 25-49 | 1 | 1/1 | 1/1 | 1/1 | 0/1 | 100.0% | 103.13 s | 103.13 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 3 | 2/3 | 3/3 | 3/3 | 0/2 | 0 of 3 | 103.13 s | 121.29 s | 27.98 s | 32.79 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| address_of_host_institution | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_outside_counsel | 1 | 1/1 | 1/1 | 1/1 | 0 |
| addressed_to_person_at_body | 1 | 0/1 | 1/1 | 0/1 | 1 |
| addressed_to_person_at_company | 8 | 5/8 | 7/8 | 7/8 | 1 |
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
| bank_details_redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bill_to_prominent | 1 | 1/1 | 1/1 | 1/1 | 0 |
| billhead | 3 | 0/3 | 1/3 | 1/3 | 1 |
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
| competing_dates | 20 | 11/20 | 19/20 | 17/20 | 2 |
| copy_list | 1 | 1/1 | 1/1 | 1/1 | 0 |
| corrected_invoice | 1 | 0/1 | 1/1 | 0/1 | 1 |
| court_filing | 6 | 4/6 | 6/6 | 5/6 | 0 |
| court_letter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| customer_first | 1 | 0/1 | 1/1 | 0/1 | 1 |
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
| enforcement | 3 | 3/3 | 3/3 | 3/3 | 0 |
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
| handwritten | 3 | 0/3 | 1/3 | 1/3 | 0 |
| handwritten_annotation | 5 | 3/5 | 5/5 | 4/5 | 1 |
| handwritten_annotations | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten_salutation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| historic | 3 | 0/3 | 1/3 | 1/3 | 1 |
| historical | 1 | 1/1 | 1/1 | 1/1 | 0 |
| image | 2 | 2/2 | 2/2 | 2/2 | 0 |
| in_re_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| inline_footnotes | 1 | 1/1 | 1/1 | 1/1 | 0 |
| invoice | 10 | 6/10 | 8/10 | 7/10 | 2 |
| issuer_not_named | 1 | 1/1 | 1/1 | 1/1 | 0 |
| jpeg | 1 | 0/1 | 0/1 | 1/1 | 0 |
| letter | 25 | 15/25 | 24/25 | 18/25 | 7 |
| letter_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letter_style | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead_image_only | 2 | 1/2 | 2/2 | 2/2 | 0 |
| letterhead_mismatch | 1 | 0/1 | 1/1 | 0/1 | 1 |
| letterhead_officers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 3 | 1/3 | 2/3 | 3/3 | 0 |
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
| name_variant | 1 | 1/1 | 1/1 | 1/1 | 0 |
| native | 7 | 5/7 | 7/7 | 7/7 | 0 |
| native_pdf | 6 | 4/6 | 6/6 | 5/6 | 1 |
| no_dated_letterhead | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_dated_line | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_explicit_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_letterhead_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_ocr_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_subject_line | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_text_layer | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notary_stamp | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 19 | 13/19 | 19/19 | 17/19 | 0 |
| notice_of_meeting | 1 | 0/1 | 1/1 | 1/1 | 0 |
| number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ocr_text_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| odt | 4 | 2/4 | 4/4 | 3/4 | 1 |
| office | 12 | 5/12 | 12/12 | 7/12 | 5 |
| ordinal_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| paid_stamp | 2 | 0/2 | 1/2 | 1/2 | 1 |
| past_due_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| person_addressee | 2 | 2/2 | 2/2 | 2/2 | 0 |
| person_party | 2 | 0/2 | 2/2 | 0/2 | 2 |
| pleading_paper_line_numbers | 1 | 0/1 | 1/1 | 1/1 | 0 |
| product_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| public_records_release | 3 | 2/3 | 3/3 | 2/3 | 1 |
| re_line_other_matter | 2 | 2/2 | 2/2 | 2/2 | 0 |
| received_stamp | 2 | 1/2 | 2/2 | 1/2 | 1 |
| redacted | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redactions | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_case | 1 | 0/1 | 1/1 | 1/1 | 0 |
| referenced_dates | 7 | 6/7 | 7/7 | 7/7 | 0 |
| referenced_document | 1 | 1/1 | 1/1 | 1/1 | 0 |
| regulator_correspondence | 4 | 2/4 | 3/4 | 3/4 | 1 |
| released_date | 2 | 2/2 | 2/2 | 2/2 | 0 |
| remit_to | 2 | 1/2 | 2/2 | 2/2 | 0 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| scanned | 20 | 13/20 | 18/20 | 16/20 | 2 |
| schedules_attached | 1 | 1/1 | 1/1 | 1/1 | 0 |
| screen_printout | 1 | 1/1 | 1/1 | 1/1 | 0 |
| serial_number_looks_like_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| service_period | 1 | 1/1 | 1/1 | 1/1 | 0 |
| settling_party_differs_from_caption | 2 | 2/2 | 2/2 | 2/2 | 0 |
| signatory_vs_org | 1 | 1/1 | 1/1 | 1/1 | 0 |
| signature_page_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| sister_company_named | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sole_proprietor | 1 | 0/1 | 0/1 | 0/1 | 0 |
| source_title_mismatch | 1 | 1/1 | 1/1 | 1/1 | 0 |
| staff_names | 6 | 3/6 | 5/6 | 5/6 | 1 |
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

- **Pages:** fast 64 · layout 12 · ocr 66 · ocr_regions 1
- **Documents by route class:** fast 24 · layout 8 · ocr 22 · ocr_regions 1

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 55 | 52.12 s | 102.87 s | 121.29 s |
| `extraction_wall_ms` | 55 | 13.5 ms | 19.61 s | 53.59 s |
| `worker_total_ms` | 55 | 12.7 ms | 19.61 s | 53.59 s |
| `worker_snapshot_ms` | 55 | 0.23 ms | 1.3 ms | 3.7 ms |
| `worker_parse_ms` | 55 | 2.5 ms | 17.9 ms | 140.3 ms |
| `worker_analysis_ms` | 55 | 0.29 ms | 2.1 ms | 16.5 ms |
| `worker_render_ms` | 55 | 0.00 ms | 1.12 s | 7.51 s |
| `worker_image_decode_ms` | 55 | 0.00 ms | 100.4 ms | 112.4 ms |
| `worker_ocr_ms` | 55 | 0.00 ms | 38.40 s | 105.46 s |
| `worker_ocr_encode_ms` | 55 | 0.00 ms | 2.55 s | 7.32 s |
| `worker_ocr_engine_ms` | 55 | 0.00 ms | 35.04 s | 97.81 s |
| `worker_vision_ms` | 55 | 0.00 ms | 11.9 ms | 16.9 ms |
| `analyze_wall_ms` | 55 | 51.58 s | 92.31 s | 103.11 s |
| `distill_ms` | 55 | 2.1 ms | 26.5 ms | 49.7 ms |
| `prompt_ms` | 55 | 0.04 ms | 0.06 ms | 0.08 ms |
| `inference_ms` | 55 | 51.57 s | 92.29 s | 103.04 s |
| `prefill_ms` | 55 | 26.02 s | 62.15 s | 78.60 s |
| `generation_ms` | 55 | 24.34 s | 31.55 s | 35.78 s |
| `validation_ms` | 55 | 6.5 ms | 28.6 ms | 35.5 ms |
| `naming_ms` | 55 | 0.27 ms | 2.7 ms | 5.8 ms |
| `index_ms` | 55 | 2.0 ms | 24.5 ms | 47.9 ms |
| `retrieval_ms` | 55 | 0.04 ms | 2.0 ms | 4.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 55 | 32.9 tok/s | 33.5 tok/s | 33.5 tok/s |
| `generation_tok_per_s` | 55 | 3.8 tok/s | 3.9 tok/s | 3.9 tok/s |
| `prompt_tokens` | 55 | 868 | 2053 | 2593 |
| `cached_tokens` | 55 | 477 | 477 | 477 |
| `generated_tokens` | 55 | 94 | 117 | 136 |
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
| `index_units` | 55 | 27 | 255 | 397 |
| `context_units` | 55 | 24 | 51 | 100 |
| `context_tokens` | 55 | 925 | 2111 | 2744 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 24 | 2.6 ms | 13.5 ms | 2.2 ms | 12.9 ms | 11.1 ms | 0.72 ms | 0.00 ms | 0.00 ms |
| layout | 8 | 6.2 ms | 161.3 ms | 5.5 ms | 159.0 ms | 140.3 ms | 16.5 ms | 0.00 ms | 0.00 ms |
| ocr_regions | 1 | 416.3 ms | 416.3 ms | 415.6 ms | 415.6 ms | 9.4 ms | 1.5 ms | 59.5 ms | 334.6 ms |
| ocr | 22 | 2.69 s | 20.87 s | 2.69 s | 20.87 s | 13.8 ms | 1.1 ms | 1.20 s | 38.40 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 23 | 1.55 s | 3.80 s | 1.55 s | 3.80 s | 13.8 ms | 0.46 ms | 155.3 ms | 3.55 s |
| 2-4 | 24 | 5.6 ms | 8.39 s | 5.2 ms | 8.39 s | 9.4 ms | 1.5 ms | 608.7 ms | 15.32 s |
| 5-9 | 5 | 12.84 s | 20.87 s | 12.84 s | 20.87 s | 122.4 ms | 1.6 ms | 1.20 s | 38.40 s |
| 10-24 | 2 | 161.3 ms | 53.59 s | 159.0 ms | 53.59 s | 140.3 ms | 16.5 ms | 7.51 s | 105.46 s |
| 25-49 | 1 | 18.0 ms | 18.0 ms | 16.3 ms | 16.3 ms | 15.9 ms | 0.00 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 10 | 1.78 s | 3.82 s | 1.78 s | 3.82 s | 17.9 ms | 4.5 ms | 155.3 ms | 3.74 s |
| letter | 25 | 9.3 ms | 10.56 s | 8.7 ms | 10.55 s | 12.1 ms | 1.6 ms | 810.3 ms | 19.25 s |
| notice | 19 | 5.7 ms | 20.87 s | 5.4 ms | 20.87 s | 140.3 ms | 16.5 ms | 1.20 s | 38.40 s |
| statement | 1 | 1.53 s | 1.53 s | 1.53 s | 1.53 s | 11.7 ms | 0.29 ms | 0.00 ms | 1.47 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 574 MB | 1401 MB | real2-letter-06 |
| Model server | 6085 MB | 6085 MB | real2-office-15 |
| Benchmark (engine) | 16 MB | 20 MB | real2-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (21)

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
