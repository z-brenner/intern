# InternBench: live run

- **Run:** 2026-11-21T04:57:01Z · commit `a51447d321`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 1.28 GB · sha256 `aaf42c8b7c3c`
- **Corpus:** 51 documents · 51 completed · gold `869c130127`
- **Wall time:** 754.73 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 18/50 (36.0%) |
| Document type | 38/51 (74.5%) |
| Date | 34/50 (68.0%) |
| Date role (when the reviewed date was chosen) | 8/34 (23.5%) |
| Parties | 31/51 (60.8%) |
| Relation word (when parties were named) | 34/48 (70.8%) |
| Party in the right role (when parties were named) | 15/24 (62.5%) |
| Ready / review routing | 3/30 (10.0%) |
| Description states nothing false | 50/51 (98.0%) |
| Description is specific | 33/51 (64.7%) |
| Description specificity | 59.5% (mean of 51) |
| Evidence recall (model's quotes) | 47.1% (mean of 51) |
| Digest recall (facts reaching the digest) | 78.4% (mean of 51) |
| Prompt recall (facts reaching the prompt sent) | 77.5% (mean of 51) |
| Sent to review | 98.0% of 51 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 100.0% | 2 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 82.3% | 51 |
| Review rate | 98.0% | 51 |
| Evidence recall | 47.1% | 51 |
| Total latency p50 | 14.05 s | 51 |
| Total latency p95 | 25.26 s | 51 |
| Generation latency p50 | 6.36 s | 51 |
| Generation latency p95 | 8.20 s | 51 |
| Generated tokens p50 | 96 | 51 |
| Generated tokens p95 | 123 | 51 |
| Prompt tokens p50 | 960 | 51 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 50 |
| Trap date chosen | 4 of 51 |
| Forbidden party named | 8 of 51 |
| Spurious parties named | 17 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 1 of 113 (0.9%) |
| Right name sent to review anyway | 15 of 26 |
| Sent to review for an unsupported fact | 42 of 51 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 2/7 | 2/7 | 2/7 | 0/4 | 100.0% | 12.62 s | 16.90 s |
| letter | 19 | 8/19 | 17/19 | 13/19 | 0/9 | 100.0% | 12.47 s | 24.80 s |
| notice | 19 | 6/18 | 12/18 | 14/19 | 2/12 | 100.0% | 15.27 s | 34.44 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0/1 | 100.0% | 7.16 s | 8.41 s |
| statement | 4 | 2/4 | 3/4 | 2/4 | 1/4 | 75.0% | 16.77 s | 22.60 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 12 | 2/12 | 6/12 | 4/12 | 0/3 | 100.0% | 9.23 s | 16.25 s |
| native | 15 | 8/14 | 12/14 | 12/15 | 1/12 | 100.0% | 15.11 s | 24.53 s |
| office | 11 | 5/11 | 8/11 | 8/11 | 1/7 | 100.0% | 10.68 s | 18.39 s |
| scan | 10 | 2/10 | 5/10 | 6/10 | 1/5 | 100.0% | 18.99 s | 34.44 s |
| text | 3 | 1/3 | 3/3 | 1/3 | 0/3 | 66.7% | 10.37 s | 19.49 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 24 | 7/24 | 14/24 | 12/24 | 2/13 | 95.8% | 9.71 s | 18.32 s |
| 2-4 | 20 | 7/19 | 16/19 | 13/20 | 1/12 | 100.0% | 15.11 s | 25.26 s |
| 5-9 | 5 | 2/5 | 2/5 | 4/5 | 0/3 | 100.0% | 24.53 s | 34.44 s |
| 10-24 | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 100.0% | 18.39 s | 19.77 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 2 | 2/2 | 2/2 | 2/2 | 0/2 | 0 of 2 | 18.39 s | 19.77 s | 4.34 s | 7.86 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_title | 1 | 0/1 | 0/1 | 0/1 | 0 |
| addressed_to_staff | 1 | 0/1 | 1/1 | 0/1 | 1 |
| agency_billing | 4 | 4/4 | 4/4 | 4/4 | 0 |
| aging_table | 2 | 2/2 | 2/2 | 2/2 | 0 |
| all_caps_parties | 2 | 2/2 | 2/2 | 2/2 | 0 |
| ambiguous_digit | 1 | 0/1 | 1/1 | 1/1 | 0 |
| archivist_annotation | 2 | 0/2 | 0/2 | 0/2 | 0 |
| attachment_with_own_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| attachments | 2 | 1/2 | 1/2 | 2/2 | 0 |
| attachments_with_own_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bcc_distribution_list | 1 | 0/1 | 0/1 | 0/1 | 0 |
| billhead | 3 | 0/3 | 0/3 | 0/3 | 0 |
| blank_date | 2 | 0/2 | 0/2 | 0/2 | 1 |
| body_recounts_events | 7 | 3/6 | 4/6 | 6/7 | 3 |
| born_digital | 13 | 8/12 | 11/12 | 11/13 | 4 |
| brand_vs_legal_name | 4 | 4/4 | 4/4 | 4/4 | 0 |
| caption_court_differs_from_header | 1 | 0/1 | 0/1 | 1/1 | 1 |
| catalogue_date_trap | 2 | 0/2 | 0/2 | 0/2 | 0 |
| circular | 1 | 0/1 | 0/1 | 1/1 | 0 |
| coloured_paper | 2 | 0/2 | 0/2 | 0/2 | 1 |
| comparative_period | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 27 | 13/27 | 20/27 | 17/27 | 5 |
| consignee_prominent | 1 | 0/1 | 0/1 | 0/1 | 0 |
| consultation_response | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copies_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| corrected_firm_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| corrected_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| court_filing | 3 | 2/3 | 2/3 | 3/3 | 1 |
| cover_sheet | 2 | 0/2 | 1/2 | 1/2 | 0 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_first | 4 | 0/4 | 0/4 | 0/4 | 1 |
| date_at_end | 6 | 2/6 | 4/6 | 4/6 | 1 |
| date_in_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_missing_from_text_layer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_only_in_certificate | 1 | 0/1 | 0/1 | 1/1 | 1 |
| deadline_date | 4 | 2/4 | 3/4 | 3/4 | 2 |
| deadline_in_body | 6 | 2/5 | 3/5 | 5/6 | 3 |
| dear_colleague | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dear_sir_madam | 1 | 1/1 | 1/1 | 1/1 | 0 |
| decision_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| decision_numbers_look_like_dates | 1 | 0/1 | 1/1 | 0/1 | 1 |
| direction | 1 | 0/1 | 0/1 | 1/1 | 1 |
| docket_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ecf_header_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| eighteenth_century | 1 | 0/1 | 0/1 | 0/1 | 0 |
| enclosures_not_included | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enforcement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| engraved_vignette | 1 | 0/1 | 0/1 | 0/1 | 0 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filing_stamps | 1 | 0/1 | 1/1 | 0/1 | 0 |
| flight_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| foi_response | 2 | 2/2 | 2/2 | 2/2 | 0 |
| footnote_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 1/2 | 1/2 | 0 |
| government_issuer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 0/6 | 0 |
| handwritten_annotation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| header_fields | 1 | 0/1 | 1/1 | 0/1 | 0 |
| hearing_date_in_body | 2 | 0/2 | 0/2 | 2/2 | 1 |
| historic | 7 | 0/7 | 0/7 | 0/7 | 1 |
| historical | 6 | 2/6 | 6/6 | 4/6 | 1 |
| image | 6 | 2/6 | 6/6 | 4/6 | 1 |
| image_only | 2 | 0/2 | 1/2 | 1/2 | 0 |
| image_only_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| improvement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| inside_address_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| invoice | 7 | 2/7 | 2/7 | 2/7 | 1 |
| iso_dates | 1 | 0/1 | 1/1 | 0/1 | 0 |
| jpeg | 6 | 0/6 | 1/6 | 1/6 | 0 |
| junk_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| legacy_doc | 2 | 0/2 | 0/2 | 2/2 | 1 |
| letter | 19 | 8/19 | 17/19 | 13/19 | 5 |
| letter_format | 2 | 0/2 | 2/2 | 1/2 | 1 |
| letterhead | 2 | 0/2 | 2/2 | 0/2 | 2 |
| letterhead_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead_names_people | 1 | 0/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 5 | 4/5 | 4/5 | 4/5 | 0 |
| live_date_field | 1 | 1/1 | 1/1 | 1/1 | 0 |
| local_government | 2 | 0/2 | 1/2 | 1/2 | 1 |
| lockbox | 2 | 2/2 | 2/2 | 2/2 | 0 |
| logo_trap | 1 | 0/1 | 1/1 | 0/1 | 1 |
| long_document | 2 | 2/2 | 2/2 | 2/2 | 0 |
| many_dates | 3 | 3/3 | 3/3 | 3/3 | 0 |
| metadata_date_trap | 5 | 2/5 | 3/5 | 5/5 | 1 |
| metadata_title_trap | 3 | 2/3 | 3/3 | 2/3 | 1 |
| microfilm_targets | 1 | 0/1 | 0/1 | 0/1 | 0 |
| mislabelled_source | 1 | 1/1 | 1/1 | 1/1 | 0 |
| misleading_metadata | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 4/4 | 4/4 | 4/4 | 0 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_recipient | 1 | 0/1 | 1/1 | 1/1 | 0 |
| multiple_parties_side | 2 | 1/2 | 1/2 | 2/2 | 1 |
| multiple_recipients | 1 | 0/1 | 1/1 | 0/1 | 1 |
| multiple_senders | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 9 | 2/9 | 7/9 | 5/9 | 3 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_amounts | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_explicit_title | 2 | 0/2 | 0/2 | 0/2 | 1 |
| no_issuer_in_text | 2 | 0/2 | 1/2 | 0/2 | 0 |
| no_letterhead_on_letter | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_title_word | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 19 | 6/18 | 12/18 | 14/19 | 6 |
| notice_of_appeal | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_decision | 2 | 1/2 | 2/2 | 2/2 | 0 |
| notice_of_determination | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice_of_hearing | 2 | 0/2 | 0/2 | 2/2 | 1 |
| notice_of_removal | 2 | 1/2 | 1/2 | 2/2 | 1 |
| notice_of_violation | 1 | 0/1 | 1/1 | 0/1 | 1 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_layer | 2 | 0/2 | 0/2 | 2/2 | 1 |
| ocr_text_layer | 2 | 0/2 | 1/2 | 1/2 | 1 |
| on_behalf_of_client | 3 | 1/3 | 2/3 | 2/3 | 1 |
| organization_issuer | 1 | 0/1 | 1/1 | 0/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_ends_after_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| pgp_signed | 1 | 0/1 | 1/1 | 0/1 | 0 |
| plain_text | 2 | 1/2 | 2/2 | 1/2 | 0 |
| posted_notice | 3 | 1/3 | 3/3 | 1/3 | 1 |
| print_field_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| printed_form | 3 | 0/3 | 0/3 | 0/3 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_notice | 5 | 1/4 | 3/4 | 3/5 | 3 |
| publication_number_year_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| quaker_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| re_line_references_document | 4 | 1/4 | 3/4 | 3/4 | 0 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| receipted | 3 | 0/3 | 0/3 | 0/3 | 0 |
| redacted_addressee | 2 | 2/2 | 2/2 | 2/2 | 0 |
| redactions | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_numbers | 1 | 0/1 | 1/1 | 0/1 | 1 |
| reference_numbers_look_like_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| relative_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| remit_to | 5 | 4/5 | 4/5 | 4/5 | 1 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| report_parameters | 1 | 1/1 | 1/1 | 1/1 | 0 |
| revenue_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| review | 2 | 0/2 | 1/2 | 0/2 | 0 |
| scanned | 12 | 2/12 | 6/12 | 7/12 | 4 |
| sender_block_above_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| sender_in_signoff_only | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 1/3 | 1/3 | 1/3 | 0 |
| signatory_trap | 3 | 1/3 | 3/3 | 1/3 | 2 |
| signatory_vs_org | 1 | 0/1 | 0/1 | 0/1 | 0 |
| signature_on_later_page | 3 | 1/3 | 2/3 | 2/3 | 1 |
| signature_only_no_typed_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| small_print_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 1 | 0/1 | 0/1 | 0/1 | 0 |
| staff_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamp_date_only | 2 | 0/2 | 0/2 | 2/2 | 1 |
| stamp_dates | 2 | 0/2 | 1/2 | 1/2 | 0 |
| statement | 4 | 2/4 | 3/4 | 2/4 | 0 |
| struck_through | 1 | 0/1 | 0/1 | 0/1 | 0 |
| subject_is_product | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tables | 1 | 1/1 | 1/1 | 1/1 | 0 |
| third_party_named_in_subject | 1 | 0/1 | 1/1 | 0/1 | 1 |
| tiff | 4 | 2/4 | 3/4 | 3/4 | 0 |
| trading_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tribunal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| trustee_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 3 | 2/3 | 2/3 | 2/3 | 1 |
| two_letterhead_seals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| typist_footer_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| typo_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| uk | 5 | 1/5 | 4/5 | 3/5 | 1 |
| undated | 1 | – | – | 1/1 | 1 |
| us_state | 5 | 0/4 | 2/4 | 3/5 | 3 |
| utility_commission | 1 | 0/1 | 1/1 | 0/1 | 1 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wide_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| windows-1252 | 1 | 0/1 | 1/1 | 0/1 | 0 |
| windows_1252_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| yearless_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| zero_amount | 2 | 2/2 | 2/2 | 2/2 | 0 |

## Routes

- **Pages:** fast 64 · layout 12 · ocr 44
- **Documents by route class:** fast 23 · layout 5 · ocr 23

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 51 | 14.05 s | 25.26 s | 34.44 s |
| `extraction_wall_ms` | 51 | 20.9 ms | 5.91 s | 15.95 s |
| `worker_total_ms` | 51 | 19.3 ms | 5.91 s | 15.95 s |
| `worker_snapshot_ms` | 51 | 0.21 ms | 2.5 ms | 8.8 ms |
| `worker_parse_ms` | 51 | 3.5 ms | 21.6 ms | 1.31 s |
| `worker_analysis_ms` | 51 | 0.18 ms | 1.6 ms | 5.2 ms |
| `worker_render_ms` | 51 | 0.00 ms | 496.1 ms | 2.01 s |
| `worker_image_decode_ms` | 51 | 0.00 ms | 88.7 ms | 1.00 s |
| `worker_ocr_ms` | 51 | 0.00 ms | 10.36 s | 31.18 s |
| `worker_ocr_encode_ms` | 51 | 0.00 ms | 748.4 ms | 2.33 s |
| `worker_ocr_engine_ms` | 51 | 0.00 ms | 9.37 s | 28.60 s |
| `worker_vision_ms` | 51 | 0.00 ms | 12.3 ms | 27.9 ms |
| `analyze_wall_ms` | 51 | 11.89 s | 22.59 s | 24.52 s |
| `distill_ms` | 51 | 1.9 ms | 12.4 ms | 24.3 ms |
| `prompt_ms` | 51 | 0.03 ms | 0.05 ms | 0.07 ms |
| `inference_ms` | 51 | 11.88 s | 22.59 s | 24.50 s |
| `prefill_ms` | 51 | 5.93 s | 16.46 s | 17.70 s |
| `generation_ms` | 51 | 6.36 s | 8.20 s | 8.87 s |
| `validation_ms` | 51 | 4.5 ms | 15.2 ms | 19.5 ms |
| `naming_ms` | 51 | 0.19 ms | 1.5 ms | 2.8 ms |
| `index_ms` | 51 | 1.7 ms | 10.9 ms | 21.6 ms |
| `retrieval_ms` | 51 | 0.03 ms | 1.2 ms | 3.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 51 | 159.0 tok/s | 162.1 tok/s | 162.7 tok/s |
| `generation_tok_per_s` | 51 | 15.2 tok/s | 15.7 tok/s | 15.8 tok/s |
| `prompt_tokens` | 51 | 960 | 2586 | 2778 |
| `cached_tokens` | 51 | 477 | 477 | 477 |
| `generated_tokens` | 51 | 96 | 123 | 137 |
| `estimated_prompt_tokens` | 51 | 1647 | 3006 | 3315 |
| `model_requests` | 51 | 1 | 1 | 1 |
| `redistillations` | 51 | 0 | 0 | 0 |
| `source_characters` | 51 | 2713 | 22421 | 45033 |
| `digest_characters` | 51 | 2946 | 7884 | 8158 |
| `prompt_characters` | 51 | 4995 | 9933 | 10207 |
| `worker_ocr_pages` | 51 | 0 | 4 | 8 |
| `worker_ocr_passes` | 51 | 0 | 4 | 9 |
| `worker_orientation_passes` | 51 | 0 | 4 | 8 |
| `worker_rendered_pixels` | 51 | 0 | 33660000 | 67578072 |
| `index_units` | 51 | 28 | 137 | 464 |
| `context_units` | 51 | 24 | 74 | 112 |
| `context_tokens` | 51 | 1059 | 2419 | 2727 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 23 | 3.2 ms | 20.9 ms | 2.9 ms | 19.3 ms | 19.0 ms | 0.88 ms | 0.00 ms | 0.00 ms |
| layout | 5 | 6.3 ms | 38.1 ms | 5.8 ms | 36.7 ms | 30.1 ms | 5.2 ms | 0.00 ms | 0.00 ms |
| ocr | 23 | 1.86 s | 6.10 s | 1.86 s | 6.10 s | 15.6 ms | 0.90 ms | 1.13 s | 11.25 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 24 | 515.2 ms | 3.69 s | 514.5 ms | 3.69 s | 14.8 ms | 1.1 ms | 101.2 ms | 3.05 s |
| 2-4 | 20 | 5.3 ms | 4.77 s | 4.7 ms | 4.77 s | 21.6 ms | 1.1 ms | 468.2 ms | 9.04 s |
| 5-9 | 5 | 17.6 ms | 15.95 s | 16.8 ms | 15.95 s | 14.5 ms | 1.6 ms | 2.01 s | 31.18 s |
| 10-24 | 2 | 20.9 ms | 38.1 ms | 19.3 ms | 36.7 ms | 30.1 ms | 5.2 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 1.38 s | 2.23 s | 1.37 s | 2.23 s | 13.9 ms | 1.9 ms | 131.6 ms | 2.21 s |
| letter | 19 | 20.9 ms | 5.91 s | 19.3 ms | 5.91 s | 1.31 s | 1.6 ms | 1.13 s | 10.36 s |
| notice | 19 | 8.6 ms | 15.95 s | 8.2 ms | 15.95 s | 30.1 ms | 5.2 ms | 2.01 s | 31.18 s |
| receipt | 2 | 515.2 ms | 1.67 s | 514.5 ms | 1.67 s | 9.3 ms | 0.07 ms | 0.00 ms | 1.64 s |
| statement | 4 | 2.0 ms | 2.15 s | 1.4 ms | 2.15 s | 3.2 ms | 1.1 ms | 101.2 ms | 2.03 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 220 MB | 1277 MB | real3-notice-02 |
| Model server | 1578 MB | 1579 MB | real3-office-15 |
| Benchmark (engine) | 21 MB | 21 MB | real3-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (32)

- **real3-letter-02**: expected `2011-02-18 Reminder Letter to Panther Motors, Inc.pdf`, got `2011-02-18 Notice to Dominick Livoti.pdf` · trap: party Dominick Livoti (addressee, President of Panther Motors) · review: TYPE_UNSUPPORTED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-06**: expected `1976-11-18 Letter to Los Alamos Scientific Laboratory.pdf`, got `1976-11-18 Document - Pacific Northwest Laboratories.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-07**: expected `1966-01-04 Letter to U.S. Atomic Energy Commission.pdf`, got `Document from D. P. Gramquiet.pdf` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-08**: expected `1995-12-07 Decision Letter to Bloom & Kreten.pdf`, got `1995-12-07 Register of Copyrights for Bernhard Kreten.pdf` · trap: party Bernhard Kreten (addressee, attorney at Bloom & Kreten) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-09**: expected `1992-12-24 Objection Letter to Pilcher and Pilcher.pdf`, got `Letter to Pilcher and Pilcher.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-10**: expected `2019-03-28 Response Letter to Transport Scotland.pdf`, got `2019-03-28 Letter - John Larkinson.pdf` · trap: party John Larkinson (ORR chief executive who signs (named above the addressee)) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-12**: expected `1907-12-18 Comment Letter to Office of the Chief Signal Officer.jpg`, got `1907-12-18 Letter to Wright Brothers.jpg` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-13**: expected `1908-02-11 Transmittal Letter to Wright Brothers.tif`, got `1908-02-11 Contract with Wright Brothers.tif` · review: PARTY_MISSING
- **real3-notice-01**: expected `2024-02-19 Improvement Notice for The Chiltern Railway Company Limited.pdf`, got `Improvement Notice for The Chiltern Railway Company Limited.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-02**: expected `2007-04-24 Notice of Public Site Visit and Informational Hearing for City of Victorville.pdf`, got `Notice Of Public Site Visit and Informational Hearing for City of Victorville.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-04**: expected `2006-02-23 Notice of Determination from City of Industry.pdf`, got `2006-02-23 Notice of Determination for County Clerk County of Los Angeles.pdf` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-05**: expected `2008-09-22 Notice of Hearing from Mojave Desert Air Quality Management District.pdf`, got `2008-10-27 Notice of Hearing from Mojave Desert Air Quality Management District.pdf` · trap: date 2008-10-27 (date of the public hearing) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-07**: expected `2023-04-07 Notice of Violation for PacifiCorp.pdf`, got `2023-04-07 Notice of Violation for Shelley McCoy.pdf` · trap: party Shelley McCoy (PacifiCorp's Director, Regulation, to whom the letter is addressed) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-08**: expected `2018-01-26 Notice of Removal between Olymbec USA LLC and CWFS Insight LLC.pdf`, got `2018-01-22 Notice of Removal between Olymbec USA LLC and CWFS Insight LLC.pdf` · trap: date 2018-01-22 (date of the Second Verified Complaint and its receipt) · review: LOW_CONFIDENCE
- **real3-notice-11**: expected `2023-11-13 Revocation of Product Approval for Atlantis Hydrotec.pdf`, got `2023-11-13 Letter to Water Company.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-notice-12**: expected `1884-12-29 Notice for William Dew.jpg`, got `1884-12-29 Animals Order from R. W. Merriman.jpg` · trap: party R. W. Merriman (Clerk to the Local Authority who signs 'by order') · review: DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-13**: expected `1831-11-01 Notice from Parish of Clerkenwell.jpg`, got `1831-11-01 Document from The Board of Health.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-15**: expected `2026-10-14 Errata Notice from The FreeBSD Project.txt`, got `2026-10-14 Errata Notice.txt` · review: LOW_CONFIDENCE
- **real3-invoice-09**: expected `1916-11-04 Invoice from Mergenthaler Linotype Co.pdf`, got `Invoice from Panama City PUB Co.pdf` · trap: party Panama City Pub Co (customer, typed above the issuer) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-10**: expected `1868-10-01 Bill from Haviland & Sutton.tif`, got `Document from M Covace dby.tif` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-11**: expected `1850-07-11 Bill from Henry D. Gray & Son.jpg`, got `Document - 23 & 24 India Street.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-12**: expected `1848-11-14 Bill from Veranda Hotel.jpg`, got `Document from Mr 29. Wonior.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-13**: expected `1776-07-15 Invoice from Clark & Nightingale.jpg`, got `Document from J Orovidenco.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-14**: expected `1817-06-17 Receipt from Gales & Seaton.jpg`, got `Document from Hon Shomas Jefferson 1817.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-15**: expected `1848-07-01 Receipt from Post-Office, Duxbury, Mass.jpg`, got `Document - Frensint Aefoers.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real3-office-01**: expected `2026-09-14 Consultation Response to London Borough of Islington.docx`, got `2026-09-14 Notice from London Borough of Islington.docx` · review: TYPE_UNSUPPORTED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-02**: expected `2018-11-21 Remit Letter to NHS Pay Review Body.docx`, got `2018-11-21 Letter to Office of Manpower Economics.docx` · trap: party Office of Manpower Economics (secretariat whose address is used) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-03**: expected `2016-03-23 Letter of Concern to Central Bedfordshire Council.docx`, got `2016-03-23 Letter - Andrew Cook.docx` · trap: party Andrew Cook (HMI Regional Director who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-05**: expected `2011-10-01 Direction for Ofcom.doc`, got `2012-04-01 Direction - Secretary of State for Business, Innovation and Skills.doc` · trap: date 2012-04-01 (deadline for Ofcom to impose the condition) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-06**: expected `2014-02-20 Circular from Home Office.doc`, got `Circular - Home Secretary.doc` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-12**: expected `2012-09-30 Statement of Financial Position from Province of British Columbia.xlsx`, got `Statement of Financial Position.xlsx` · review: DATE_UNSUPPORTED, DESCRIPTION_INVALID
- **real3-office-14**: expected `2017-03-31 Condensed Statement of Financial Position from Administrative Tribunals Support Service of Canada.csv`, got `2017-03-31 Condensed Statement of Financial Position unaudited.csv` · **filed without review**

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
