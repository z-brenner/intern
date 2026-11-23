# InternBench: live run

- **Run:** 2026-11-23T16:53:33Z · commit `70ce2164a1`
- **Machine:** Intel(R) Xeon(R) 6975P-C, 4 logical cores, 15.3 GB RAM, linux (Ubuntu 24.04.5 LTS)
- **Model:** `intern-local` · 2.74 GB · sha256 `00fe7986ff5f`
- **Corpus:** 53 documents · 53 completed · gold `a9b2c28057`
- **Wall time:** 1779.13 s for the whole run, the worker's start included

## Scorecard

| Score | Result |
| --- | ---: |
| Filename (the whole name) | 18/52 (34.6%) |
| Document type | 42/53 (79.2%) |
| Date | 38/52 (73.1%) |
| Date role (when the reviewed date was chosen) | 16/38 (42.1%) |
| Parties | 33/53 (62.3%) |
| Relation word (when parties were named) | 37/49 (75.5%) |
| Party in the right role (when parties were named) | 21/34 (61.8%) |
| Ready / review routing | 6/31 (19.4%) |
| Description states nothing false | 53/53 (100.0%) |
| Description is specific | 38/53 (71.7%) |
| Description specificity | 63.5% (mean of 53) |
| Evidence recall (model's quotes) | 62.3% (mean of 53) |
| Digest recall (facts reaching the digest) | 79.2% (mean of 53) |
| Prompt recall (facts reaching the prompt sent) | 76.4% (mean of 53) |
| Sent to review | 94.3% of 53 completed |

## Phase 3 scorecard

| Figure | Value | Docs |
| --- | ---: | ---: |
| Long-document filename accuracy (10+ pages) | 50.0% | 2 |
| Complex-document filename accuracy | – | 0 |
| Description completeness | – | 0 |
| Unsupported-fact rate (documents) | 75.5% | 53 |
| Review rate | 94.3% | 53 |
| Evidence recall | 62.3% | 53 |
| Total latency p50 | 31.89 s | 53 |
| Total latency p95 | 58.61 s | 53 |
| Generation latency p50 | 13.30 s | 53 |
| Generation latency p95 | 16.36 s | 53 |
| Generated tokens p50 | 89 | 53 |
| Generated tokens p95 | 110 | 53 |
| Prompt tokens p50 | 960 | 53 |

Long: 10 pages or more. Complex: any of `referenced_agreement`, `middle_fact`, `multi_column`, `layout_parties`, `irrelevant_names`, `information_dense`, `stream_order`, `date_in_table`, `key_value`, `complex_pdf`. Unsupported-fact rate: documents with any `*_UNSUPPORTED` review reason. Latency and tokens over the documents that completed.

## Safety

| Check | Count |
| --- | ---: |
| Filed without review under a wrong name | 1 of 52 |
| Trap date chosen | 3 of 53 |
| Forbidden party named | 6 of 53 |
| Spurious parties named | 16 |
| Description states a known-wrong fact | 0 |
| Unsupported description claims | 0 of 128 (0.0%) |
| Right name sent to review anyway | 10 of 26 |
| Sent to review for an unsupported fact | 40 of 53 |

## By kind

| Kind | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 2/7 | 2/7 | 3/7 | 1/4 | 85.7% | 21.95 s | 43.02 s |
| letter | 21 | 7/21 | 19/21 | 13/21 | 1/10 | 100.0% | 30.96 s | 48.48 s |
| notice | 19 | 7/18 | 14/18 | 15/19 | 3/12 | 94.7% | 37.56 s | 60.31 s |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0/1 | 100.0% | 13.74 s | 19.98 s |
| statement | 4 | 2/4 | 3/4 | 2/4 | 1/4 | 75.0% | 37.96 s | 58.61 s |

## By text layer

| Text layer | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| image | 12 | 3/12 | 6/12 | 5/12 | 0/3 | 100.0% | 19.67 s | 37.56 s |
| native | 14 | 11/13 | 13/13 | 14/14 | 2/12 | 92.9% | 36.13 s | 59.62 s |
| office | 14 | 1/14 | 12/14 | 5/14 | 2/8 | 100.0% | 25.67 s | 49.86 s |
| scan | 10 | 2/10 | 4/10 | 7/10 | 2/5 | 90.0% | 37.96 s | 60.31 s |
| text | 3 | 1/3 | 3/3 | 2/3 | 0/3 | 66.7% | 20.51 s | 52.67 s |

## By page count

| Pages | Docs | Filename | Date | Parties | Routing | Review | p50 total | p95 total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 26 | 9/26 | 17/26 | 14/26 | 3/13 | 92.3% | 21.78 s | 48.63 s |
| 2-4 | 19 | 6/18 | 16/18 | 13/19 | 2/12 | 94.7% | 36.13 s | 60.31 s |
| 5-9 | 6 | 2/6 | 3/6 | 5/6 | 1/4 | 100.0% | 41.61 s | 59.62 s |
| 10-24 | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 100.0% | 49.55 s | 49.86 s |

## By slice

| Slice | Docs | Filename | Date | Parties | Routing | Unsafe ready | p50 total | p95 total | p50 generation | p95 generation |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| long | 2 | 1/2 | 2/2 | 1/2 | 0/2 | 0 of 2 | 49.55 s | 49.86 s | 12.77 s | 18.70 s |

`long`: 10 pages or more. `complex`: any of the categories listed under the phase 3 scorecard. A document may be in both.

## By challenge

| Category | Docs | Filename | Date | Parties | Traps sprung |
| --- | ---: | ---: | ---: | ---: | ---: |
| abbreviated_title | 1 | 0/1 | 0/1 | 0/1 | 0 |
| addressed_to_staff | 1 | 1/1 | 1/1 | 1/1 | 0 |
| agency_billing | 4 | 4/4 | 4/4 | 4/4 | 0 |
| aging_table | 2 | 2/2 | 2/2 | 2/2 | 0 |
| all_caps_parties | 2 | 0/2 | 2/2 | 1/2 | 0 |
| ambiguous_digit | 1 | 1/1 | 1/1 | 1/1 | 0 |
| archivist_annotation | 2 | 0/2 | 0/2 | 0/2 | 0 |
| attachment_with_own_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| attachments | 2 | 1/2 | 1/2 | 2/2 | 0 |
| attachments_with_own_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| bcc_distribution_list | 1 | 0/1 | 0/1 | 0/1 | 1 |
| billhead | 3 | 0/3 | 0/3 | 1/3 | 0 |
| blank_date | 2 | 0/2 | 0/2 | 0/2 | 1 |
| body_recounts_events | 7 | 5/6 | 5/6 | 7/7 | 1 |
| born_digital | 13 | 11/12 | 12/12 | 13/13 | 1 |
| brand_vs_legal_name | 4 | 4/4 | 4/4 | 4/4 | 0 |
| caption_court_differs_from_header | 1 | 1/1 | 1/1 | 1/1 | 0 |
| catalogue_date_trap | 2 | 0/2 | 0/2 | 1/2 | 0 |
| circular | 1 | 0/1 | 1/1 | 0/1 | 0 |
| client_in_subject | 1 | 1/1 | 1/1 | 1/1 | 0 |
| coloured_paper | 2 | 0/2 | 0/2 | 0/2 | 1 |
| comparative_period | 1 | 0/1 | 1/1 | 0/1 | 0 |
| competing_dates | 28 | 11/28 | 22/28 | 21/28 | 3 |
| consignee_prominent | 1 | 0/1 | 0/1 | 0/1 | 0 |
| consultation_response | 1 | 0/1 | 1/1 | 0/1 | 0 |
| copies_list | 1 | 0/1 | 1/1 | 1/1 | 0 |
| corrected_firm_name | 1 | 0/1 | 0/1 | 0/1 | 0 |
| corrected_printed_year | 1 | 0/1 | 0/1 | 0/1 | 0 |
| court_filing | 3 | 3/3 | 3/3 | 3/3 | 0 |
| cover_sheet | 2 | 0/2 | 1/2 | 1/2 | 1 |
| csv | 1 | 0/1 | 1/1 | 0/1 | 0 |
| customer_first | 4 | 0/4 | 0/4 | 0/4 | 1 |
| date_at_end | 6 | 2/6 | 4/6 | 5/6 | 1 |
| date_in_caption | 1 | 1/1 | 1/1 | 1/1 | 0 |
| date_only_in_certificate | 1 | 1/1 | 1/1 | 1/1 | 0 |
| deadline_date | 4 | 0/4 | 4/4 | 2/4 | 0 |
| deadline_in_body | 6 | 2/5 | 3/5 | 5/6 | 3 |
| dear_colleague | 1 | 1/1 | 1/1 | 1/1 | 0 |
| dear_sir_madam | 1 | 1/1 | 1/1 | 1/1 | 0 |
| decision_letter | 1 | 1/1 | 1/1 | 1/1 | 0 |
| decision_numbers_look_like_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| direction | 1 | 0/1 | 1/1 | 1/1 | 0 |
| docket_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| ecf_header_text | 1 | 1/1 | 1/1 | 1/1 | 0 |
| effective_date_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| eighteenth_century | 1 | 0/1 | 0/1 | 0/1 | 0 |
| email | 2 | 1/2 | 2/2 | 1/2 | 1 |
| email_thread | 1 | 0/1 | 1/1 | 0/1 | 1 |
| enclosures_not_included | 1 | 1/1 | 1/1 | 1/1 | 0 |
| enforcement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| engraved_vignette | 1 | 0/1 | 0/1 | 0/1 | 0 |
| file_number | 1 | 1/1 | 1/1 | 1/1 | 0 |
| filing_stamps | 1 | 0/1 | 0/1 | 1/1 | 0 |
| flight_dates | 2 | 2/2 | 2/2 | 2/2 | 0 |
| foi_response | 2 | 0/2 | 2/2 | 1/2 | 1 |
| footnote_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| form | 2 | 0/2 | 0/2 | 2/2 | 0 |
| government_issuer | 1 | 0/1 | 0/1 | 0/1 | 0 |
| handwritten | 6 | 0/6 | 0/6 | 1/6 | 0 |
| handwritten_annotation | 1 | 0/1 | 1/1 | 1/1 | 0 |
| handwritten_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| header_fields | 1 | 0/1 | 1/1 | 1/1 | 0 |
| hearing_date_in_body | 2 | 0/2 | 0/2 | 1/2 | 2 |
| hidden_sheets | 1 | 0/1 | 0/1 | 1/1 | 1 |
| historic | 7 | 0/7 | 0/7 | 1/7 | 1 |
| historical | 6 | 3/6 | 6/6 | 4/6 | 1 |
| image | 6 | 3/6 | 6/6 | 4/6 | 1 |
| image_only | 2 | 0/2 | 0/2 | 2/2 | 0 |
| image_only_body | 1 | 1/1 | 1/1 | 1/1 | 0 |
| improvement_notice | 1 | 0/1 | 0/1 | 1/1 | 0 |
| inside_address_at_end | 1 | 0/1 | 1/1 | 1/1 | 0 |
| invoice | 7 | 2/7 | 2/7 | 3/7 | 1 |
| iso_dates | 1 | 0/1 | 1/1 | 1/1 | 0 |
| jpeg | 6 | 1/6 | 1/6 | 1/6 | 0 |
| junk_ocr_layer | 1 | 0/1 | 0/1 | 0/1 | 1 |
| large_image | 1 | 0/1 | 0/1 | 1/1 | 0 |
| legacy_doc | 2 | 0/2 | 2/2 | 1/2 | 0 |
| letter | 21 | 7/21 | 19/21 | 13/21 | 4 |
| letter_format | 2 | 2/2 | 2/2 | 2/2 | 0 |
| letterhead | 2 | 0/2 | 2/2 | 0/2 | 0 |
| letterhead_logo | 1 | 0/1 | 1/1 | 1/1 | 0 |
| letterhead_names_people | 1 | 1/1 | 1/1 | 1/1 | 0 |
| line_item_dates | 5 | 4/5 | 4/5 | 5/5 | 0 |
| live_date_field | 1 | 0/1 | 1/1 | 1/1 | 0 |
| local_government | 2 | 0/2 | 0/2 | 1/2 | 2 |
| lockbox | 2 | 2/2 | 2/2 | 2/2 | 0 |
| logo_trap | 1 | 0/1 | 1/1 | 0/1 | 0 |
| long_document | 2 | 1/2 | 2/2 | 1/2 | 0 |
| many_dates | 3 | 3/3 | 3/3 | 3/3 | 0 |
| memo | 1 | 0/1 | 0/1 | 1/1 | 1 |
| metadata_date_trap | 5 | 0/5 | 5/5 | 1/5 | 1 |
| metadata_title_trap | 3 | 0/3 | 3/3 | 1/3 | 0 |
| microfilm_targets | 1 | 0/1 | 0/1 | 0/1 | 1 |
| mislabelled_source | 1 | 0/1 | 1/1 | 0/1 | 1 |
| misleading_metadata | 1 | 1/1 | 1/1 | 1/1 | 0 |
| modern | 4 | 4/4 | 4/4 | 4/4 | 0 |
| msg | 2 | 1/2 | 2/2 | 1/2 | 1 |
| multi_page | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_recipient | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multi_round | 1 | 0/1 | 0/1 | 1/1 | 1 |
| multiple_parties_side | 2 | 2/2 | 2/2 | 2/2 | 0 |
| multiple_recipients | 1 | 0/1 | 1/1 | 0/1 | 0 |
| multiple_senders | 1 | 1/1 | 1/1 | 1/1 | 0 |
| multiple_signatories | 1 | 0/1 | 1/1 | 1/1 | 0 |
| named_person_at_org | 8 | 3/8 | 7/8 | 7/8 | 1 |
| network_logo | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_amounts | 1 | 0/1 | 0/1 | 0/1 | 0 |
| no_explicit_title | 2 | 0/2 | 0/2 | 0/2 | 1 |
| no_issuer_in_text | 2 | 0/2 | 1/2 | 0/2 | 0 |
| no_letterhead_on_letter | 1 | 0/1 | 0/1 | 0/1 | 1 |
| no_specific_addressee | 1 | 1/1 | 1/1 | 1/1 | 0 |
| no_title | 1 | 0/1 | 1/1 | 0/1 | 0 |
| no_title_word | 1 | 0/1 | 1/1 | 0/1 | 1 |
| notice | 19 | 7/18 | 14/18 | 15/19 | 4 |
| notice_of_appeal | 1 | 1/1 | 1/1 | 1/1 | 0 |
| notice_of_decision | 2 | 2/2 | 2/2 | 2/2 | 0 |
| notice_of_determination | 1 | 0/1 | 0/1 | 1/1 | 0 |
| notice_of_hearing | 2 | 0/2 | 0/2 | 1/2 | 2 |
| notice_of_removal | 2 | 2/2 | 2/2 | 2/2 | 0 |
| notice_of_violation | 1 | 1/1 | 1/1 | 1/1 | 0 |
| numeric_date | 1 | 0/1 | 0/1 | 0/1 | 1 |
| ocr_layer | 2 | 0/2 | 0/2 | 1/2 | 2 |
| ocr_text_layer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| on_behalf_of_client | 2 | 0/2 | 2/2 | 2/2 | 0 |
| organization_issuer | 1 | 0/1 | 1/1 | 1/1 | 0 |
| penalty_notice | 1 | 1/1 | 1/1 | 1/1 | 0 |
| period_ends_after_date | 1 | 0/1 | 0/1 | 0/1 | 0 |
| pgp_signed | 1 | 0/1 | 1/1 | 1/1 | 0 |
| plain_text | 2 | 1/2 | 2/2 | 2/2 | 0 |
| posted_notice | 3 | 1/3 | 3/3 | 2/3 | 1 |
| print_field_trap | 1 | 0/1 | 0/1 | 0/1 | 0 |
| printed_form | 3 | 0/3 | 0/3 | 0/3 | 1 |
| printer_credit | 1 | 0/1 | 0/1 | 0/1 | 1 |
| public_notice | 5 | 1/4 | 3/4 | 3/5 | 4 |
| publication_number_year_differs | 1 | 1/1 | 1/1 | 1/1 | 0 |
| quaker_date | 1 | 0/1 | 0/1 | 1/1 | 0 |
| quoted_third_party | 1 | 0/1 | 1/1 | 0/1 | 1 |
| re_line_references_document | 4 | 0/4 | 3/4 | 2/4 | 1 |
| receipt | 2 | 0/2 | 0/2 | 0/2 | 0 |
| receipted | 3 | 0/3 | 0/3 | 0/3 | 0 |
| recipient_by_domain_only | 1 | 1/1 | 1/1 | 1/1 | 0 |
| redacted_addressee | 2 | 0/2 | 2/2 | 1/2 | 1 |
| redactions | 1 | 0/1 | 1/1 | 1/1 | 0 |
| reference_numbers | 1 | 1/1 | 1/1 | 1/1 | 0 |
| reference_numbers_look_like_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| relative_dates | 1 | 1/1 | 1/1 | 1/1 | 0 |
| remit_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| remit_to | 5 | 4/5 | 4/5 | 4/5 | 1 |
| remit_to_bank | 1 | 1/1 | 1/1 | 1/1 | 0 |
| renamed_firm | 1 | 0/1 | 1/1 | 0/1 | 1 |
| report_parameters | 1 | 1/1 | 1/1 | 1/1 | 0 |
| revenue_stamp | 1 | 0/1 | 0/1 | 1/1 | 0 |
| review | 3 | 0/3 | 1/3 | 1/3 | 1 |
| scanned | 11 | 2/11 | 5/11 | 8/11 | 4 |
| sender_block_above_addressee | 1 | 0/1 | 1/1 | 1/1 | 0 |
| sender_in_signoff_only | 1 | 0/1 | 1/1 | 0/1 | 0 |
| service_period | 3 | 1/3 | 1/3 | 1/3 | 0 |
| signatory_trap | 3 | 0/3 | 3/3 | 0/3 | 0 |
| signatory_vs_org | 1 | 0/1 | 0/1 | 0/1 | 0 |
| signature_on_later_page | 3 | 2/3 | 2/3 | 3/3 | 0 |
| signature_only_no_typed_name | 2 | 1/2 | 2/2 | 1/2 | 0 |
| small_print_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| spreadsheet | 2 | 0/2 | 0/2 | 1/2 | 1 |
| staff_names | 1 | 1/1 | 1/1 | 1/1 | 0 |
| stamp_date_only | 1 | 0/1 | 0/1 | 0/1 | 2 |
| stamp_dates | 2 | 0/2 | 1/2 | 1/2 | 1 |
| statement | 4 | 2/4 | 3/4 | 2/4 | 0 |
| struck_through | 1 | 0/1 | 0/1 | 1/1 | 0 |
| subject_is_product | 1 | 1/1 | 1/1 | 1/1 | 0 |
| tables | 1 | 0/1 | 1/1 | 0/1 | 0 |
| third_party_named_in_subject | 1 | 0/1 | 1/1 | 1/1 | 0 |
| tiff | 4 | 1/4 | 3/4 | 3/4 | 0 |
| trading_name | 1 | 1/1 | 1/1 | 1/1 | 0 |
| transmittal_letter | 1 | 0/1 | 1/1 | 0/1 | 0 |
| tribunal | 2 | 0/2 | 2/2 | 1/2 | 0 |
| trustee_party | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_digit_year | 3 | 2/3 | 2/3 | 2/3 | 1 |
| two_letterhead_seals | 1 | 1/1 | 1/1 | 1/1 | 0 |
| two_signatories | 1 | 1/1 | 1/1 | 1/1 | 0 |
| typed | 1 | 0/1 | 0/1 | 0/1 | 1 |
| typist_footer_date | 1 | 0/1 | 1/1 | 1/1 | 0 |
| typo_date | 1 | 0/1 | 1/1 | 0/1 | 0 |
| uk | 5 | 3/5 | 4/5 | 4/5 | 1 |
| undated | 1 | – | – | 1/1 | 1 |
| us_state | 5 | 1/4 | 1/4 | 4/5 | 3 |
| utility_commission | 1 | 1/1 | 1/1 | 1/1 | 0 |
| watermark | 1 | 1/1 | 1/1 | 1/1 | 0 |
| wide_image | 1 | 0/1 | 0/1 | 0/1 | 0 |
| windows-1252 | 1 | 0/1 | 1/1 | 0/1 | 0 |
| windows_1252_encoding | 1 | 1/1 | 1/1 | 1/1 | 0 |
| yearless_date | 1 | 0/1 | 1/1 | 0/1 | 1 |
| zero_amount | 2 | 2/2 | 2/2 | 2/2 | 0 |

## Routes

- **Pages:** fast 73 · layout 12 · ocr 42
- **Documents by route class:** fast 26 · layout 5 · ocr 22

## Where the time goes

| Stage | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `total_ms` | 53 | 31.89 s | 58.61 s | 60.31 s |
| `extraction_wall_ms` | 53 | 16.0 ms | 6.04 s | 16.64 s |
| `worker_total_ms` | 53 | 15.6 ms | 6.04 s | 16.64 s |
| `worker_snapshot_ms` | 53 | 0.23 ms | 2.5 ms | 8.9 ms |
| `worker_parse_ms` | 53 | 3.3 ms | 25.0 ms | 1.93 s |
| `worker_analysis_ms` | 53 | 0.14 ms | 1.6 ms | 5.2 ms |
| `worker_render_ms` | 53 | 0.00 ms | 521.8 ms | 2.24 s |
| `worker_image_decode_ms` | 53 | 0.00 ms | 79.2 ms | 1.02 s |
| `worker_ocr_ms` | 53 | 0.00 ms | 10.81 s | 32.49 s |
| `worker_ocr_encode_ms` | 53 | 0.00 ms | 800.4 ms | 2.26 s |
| `worker_ocr_engine_ms` | 53 | 0.00 ms | 9.76 s | 29.96 s |
| `worker_vision_ms` | 53 | 0.00 ms | 22.9 ms | 28.4 ms |
| `analyze_wall_ms` | 53 | 29.28 s | 58.61 s | 59.61 s |
| `distill_ms` | 53 | 1.9 ms | 20.2 ms | 24.5 ms |
| `prompt_ms` | 53 | 0.03 ms | 0.05 ms | 0.07 ms |
| `inference_ms` | 53 | 29.27 s | 58.60 s | 59.59 s |
| `prefill_ms` | 53 | 15.62 s | 43.89 s | 46.41 s |
| `generation_ms` | 53 | 13.30 s | 16.36 s | 19.26 s |
| `validation_ms` | 53 | 4.4 ms | 16.6 ms | 20.1 ms |
| `naming_ms` | 53 | 0.18 ms | 1.7 ms | 2.8 ms |
| `index_ms` | 53 | 1.8 ms | 18.4 ms | 21.7 ms |
| `retrieval_ms` | 53 | 0.03 ms | 1.3 ms | 3.5 ms |

| Measure | Docs | p50 | p95 | Max |
| --- | ---: | ---: | ---: | ---: |
| `prefill_tok_per_s` | 53 | 59.9 tok/s | 62.5 tok/s | 62.9 tok/s |
| `generation_tok_per_s` | 53 | 6.8 tok/s | 7.3 tok/s | 7.4 tok/s |
| `prompt_tokens` | 53 | 960 | 2586 | 2778 |
| `cached_tokens` | 53 | 477 | 477 | 477 |
| `generated_tokens` | 53 | 89 | 110 | 127 |
| `estimated_prompt_tokens` | 53 | 1647 | 3006 | 3315 |
| `model_requests` | 53 | 1 | 1 | 1 |
| `redistillations` | 53 | 0 | 0 | 0 |
| `source_characters` | 53 | 2713 | 36778 | 45033 |
| `digest_characters` | 53 | 2946 | 7884 | 8158 |
| `prompt_characters` | 53 | 4995 | 9933 | 10207 |
| `worker_ocr_pages` | 53 | 0 | 4 | 8 |
| `worker_ocr_passes` | 53 | 0 | 4 | 9 |
| `worker_orientation_passes` | 53 | 0 | 4 | 8 |
| `worker_rendered_pixels` | 53 | 0 | 33660000 | 67578072 |
| `index_units` | 53 | 28 | 258 | 464 |
| `context_units` | 53 | 24 | 74 | 112 |
| `context_tokens` | 53 | 1059 | 2419 | 2727 |

## Extraction time

| Route class | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| fast | 26 | 3.5 ms | 21.3 ms | 3.0 ms | 19.6 ms | 19.3 ms | 0.89 ms | 0.00 ms | 0.00 ms |
| layout | 5 | 6.3 ms | 38.6 ms | 5.8 ms | 37.1 ms | 30.4 ms | 5.2 ms | 0.00 ms | 0.00 ms |
| ocr | 22 | 1.84 s | 6.44 s | 1.84 s | 6.44 s | 16.5 ms | 0.88 ms | 1.14 s | 11.81 s |

| Pages | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 26 | 457.8 ms | 3.80 s | 457.4 ms | 3.80 s | 14.1 ms | 1.1 ms | 102.7 ms | 3.13 s |
| 2-4 | 19 | 5.4 ms | 6.44 s | 4.9 ms | 6.44 s | 1.93 s | 1.6 ms | 521.8 ms | 11.81 s |
| 5-9 | 6 | 14.1 ms | 16.64 s | 13.6 ms | 16.64 s | 14.6 ms | 1.6 ms | 2.24 s | 32.49 s |
| 10-24 | 2 | 21.3 ms | 38.6 ms | 19.6 ms | 37.1 ms | 30.4 ms | 5.2 ms | 0.00 ms | 0.00 ms |

| Kind | Docs | p50 wall | p95 wall | p50 worker | p95 worker | p95 parse | p95 analysis | p95 render | p95 OCR |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| invoice | 7 | 1.41 s | 2.26 s | 1.41 s | 2.26 s | 13.7 ms | 1.9 ms | 134.6 ms | 2.24 s |
| letter | 21 | 14.1 ms | 4.57 s | 13.6 ms | 4.57 s | 19.3 ms | 0.89 ms | 495.6 ms | 8.25 s |
| notice | 19 | 9.0 ms | 16.64 s | 8.4 ms | 16.64 s | 30.4 ms | 5.2 ms | 2.24 s | 32.49 s |
| receipt | 2 | 520.9 ms | 1.69 s | 520.5 ms | 1.69 s | 8.7 ms | 0.07 ms | 0.00 ms | 1.67 s |
| statement | 4 | 1.9 ms | 2.19 s | 1.3 ms | 2.19 s | 3.3 ms | 1.1 ms | 102.7 ms | 2.08 s |

Over the documents that completed. Wall is the runner's clock around the request; worker is the worker's own total. OCR is summed over the pages, so when the worker reads pages in parallel it can exceed the total. A document's route class is the most expensive route any of its pages took (ocr > ocr_regions > layout > fast), or `unrouted` when the worker sent no layouts.

## Memory

| Process | Typical peak (p50) | Highest peak | During |
| --- | ---: | ---: | ---: |
| Parser worker | 275 MB | 1283 MB | real3-notice-02 |
| Model server | 3267 MB | 3267 MB | real3-office-15 |
| Benchmark (engine) | 21 MB | 21 MB | real3-office-15 |

Resident memory sampled every 25 ms per document; a spike shorter than that can be missed.

## Misses (34)

- **real3-letter-01**: expected `2026-06-05 Decision Letter to NBA Properties, Inc.pdf`, got `2026-06-05 Second Request for Reconsideration of Refusal to Register GRAND RAPIDS for NBA Properties, Inc.pdf` · review: LOW_CONFIDENCE
- **real3-letter-06**: expected `1976-11-18 Letter to Los Alamos Scientific Laboratory.pdf`, got `1976-11-18 Document from Pacific Northwest Laboratories.pdf` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-07**: expected `1966-01-04 Letter to U.S. Atomic Energy Commission.pdf`, got `Document - Paul G. Holsted.pdf` · trap: party Paul G. Holsted (addressee, AEC division director) · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-08**: expected `1995-12-07 Decision Letter to Bloom & Kreten.pdf`, got `1995-12-07 Letter from The Register of Copyrights.pdf` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-letter-10**: expected `2019-03-28 Response Letter to Transport Scotland.pdf`, got `2019-03-28 Letter to Rail Transport Scotland.pdf` · review: TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-letter-13**: expected `1908-02-11 Transmittal Letter to Wright Brothers.tif`, got `1908-02-11 Document from U. S. A.tif` · review: TYPE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real3-notice-01**: expected `2024-02-19 Improvement Notice for The Chiltern Railway Company Limited.pdf`, got `Improvement Notice from Office of Rail and Road.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-02**: expected `2007-04-24 Notice of Public Site Visit and Informational Hearing for City of Victorville.pdf`, got `Notice Of Public Site Visit and Informational Hearing for City of Victorville.pdf` · review: DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-04**: expected `2006-02-23 Notice of Determination from City of Industry.pdf`, got `Notice of Determination to Office of Planning and Research.pdf` · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-05**: expected `2008-09-22 Notice of Hearing from Mojave Desert Air Quality Management District.pdf`, got `2008-10-27 Notice of Hearing for Eldon Heaston.pdf` · trap: date 2008-10-27 (date of the public hearing); party Eldon Heaston (Executive Director, recipient of comments) · review: DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-13**: expected `1831-11-01 Notice from Parish of Clerkenwell.jpg`, got `1831-11-01 Document - Thos. Key.jpg` · trap: party Thos. Key (churchwarden who signs) · review: TYPE_UNSUPPORTED, DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, LOW_CONFIDENCE
- **real3-notice-14**: expected `1867-05-01 Notice from Assessors of the City of Boston.tif`, got `1867-05-01 Notice to the Inhabitants and Other Persons Liable to Pay from The Assessors of the City of Boston.tif` · review: DATE_IMPLAUSIBLE, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-notice-15**: expected `2026-10-14 Errata Notice from The FreeBSD Project.txt`, got `2026-10-14 Errata Notice from FreeBSD Project.txt` · review: PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-09**: expected `1916-11-04 Invoice from Mergenthaler Linotype Co.pdf`, got `2016-11-04 Invoice from Panama City PUB Co.pdf` · trap: party Panama City Pub Co (customer, typed above the issuer) · review: DATE_AMBIGUOUS, LOW_CONFIDENCE
- **real3-invoice-10**: expected `1868-10-01 Bill from Haviland & Sutton.tif`, got `Document - Haviland & Sutton.tif` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-11**: expected `1850-07-11 Bill from Henry D. Gray & Son.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-invoice-12**: expected `1848-11-14 Bill from Veranda Hotel.jpg`, got `Document.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-13**: expected `1776-07-15 Invoice from Clark & Nightingale.jpg`, got `Tillighat Invoice from J Orovidenco.jpg` · review: DATE_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-14**: expected `1817-06-17 Receipt from Gales & Seaton.jpg`, got `Document - Mrongh M. Barny.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, PARTY_UNSUPPORTED, PARSER_WARNING, LOW_CONFIDENCE
- **real3-invoice-15**: expected `1848-07-01 Receipt from Post-Office, Duxbury, Mass.jpg`, got `Document from Mr.Dan-L. Wisov.jpg` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, DESCRIPTION_INVALID, PARSER_WARNING, LOW_CONFIDENCE
- **real3-office-01**: expected `2026-09-14 Consultation Response to London Borough of Islington.docx`, got `2026-09-14 Letter to Local Planning Authority.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-02**: expected `2018-11-21 Remit Letter to NHS Pay Review Body.docx`, got `2018-11-21 Letter to Ms Hird.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-03**: expected `2016-03-23 Letter of Concern to Central Bedfordshire Council.docx`, got `2016-03-23 Letter from Local Members of Parliament.docx` · review: TYPE_MISSING, LOW_CONFIDENCE
- **real3-office-04**: expected `2017-08-24 Freedom of Information Response from Office of the Advocate General for Scotland.docx`, got `2017-08-24 Letter to Office of the Advocate General for Scotland.docx` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-05**: expected `2011-10-01 Direction for Ofcom.doc`, got `2011-10-01 Direction to Ofcom to Impose a Notification Condition for Office of Communications.doc` · review: TYPE_INFERRED, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-06**: expected `2014-02-20 Circular from Home Office.doc`, got `2014-02-20 Circular from crime and policing group, policing directorate, police workforce and efficiency.doc` · review: TYPE_INFERRED, PARTY_UNSUPPORTED
- **real3-office-07**: expected `2020-04-28 Update Letter to Home Affairs Select Committee.odt`, got `2020-04-28 Letter to Home Secretary.odt` · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-08**: expected `2021-05-14 Freedom of Information Response from Office of Manpower Economics.odt`, got `2021-05-14 Letter to Rashmi Panigrahi.odt` · trap: party Rashmi Panigrahi (FOI Officer who signs) · review: TYPE_UNSUPPORTED, TYPE_INFERRED, TYPE_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-09**: expected `2018-07-25 Notice of Change of Company Name for Aston Car Group Limited.rtf`, got `2018-07-25 Notice of Change of Company Name for Aston Martin Lagonda Limited.rtf` · review: PARTY_UNSUPPORTED
- **real3-office-10**: expected `2018-03-21 Email to Royal Borough of Greenwich.msg`, got `2018-03-21 Email to Jillian Holford.msg` · trap: party Jillian Holford (council officer addressed) · review: LOW_CONFIDENCE
- **real3-office-12**: expected `2012-09-30 Statement of Financial Position from Province of British Columbia.xlsx`, got `Document.xlsx` · review: TYPE_UNSUPPORTED, DATE_UNSUPPORTED, LOW_CONFIDENCE
- **real3-office-13**: expected `2026-08-07 Air Quality Memo to Stoney Wood Property Developments Ltd.xlsx`, got `2025-11-01 Air Quality Memo GLA Consultation to Stoney Wood Property Developments Ltd.xlsx` · trap: date 2025-11-01 (date of the applicant's air quality assessment) · review: PARTY_UNSUPPORTED
- **real3-office-14**: expected `2017-03-31 Condensed Statement of Financial Position from Administrative Tribunals Support Service of Canada.csv`, got `2017-03-31 Condensed Statement of Financial Position unaudited.csv` · **filed without review**
- **real3-office-15**: expected `2018-07-26 Notice of Change of Company Name for PORSCHE LTD.rtf`, got `2018-07-26 Notice of Change of Company Name from PORSCHE LTD.rtf` · review: PARTY_UNSUPPORTED

## How to compare

Keep this run's JSON, make the change, run again, then:

```text
intern-bench compare --before before.json --after after.json --markdown diff.md
```

It recomputes every rate and count over the documents both runs scored, lists each document whose score flipped, and gives the change in p50/p95 of every stage over the documents both completed - between two live runs only, since a replay's timings are its recording's. `intern-bench report --input report.json --markdown report.md` re-renders this page from the JSON.
