# InternBench: live run

- **Run:** 2026-11-23T16:59:16Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 5.68 GB · sha256 `03b74727a860`
- **Corpus:** 54 documents · 54 completed · gold `0928b994d6`
- **Wall time:** 2964.50 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 25/53 (47.2%) |
| Document type | 43/54 (79.6%) |
| Date | 40/53 (75.5%) |
| Date role (when the reviewed date was chosen) | 16/40 (40.0%) |
| Parties | 38/54 (70.4%) |
| Relation word (when parties were named) | 40/47 (85.1%) |
| Party in the right role (when parties were named) | 23/32 (71.9%) |
| Ready / review routing | 6/31 (19.4%) |
| Description states nothing false | 54/54 (100.0%) |
| Description is specific | 40/54 (74.1%) |
| Description specificity | 63.0% (mean of 54) |
| Evidence recall (model's quotes) | 64.5% (mean of 54) |
| Digest recall (facts reaching the digest) | 78.7% (mean of 54) |
| Prompt recall (facts reaching the prompt sent) | 75.9% (mean of 54) |
| Sent to review | 96.3% of 54 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 2 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 79.6% | 54 |
| Review rate | 96.3% | 54 |
| Evidence recall | 64.5% | 54 |
| Total latency p50 | 49.87 s | 54 |
| Total latency p95 | 96.65 s | 54 |
| Generation latency p50 | 22.83 s | 54 |
| Generation latency p95 | 27.96 s | 54 |
| Generated tokens p50 | 91 | 54 |
| Generated tokens p95 | 117 | 54 |
| Prompt tokens p50 | 960 | 54 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 0 of 53 |
| Trap date chosen | 3 of 54 |
| Forbidden party named | 7 of 54 |
| Spurious parties named | 9 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 120 (0.0%) |
| Right name sent to review anyway | 13 of 26 |
| Sent to review for an unsupported fact | 43 of 54 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 2/7 | 2/7 | 2/7 | 1/4 | 85.7% | 41.88 s | 70.27 s |
| letter | 22 | 9/22 | 19/22 | 16/22 | 1/10 | 100.0% | 49.67 s | 77.90 s |
| notice | 19 | 13/18 | 16/18 | 18/19 | 2/12 | 94.7% | 58.42 s | 96.81 s |
| receipt | 2 | 0/2 | 0/2 | 1/2 | 0/1 | 100.0% | 12.85 s | 31.23 s |
| statement | 4 | 1/4 | 3/4 | 1/4 | 2/4 | 100.0% | 61.46 s | 99.32 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 12 | 3/12 | 6/12 | 6/12 | 0/3 | 100.0% | 31.23 s | 58.42 s |
| native | 15 | 10/14 | 13/14 | 14/15 | 2/12 | 93.3% | 59.50 s | 99.32 s |
| office | 14 | 8/14 | 12/14 | 10/14 | 2/8 | 100.0% | 46.89 s | 82.62 s |
| scan | 10 | 3/10 | 6/10 | 6/10 | 1/5 | 90.0% | 61.46 s | 96.81 s |
| text | 3 | 1/3 | 3/3 | 2/3 | 1/3 | 100.0% | 34.49 s | 87.76 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 26 | 10/26 | 17/26 | 15/26 | 4/13 | 96.2% | 36.17 s | 82.62 s |
| 2-4 | 20 | 12/19 | 17/19 | 17/20 | 1/12 | 95.0% | 58.40 s | 96.81 s |
| 5-9 | 6 | 2/6 | 4/6 | 5/6 | 1/4 | 100.0% | 69.63 s | 96.65 s |
| 10-24 | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 78.15 s | 78.42 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 0 of 2 | 78.15 s | 78.42 s | 18.48 s | 27.49 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_title | 1 | 0/1 | 0/1 | 0/1 | 0 |
| addressed_to_staff | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_billing | 4 | 3/4 | 4/4 | 3/4 | 1 |
| aging_table | 2 | 1/2 | 2/2 | 1/2 | 1 |
| all_caps_parties | 2 | 2/2 | 2/2 | 2/2 | 0 |
| ambiguous_digit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| archivist_annotation | 2 | 0/2 | 0/2 | 0/2 | 0 |
| attachment_with_own_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| attachments | 2 | 1/2 | 2/2 | 2/2 | 0 |
| attachments_with_own_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bcc_distribution_list | 1 | 0/1 | 0/1 | 0/1 | 1 |
| billhead | 3 | 0/3 | 0/3 | 0/3 | 0 |
| blank_date | 2 | 0/2 | 0/2 | 1/2 | 1 |
| body_recounts_events | 7 | 6/6 | 6/6 | 7/7 | 1 |
| born_digital | 13 | 10/12 | 12/12 | 13/13 | 1 |
| brand_vs_legal_name | 4 | 3/4 | 4/4 | 3/4 | 1 |
| caption_court_differs_from_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 2 | 0/2 | 0/2 | 0/2 | 0 |
| circular | 1 | 1/1 | 1/1 | 1/1 | 0 |
| client_in_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| coloured_paper | 2 | 0/2 | 0/2 | 0/2 | 1 |
| comparative_period | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 29 | 13/29 | 23/29 | 20/29 | 6 |
| consignee_prominent | 1 | 0/1 | 0/1 | 0/1 | 0 |
| consultation_response | 1 | 0/1 | 1/1 | 1/1 | 0 |
| copies_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| corrected_firm_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| corrected_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| court_filing | 3 | 3/3 | 3/3 | 3/3 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 1/2 | 1 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_first | 4 | 0/4 | 0/4 | 1/4 | 1 |
| date_at_end | 6 | 3/6 | 5/6 | 5/6 | 0 |
| date_in_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_missing_from_text_layer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| date_only_in_certificate | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_date | 4 | 3/4 | 4/4 | 3/4 | 1 |
| deadline_in_body | 6 | 2/5 | 3/5 | 6/6 | 2 |
| dear_colleague | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dear_sir_madam | 1 | 0/1 | 1/1 | 1/1 | 0 |
| decision_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| decision_numbers_look_like_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| direction | 1 | 1/1 | 1/1 | 1/1 | 0 |
| docket_stamp | 1 | 1/1 | 1/1 | 1/1 | 0 |
| ecf_header_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 0/1 | 1/1 | 1/1 | 0 |
| eighteenth_century | 1 | 0/1 | 0/1 | 0/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| email_thread | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enclosures_not_included | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enforcement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| engraved_vignette | 1 | 0/1 | 0/1 | 0/1 | 0 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filing_stamps | 1 | 1/1 | 1/1 | 1/1 | 0 |
| flight_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| foi_response | 2 | 2/2 | 2/2 | 2/2 | 0 |
| footnote_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 1/2 | 1/2 | 2/2 | 0 |
| government_issuer | 1 | 0/1 | 0/1 | 1/1 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_annotation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| header_fields | 1 | 0/1 | 1/1 | 1/1 | 0 |
| hearing_date_in_body | 2 | 1/2 | 1/2 | 2/2 | 1 |
| hidden_sheets | 1 | 0/1 | 0/1 | 1/1 | 1 |
| historic | 7 | 0/7 | 0/7 | 1/7 | 1 |
| historical | 6 | 3/6 | 6/6 | 5/6 | 0 |
| image | 6 | 3/6 | 6/6 | 5/6 | 0 |
| image_only | 2 | 2/2 | 2/2 | 2/2 | 0 |
| image_only_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| improvement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| inside_address_at_end | 1 | 0/1 | 1/1 | 0/1 | 1 |
| invoice | 7 | 2/7 | 2/7 | 2/7 | 1 |
| iso_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 6 | 1/6 | 1/6 | 2/6 | 0 |
| junk_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| large_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| legacy_doc | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letter | 22 | 9/22 | 19/22 | 16/22 | 6 |
| letter_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letterhead | 2 | 1/2 | 2/2 | 1/2 | 1 |
| letterhead_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| letterhead_names_people | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 5 | 3/5 | 4/5 | 3/5 | 1 |
| live_date_field | 1 | 1/1 | 1/1 | 1/1 | 0 |
| local_government | 2 | 1/2 | 1/2 | 2/2 | 1 |
| lockbox | 2 | 2/2 | 2/2 | 2/2 | 0 |
| logo_trap | 1 | 1/1 | 1/1 | 1/1 | 0 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 0 |
| many_dates | 3 | 2/3 | 3/3 | 2/3 | 1 |
| memo | 1 | 0/1 | 0/1 | 1/1 | 1 |
| metadata_date_trap | 5 | 3/5 | 5/5 | 4/5 | 0 |
| metadata_title_trap | 3 | 2/3 | 3/3 | 2/3 | 1 |
| microfilm_targets | 1 | 0/1 | 0/1 | 0/1 | 1 |
| mislabelled_source | 1 | 1/1 | 1/1 | 1/1 | 0 |
| misleading_metadata | 1 | 0/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 3/4 | 4/4 | 3/4 | 1 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 1 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_round | 1 | 0/1 | 0/1 | 1/1 | 1 |
| multiple_parties_side | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multiple_recipients | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_senders | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signatories | 1 | 0/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 9 | 3/9 | 7/9 | 6/9 | 3 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_amounts | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_explicit_title | 2 | 0/2 | 0/2 | 0/2 | 1 |
| no_issuer_in_text | 2 | 0/2 | 1/2 | 0/2 | 0 |
| no_letterhead_on_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_title | 1 | 0/1 | 1/1 | 1/1 | 0 |
| no_title_word | 1 | 0/1 | 1/1 | 0/1 | 0 |
| notice | 19 | 13/18 | 16/18 | 18/19 | 2 |
| notice_of_appeal | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_decision | 2 | 2/2 | 2/2 | 2/2 | 0 |
| notice_of_determination | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_hearing | 2 | 1/2 | 1/2 | 2/2 | 1 |
| notice_of_removal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_layer | 2 | 0/2 | 0/2 | 2/2 | 1 |
| ocr_text_layer | 2 | 0/2 | 1/2 | 1/2 | 1 |
| on_behalf_of_client | 3 | 0/3 | 2/3 | 2/3 | 1 |
| organization_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_ends_after_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| pgp_signed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| plain_text | 2 | 1/2 | 2/2 | 2/2 | 0 |
| posted_notice | 3 | 1/3 | 3/3 | 2/3 | 0 |
| print_field_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| printed_form | 3 | 0/3 | 0/3 | 1/3 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_notice | 5 | 1/4 | 3/4 | 4/5 | 2 |
| publication_number_year_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| quaker_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| quoted_third_party | 1 | 0/1 | 1/1 | 0/1 | 1 |
| re_line_references_document | 4 | 0/4 | 3/4 | 3/4 | 1 |
| receipt | 2 | 0/2 | 0/2 | 1/2 | 0 |
| receipted | 3 | 0/3 | 0/3 | 1/3 | 0 |
| recipient_by_domain_only | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted_addressee | 2 | 2/2 | 2/2 | 2/2 | 0 |
| redactions | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_numbers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| reference_numbers_look_like_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_letter | 1 | 0/1 | 1/1 | 0/1 | 1 |
| remit_to | 5 | 3/5 | 4/5 | 3/5 | 2 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| renamed_firm | 1 | 0/1 | 1/1 | 0/1 | 1 |
| report_parameters | 1 | 1/1 | 1/1 | 1/1 | 0 |
| revenue_stamp | 1 | 0/1 | 0/1 | 0/1 | 0 |
| review | 3 | 0/3 | 1/3 | 1/3 | 1 |
| scanned | 12 | 3/12 | 7/12 | 7/12 | 6 |
| sender_block_above_addressee | 1 | 0/1 | 1/1 | 0/1 | 1 |
| sender_in_signoff_only | 1 | 0/1 | 1/1 | 1/1 | 0 |
| service_period | 3 | 1/3 | 1/3 | 2/3 | 0 |
| signatory_trap | 3 | 1/3 | 3/3 | 1/3 | 1 |
| signatory_vs_org | 1 | 0/1 | 0/1 | 0/1 | 0 |
| signature_on_later_page | 3 | 3/3 | 3/3 | 3/3 | 0 |
| signature_only_no_typed_name | 2 | 1/2 | 2/2 | 2/2 | 0 |
| small_print_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| spreadsheet | 2 | 0/2 | 0/2 | 1/2 | 1 |
| staff_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamp_date_only | 2 | 0/2 | 0/2 | 2/2 | 1 |
| stamp_dates | 2 | 0/2 | 1/2 | 1/2 | 1 |
| statement | 4 | 1/4 | 3/4 | 1/4 | 1 |
| struck_through | 1 | 0/1 | 0/1 | 0/1 | 0 |
| subject_is_product | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tables | 1 | 0/1 | 1/1 | 0/1 | 0 |
| third_party_named_in_subject | 1 | 0/1 | 1/1 | 0/1 | 1 |
| tiff | 4 | 1/4 | 3/4 | 3/4 | 0 |
| trading_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tribunal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| trustee_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 3 | 2/3 | 2/3 | 2/3 | 1 |
| two_letterhead_seals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| typist_footer_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| typo_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| uk | 5 | 3/5 | 4/5 | 4/5 | 0 |
| undated | 1 | – | – | 1/1 | 1 |
| us_state | 5 | 3/4 | 3/4 | 5/5 | 2 |
| utility_commission | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wide_image | 1 | 0/1 | 0/1 | 1/1 | 0 |
| windows-1252 | 1 | 0/1 | 1/1 | 0/1 | 0 |
| windows_1252_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| yearless_date | 1 | 1/1 | 1/1 | 1/1 | 0 |
| zero_amount | 2 | 1/2 | 2/2 | 1/2 | 1 |

## Routes

- **Pages:** fast 73 · layout 12 · ocr 44
- **Documents by route class:** fast 26 · layout 5 · ocr 23

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 54 | 49.87 s | 96.65 s | 99.32 s |
| `extraction_wall_ms` | 54 | 16.2 ms | 5.95 s | 16.59 s |
| `worker_total_ms` | 54 | 15.7 ms | 5.95 s | 16.59 s |
| `worker_snapshot_ms` | 54 | 0.22 ms | 2.6 ms | 8.9 ms |
| `worker_parse_ms` | 54 | 3.2 ms | 22.4 ms | 2.37 s |
| `worker_analysis_ms` | 54 | 0.15 ms | 1.6 ms | 5.2 ms |
| `worker_render_ms` | 54 | 0.00 ms | 525.4 ms | 2.15 s |
| `worker_image_decode_ms` | 54 | 0.00 ms | 88.4 ms | 1.01 s |
| `worker_ocr_ms` | 54 | 0.00 ms | 10.52 s | 32.51 s |
| `worker_ocr_encode_ms` | 54 | 0.00 ms | 856.7 ms | 2.26 s |
| `worker_ocr_engine_ms` | 54 | 0.00 ms | 9.41 s | 30.03 s |
| `worker_vision_ms` | 54 | 0.00 ms | 12.4 ms | 28.3 ms |
| `analyze_wall_ms` | 54 | 49.67 s | 95.22 s | 99.32 s |
| `distill_ms` | 54 | 2.0 ms | 20.1 ms | 24.5 ms |
| `prompt_ms` | 54 | 0.03 ms | 0.06 ms | 0.07 ms |
| `inference_ms` | 54 | 49.66 s | 95.21 s | 99.31 s |
| `prefill_ms` | 54 | 25.78 s | 71.22 s | 75.98 s |
| `generation_ms` | 54 | 22.83 s | 27.96 s | 32.38 s |
| `validation_ms` | 54 | 4.3 ms | 16.8 ms | 20.2 ms |
| `naming_ms` | 54 | 0.20 ms | 1.7 ms | 2.8 ms |
| `index_ms` | 54 | 1.7 ms | 18.6 ms | 21.7 ms |
| `retrieval_ms` | 54 | 0.03 ms | 1.3 ms | 3.4 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 54 | 36.7 tok/s | 37.4 tok/s | 37.7 tok/s |
| `generation_tok_per_s` | 54 | 4.1 tok/s | 4.3 tok/s | 4.3 tok/s |
| `prompt_tokens` | 54 | 960 | 2586 | 2778 |
| `cached_tokens` | 54 | 477 | 477 | 477 |
| `generated_tokens` | 54 | 91 | 117 | 134 |
| `estimated_prompt_tokens` | 54 | 1647 | 3006 | 3315 |
| `model_requests` | 54 | 1 | 1 | 1 |
| `redistillations` | 54 | 0 | 0 | 0 |
| `source_characters` | 54 | 2713 | 36778 | 45033 |
| `digest_characters` | 54 | 2946 | 7884 | 8158 |
| `prompt_characters` | 54 | 4995 | 9933 | 10207 |
| `worker_ocr_pages` | 54 | 0 | 4 | 8 |
| `worker_ocr_passes` | 54 | 0 | 4 | 9 |
| `worker_orientation_passes` | 54 | 0 | 4 | 8 |
| `worker_rendered_pixels` | 54 | 0 | 33660000 | 67578072 |
| `index_units` | 54 | 28 | 258 | 464 |
| `context_units` | 54 | 24 | 74 | 112 |
| `context_tokens` | 54 | 1059 | 2419 | 2727 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 26 | 3.7 ms | 21.7 ms | 3.0 ms | 20.2 ms | 19.9 ms | 0.88 ms | 0.00 ms | 0.00 ms |
| layout | 5 | 6.2 ms | 38.3 ms | 5.6 ms | 36.8 ms | 30.1 ms | 5.2 ms | 0.00 ms | 0.00 ms |
| ocr | 23 | 1.87 s | 6.15 s | 1.87 s | 6.15 s | 16.0 ms | 0.90 ms | 1.13 s | 11.27 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 26 | 459.3 ms | 3.70 s | 458.9 ms | 3.70 s | 14.3 ms | 1.1 ms | 92.1 ms | 3.04 s |
| 2-4 | 20 | 5.2 ms | 4.83 s | 4.7 ms | 4.83 s | 22.4 ms | 1.2 ms | 491.6 ms | 9.12 s |
| 5-9 | 6 | 14.8 ms | 16.59 s | 14.1 ms | 16.59 s | 14.5 ms | 1.6 ms | 2.15 s | 32.51 s |
| 10-24 | 2 | 21.7 ms | 38.3 ms | 20.2 ms | 36.8 ms | 30.1 ms | 5.2 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 1.39 s | 2.25 s | 1.39 s | 2.25 s | 14.3 ms | 1.9 ms | 140.5 ms | 2.23 s |
| letter | 22 | 14.8 ms | 4.83 s | 14.1 ms | 4.83 s | 19.9 ms | 0.88 ms | 491.6 ms | 9.12 s |
| notice | 19 | 8.6 ms | 16.59 s | 8.2 ms | 16.59 s | 30.1 ms | 5.2 ms | 2.15 s | 32.51 s |
| receipt | 2 | 534.9 ms | 1.72 s | 534.5 ms | 1.72 s | 9.0 ms | 0.07 ms | 0.00 ms | 1.69 s |
| statement | 4 | 1.9 ms | 2.18 s | 1.3 ms | 2.18 s | 3.2 ms | 1.2 ms | 92.1 ms | 2.07 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 274 MB | 1336 MB | real3-notice-02 |
| Model server | 6085 MB | 6085 MB | real3-office-15 |
| Benchmark (engine) | 21 MB | 21 MB | real3-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (28)

- **real3-letter-01**: expected `2026-06-05 Decision Letter to NBA Properties, Inc.pdf`, got `2026-06-05 Second Request for Reconsideration of Refusal to Register GRAND RAPIDS from U.S. Copyright Office.pdf` · review: LOW_CONFIDENCE
- **real3-letter-03**: expected `2023-03-28 Decision Letter to Good Energy.pdf`, got `2023-03-28 Letter for Gas and Electricity Markets Authority.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-06**: expected `1976-11-18 Letter to Los Alamos Scientific Laboratory.pdf`, got `1976-11-18 Document - Pacific Northwest Laboratories.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-07**: expected `1966-01-04 Letter to U.S. Atomic Energy Commission.pdf`, got `Document - Paul G. Holsted.pdf` · trap: party Paul G. Holsted (addressee, AEC division director) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-08**: expected `1995-12-07 Decision Letter to Bloom & Kreten.pdf`, got `1995-12-07 Letter to Bernhard Kreten.pdf` · trap: party Bernhard Kreten (addressee, attorney at Bloom & Kreten) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-09**: expected `1992-12-24 Objection Letter to Pilcher and Pilcher.pdf`, got `Letter to Pilcher and Pilcher.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-10**: expected `2019-03-28 Response Letter to Transport Scotland.pdf`, got `2019-03-28 Letter to Network Rail Scotland.pdf` · trap: party Network Rail (subject of the letter, not its recipient) · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-13**: expected `1908-02-11 Transmittal Letter to Wright Brothers.tif`, got `1908-02-11 Document - Wright Brothers.tif` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-01**: expected `2024-02-19 Improvement Notice for The Chiltern Railway Company Limited.pdf`, got `Improvement Notice from Office of Rail and Road.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED
- **real3-notice-05**: expected `2008-09-22 Notice of Hearing from Mojave Desert Air Quality Management District.pdf`, got `2008-10-27 Notice of Hearing from Mojave Desert Air Quality Management District.pdf` · trap: date 2008-10-27 (date of the public hearing) · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-13**: expected `1831-11-01 Notice from Parish of Clerkenwell.jpg`, got `1831-11-01 Document.jpg` · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real3-notice-14**: expected `1867-05-01 Notice from Assessors of the City of Boston.tif`, got `1867-05-01 Notice to the Inhabitants and Other Persons Liable to Pay from City of Boston.tif` · review: DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-15**: expected `2026-10-14 Errata Notice from The FreeBSD Project.txt`, got `2026-10-14 Errata Notice from FreeBSD Project.txt` · review: PARTY_UNSUPPORTED
- **real3-invoice-05**: expected `2025-12-05 Statement of Account from Lotus Communications Corp.pdf`, got `2025-12-05 Statement of Account from Media Financial Services.pdf` · trap: party Media Financial Services (agency addressed on the statement) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-09**: expected `1916-11-04 Invoice from Mergenthaler Linotype Co.pdf`, got `INVOICE from Panama City PUB Co.pdf` · trap: party Panama City Pub Co (customer, typed above the issuer) · review: DATE_UNSUPPORTED
- **real3-invoice-10**: expected `1868-10-01 Bill from Haviland & Sutton.tif`, got `Document from M Covace dby.tif` · review: TYPE_MISSING, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-11**: expected `1850-07-11 Bill from Henry D. Gray & Son.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-12**: expected `1848-11-14 Bill from Veranda Hotel.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-13**: expected `1776-07-15 Invoice from Clark & Nightingale.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-14**: expected `1817-06-17 Receipt from Gales & Seaton.jpg`, got `Document.jpg` · review: TYPE_MISSING, DATE_MISSING, DESCRIPTION_INVALID, PARSER_WARNING
- **real3-invoice-15**: expected `1848-07-01 Receipt from Post-Office, Duxbury, Mass.jpg`, got `Document - POST-OFFICE.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real3-office-01**: expected `2026-09-14 Consultation Response to London Borough of Islington.docx`, got `2026-09-14 Letter from London Borough of Islington.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-02**: expected `2018-11-21 Remit Letter to NHS Pay Review Body.docx`, got `2018-11-21 Letter to Office of Manpower Economics.docx` · trap: party Office of Manpower Economics (secretariat whose address is used) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-07**: expected `2020-04-28 Update Letter to Home Affairs Select Committee.odt`, got `2020-04-28 Letter to House of Commons.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-10**: expected `2018-03-21 Email to Royal Borough of Greenwich.msg`, got `2018-03-21 Email from Jillian Holford.msg` · trap: party Jillian Holford (council officer addressed) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-12**: expected `2012-09-30 Statement of Financial Position from Province of British Columbia.xlsx`, got `Statement of Financial Position.xlsx` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-13**: expected `2026-08-07 Air Quality Memo to Stoney Wood Property Developments Ltd.xlsx`, got `2025-11-01 Air Quality Memo GLA Consultation to Stoney Wood Property Developments Ltd.xlsx` · trap: date 2025-11-01 (date of the applicant's air quality assessment) · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-14**: expected `2017-03-31 Condensed Statement of Financial Position from Administrative Tribunals Support Service of Canada.csv`, got `2017-03-31 Condensed Statement of Financial Position unaudited.csv` · review: LOW_CONFIDENCE

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
